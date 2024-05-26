use std::{collections::{HashMap, HashSet}, net::{IpAddr, Ipv4Addr, Ipv6Addr}, sync::{Arc}};

use crate::{
	context::{self, UPF_PARAMETERS},
	datapath::{PacketPipeline, FlattenedFAR, FlattenedURR, tofino::TofinoBackendError, FlattenedQER, OperationPerfStats},
};

use super::{N4Handlers, PFCPSessionUP};
use async_trait::async_trait;
use libpfcp::{
	handlers::SessionRequestHandlers,
	messages::{
		CreateBAR, CreateFAR, CreatePDR, CreateQER, CreateURR, ForwardingParameters,
		PFCPSessionEstablishmentRequest, PFCPSessionEstablishmentResponse,
		PFCPSessionModificationRequest, UpdatePDR, PDI, PFCPSessionModificationResponse,
	},
	messages::{CreatedPDR, PFCPSessionDeletionRequest, PFCPSessionDeletionResponse},
	models::{Cause, DestinationInterface, NodeID, PFCPSMReqFlags, SourceInterface, FAR_ID},
	PFCPModel, PFCPSessionRulesUP,
};
use log::{error, info, warn};
use tokio::{runtime::Handle, sync::RwLock};

/// Handle CreatePDR, allocates TEID and creates CreatedPDR response
fn handle_establishment_rules(
	rules: &mut PFCPSessionRulesUP,
	create_pdrs: &mut Vec<CreatePDR>,
	created_pdr: &mut Vec<CreatedPDR>,
	session_handle: Arc<RwLock<PFCPSessionUP>>,
) {
	let upf = crate::context::UPF_PARAMETERS.get().unwrap();
	let mut choose_id_map: HashMap<u8, u32> = HashMap::new();
	for pdr in create_pdrs.iter_mut() {
		if let Some(fteid) = pdr.pdi.local_f_teid.as_mut() {
			if fteid.flags.getCH() != 0 {
				if let Some(chid) = fteid.choose_id {
					if let Some(allocated_teid) = choose_id_map.get(&chid) {
						fteid.teid = Some(*allocated_teid);
					} else {
						// allocate teid
						let new_teid = crate::context::TEIDPOOL.write().unwrap().allocate();
						fteid.teid = Some(new_teid);
						choose_id_map.insert(chid, new_teid);
					}
				} else {
					let new_teid = crate::context::TEIDPOOL.write().unwrap().allocate();
					fteid.teid = Some(new_teid);
				}
				fteid.flags.setCH(0);
				if fteid.flags.getV4() != 0 && upf.assigned_ipv4.is_some() {
					fteid.ipv4 = Some(*upf.assigned_ipv4.as_ref().unwrap());
				}
				if fteid.flags.getV6() != 0 && upf.assigned_ipv6.is_some() {
					fteid.ipv6 = Some(*upf.assigned_ipv6.as_ref().unwrap());
				}
			}
			// info!("Assigned data plane F-TEID: {:?}", fteid);
		}
		created_pdr.push(CreatedPDR::from_create_pdr(&pdr));
		rules.pdr_map.insert(pdr.pdr_id.rule_id, pdr.clone());
	}
}

fn tailor_predefined_pdr(predefined_pdr: &mut CreatePDR, pdr: &CreatePDR) {
	if let Some(field) = pdr.pdi.local_f_teid.as_ref() {
		predefined_pdr.pdi.local_f_teid.replace(field.clone()); // repalce predefined one
	}
	if pdr.pdi.ue_ip_address.len() != 0 {
		predefined_pdr.pdi.ue_ip_address = pdr.pdi.ue_ip_address.clone(); // repalce predefined ones
	}
	if pdr.pdi.qfi.len() != 0 {
		predefined_pdr.pdi.qfi = pdr.pdi.qfi.clone(); // repalce predefined ones
	}
	predefined_pdr.precedence = pdr.precedence.clone(); // repalce predefined one
	if let Some(field) = pdr.far_id.as_ref() {
		predefined_pdr.far_id.replace(field.clone()); // repalce predefined one
	}
	if pdr.urr_id.len() != 0 {
		predefined_pdr.urr_id.append(&mut pdr.urr_id.clone()); // extend predefined ones
	}
	if let Some(field) = pdr.qer_id.as_ref() {
		predefined_pdr.qer_id.replace(field.clone()); // repalce predefined one
	}
}

fn tailor_predefined_pdr_update(predefined_pdr: &mut CreatePDR, pdr: &UpdatePDR) {
	if let Some(pdi) = pdr.pdi.as_ref() {
		if let Some(field) = pdi.local_f_teid.as_ref() {
			predefined_pdr.pdi.local_f_teid.replace(field.clone()); // repalce predefined one
		}
		if pdi.ue_ip_address.len() != 0 {
			predefined_pdr.pdi.ue_ip_address = pdi.ue_ip_address.clone(); // repalce predefined ones
		}
		if pdi.qfi.len() != 0 {
			predefined_pdr.pdi.qfi = pdi.qfi.clone(); // repalce predefined ones
		}
	}
	if let Some(precedence) = pdr.precedence.as_ref() {
		predefined_pdr.precedence = precedence.clone(); // repalce predefined one
	}
	if let Some(field) = pdr.far_id.as_ref() {
		predefined_pdr.far_id.replace(field.clone()); // repalce predefined one
	}
	if pdr.urr_id.len() != 0 {
		predefined_pdr.urr_id.append(&mut pdr.urr_id.clone()); // extend predefined ones
	}
	if let Some(field) = pdr.qer_id.as_ref() {
		predefined_pdr.qer_id.replace(field.clone()); // repalce predefined one
	}
}

fn create_packet_pipeline_from_pdr(
	seid: u64,
	pdr: &CreatePDR,
	qers: &Vec<CreateQER>,
	urrs: &Vec<CreateURR>,
	fars: &Vec<CreateFAR>,
	bar: &Option<CreateBAR>,
	pfcpsmflags: Option<&HashMap<FAR_ID, PFCPSMReqFlags>>,
) -> Option<PacketPipeline> {
	if pdr.activate_predefined_rules.len() != 0 {
		None // this PDR serves as an addtion to predefined PDRs
	} else {
		if pdr.far_id.is_none() {
			return None; // no actions, ignore
		}
		let far_id = pdr.far_id.unwrap();
		let far = fars.iter().find(|&o| o.far_id == far_id);
		if far.is_none() {
			return None;
		}
		let far = far.unwrap();
		let mut pipleline = PacketPipeline {
			seid: seid,

			pdr_id: pdr.pdr_id,
			precedence: pdr.precedence.clone(),
			pdi: pdr.pdi.clone(),
			outer_header_removal: pdr.outer_header_removal.clone(),

			qer_id: None,
			gate_status: None,
			// maximum_bitrate: None,
			// guaranteed_bitrate: None,
			qfi: None,

			apply_action: far.apply_action.clone(),
			forwarding_parameters: far.forwarding_parameters.clone(),

			pfcpsm_req_flags: if let Some(flags) = pfcpsmflags {
				flags.get(&far_id).map(|o| o.clone())
			} else {
				None
			},

			urr_ids: pdr.urr_id.clone(),
			// urrs: pdr
			// 	.urr_id
			// 	.iter()
			// 	.filter_map(|urr_id| {
			// 		if let Some(urr) = urrs.iter().find(|&f| f.urr_id == *urr_id) {
			// 			Some(FlattenedURR {
			// 				urr_id: urr.urr_id.rule_id,
			// 				measurement_method: urr.measurement_method.clone(),
			// 				reporting_triggers: urr.reporting_triggers.clone(),
			// 				volume_threshold: urr.volume_threshold.clone(),
			// 				volume_quota: urr.volume_quota.clone(),
			// 				dropped_dl_traffic_threshold: urr.dropped_dl_traffic_threshold.clone(),
			// 				measurement_information: urr.measurement_information.clone(),
			// 				far_for_quota_action: if let Some(far_id) = urr.far_id_for_quota_action
			// 				{
			// 					if let Some(far) = fars.iter().find(|&f| f.far_id == far_id) {
			// 						Some(FlattenedFAR {
			// 							apply_action: far.apply_action.clone(),
			// 							forwarding_parameters: far.forwarding_parameters.clone(),
			// 						})
			// 					} else {
			// 						None
			// 					}
			// 				} else {
			// 					None
			// 				},
			// 				number_of_reports: urr.number_of_reports.clone(),
			// 				measurement_period: urr.measurement_period.clone(),
			// 				event_threshold: urr.event_threshold.clone(),
			// 				event_quota: urr.event_quota.clone(),
			// 				time_threshold: urr.time_threshold.clone(),
			// 				time_quota: urr.time_quota.clone(),
			// 				quota_holding_time: urr.quota_holding_time.clone(),
			// 				quota_validity_time: urr.quota_validity_time.clone(),
			// 				monitoring_time: urr.monitoring_time.clone(),
			// 				subsequent_volume_threshold: urr.subsequent_volume_threshold.clone(),
			// 				subsequent_time_threshold: urr.subsequent_time_threshold.clone(),
			// 				subsequent_volume_quota: urr.subsequent_volume_quota.clone(),
			// 				subsequent_time_quota: urr.subsequent_time_quota.clone(),
			// 				subsequent_event_threshold: urr.subsequent_event_threshold.clone(),
			// 				subsequent_event_quota: urr.subsequent_event_quota.clone(),
			// 				inactivity_detection_time: urr.inactivity_detection_time.clone(),
			// 				linked_urr_id: urr.linked_urr_id.clone(),
			// 				ethernet_inactivity_timer: urr.ethernet_inactivity_timer.clone(),
			// 				additional_monitoring_time: urr.additional_monitoring_time.clone(),
			// 				exempted_application_id_for_quota_action: urr.exempted_application_id_for_quota_action.clone(),
			// 				exempted_sdf_filter_for_quota_action: urr.exempted_sdf_filter_for_quota_action.clone(),
			// 			})
			// 		} else {
			// 			None
			// 		}
			// 	})
			// 	.collect::<Vec<_>>(),

			suggested_buffering_packets_count: if let Some(o) = bar {
				o.suggested_buffering_packets_count.clone()
			} else {
				None
			},
		};
		if let Some(qer_id) = pdr.qer_id {
			if let Some(qer) = qers.iter().find(|&o| o.qer_id == qer_id) {
				pipleline.qer_id = Some(qer.qer_id);
				pipleline.gate_status = Some(qer.gate_status.clone());
				// pipleline.maximum_bitrate = qer.maximum_bitrate.clone();
				// pipleline.guaranteed_bitrate = qer.guaranteed_bitrate.clone();
				pipleline.qfi = qer.qfi.clone();
			}
		};
		Some(pipleline)
	}
}

fn generate_urr_tests(urrs: &mut Vec<CreateURR>) {
	use libpfcp::models::*;
	urrs.clear();
	let mut mm = MeasurementMethod(0);
	mm.setVOLUM(1);
	let mut rt = ReportingTriggers(0);
	rt.setVOLTH(1);
	let vol_thres = VolumeThreshold {
		flags: {
			let mut f = VolumeThresholdFlags(0);
			f.setDLVOL(1);
			f
		},
		total_volume: None,
		uplink_volume: None,
		downlink_volume: Some(600)
	};
	let cu = CreateURR {
		urr_id: URR_ID { rule_id: 0 },
		measurement_method: mm,
		reporting_triggers: rt,
		measurement_period: None,
		volume_threshold: Some(vol_thres),
		volume_quota: None,
		event_threshold: None,
		event_quota: None,
		time_threshold: None,
		time_quota: None,
		quota_holding_time: None,
		dropped_dl_traffic_threshold: None,
		quota_validity_time: None,
		monitoring_time: None,
		subsequent_volume_threshold: None,
		subsequent_time_threshold: None,
		subsequent_volume_quota: None,
		subsequent_time_quota: None,
		subsequent_event_threshold: None,
		subsequent_event_quota: None,
		inactivity_detection_time: None,
		linked_urr_id: vec![],
		measurement_information: None,
		far_id_for_quota_action: None,
		ethernet_inactivity_timer: None,
		additional_monitoring_time: vec![],
		number_of_reports: None,
		exempted_application_id_for_quota_action: vec![],
		exempted_sdf_filter_for_quota_action: vec![],
	};
	urrs.push(cu);
}

#[async_trait]
impl SessionRequestHandlers for N4Handlers {
	async fn handle_session_establishment(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let handle_session_establishment_start = std::time::Instant::now();
		let upf_para = UPF_PARAMETERS.get().unwrap();
		let mut response = PFCPSessionEstablishmentResponse {
			node_id: NodeID::from_ip(upf_para.node_ip),
			cause: Cause::RequestAccepted,
			offending_ie: None,
			up_f_seid: None,
			created_pdr: vec![],
			ago_perf: None
		};
		let mut request = match PFCPSessionEstablishmentRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				println!("failed to decode Request, {}", e);
				return response.encode();
			}
		};
		//generate_urr_tests(&mut request.create_urr);
		let sess_handle = match super::allocate_pfcp_session(&request.cp_f_seid).await {
			Some(s) => s,
			None => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				println!("failed to create session");
				return response.encode();
			}
		};
		let mut sess = sess_handle.write().await;
		// step 1: verify rule IDs
		// step 2: setup rules
		sess.active_rules.pdr_map.clear();
		handle_establishment_rules(
			&mut sess.active_rules,
			&mut request.create_pdr,
			&mut response.created_pdr,
			sess_handle.clone(),
		);
		sess.active_rules.add_batch(
			&request.create_pdr,
			&request.create_far,
			&request.create_urr,
			&request.create_qer,
			&request.create_bar,
		);
		// handle predefined PDRs
		let predefined_content_guard = super::PREDEFINED_RULES.read().await;
		let predefined_content = predefined_content_guard.as_ref().unwrap();
		for pdr in request.create_pdr.iter() {
			for rulename in pdr.activate_predefined_rules.iter() {
				if let Some(predefined_pdr) = predefined_content.PDRs.get(&rulename.name) {
					let mut predefined_pdr = predefined_pdr.clone();
					tailor_predefined_pdr(&mut predefined_pdr, pdr);
					let rule_id = sess.active_rules.predefined_pdr_alloc.allocate().unwrap();
					predefined_pdr.pdr_id.rule_id = rule_id;
					sess.active_rules
						.pdr_map
						.insert(rule_id, predefined_pdr.clone());
					if let Some(rulelist) = sess
						.active_rules
						.active_predefined_rules
						.get_mut(&(pdr.pdr_id.rule_id, rulename.name.clone()))
					{
						rulelist.push(rule_id);
					} else {
						sess.active_rules
							.active_predefined_rules
							.insert((pdr.pdr_id.rule_id, rulename.name.clone()), vec![rule_id]);
					}
				}
			}
		}
		// step 3: create QER and URR
		let flatten_qers = request.create_qer.iter().map(|qer| {
			FlattenedQER {
				qer_id: qer.qer_id.rule_id,
				// gate_status: qer.gate_status.clone(),
				maximum_bitrate: qer.maximum_bitrate.clone(),
				guaranteed_bitrate: qer.guaranteed_bitrate.clone(),
			}
		}).collect::<Vec<_>>();
		let flatten_urrs = request.create_urr.iter().map(|urr| {
			FlattenedURR {
				urr_id: urr.urr_id.rule_id,
				measurement_method: urr.measurement_method.clone(),
				reporting_triggers: urr.reporting_triggers.clone(),
				volume_threshold: urr.volume_threshold.clone(),
				volume_quota: urr.volume_quota.clone(),
				dropped_dl_traffic_threshold: urr.dropped_dl_traffic_threshold.clone(),
				measurement_information: urr.measurement_information.clone(),
				far_for_quota_action: None, // no longer supported!
				number_of_reports: urr.number_of_reports.clone(),
				measurement_period: urr.measurement_period.clone(),
				event_threshold: urr.event_threshold.clone(),
				event_quota: urr.event_quota.clone(),
				time_threshold: urr.time_threshold.clone(),
				time_quota: urr.time_quota.clone(),
				quota_holding_time: urr.quota_holding_time.clone(),
				quota_validity_time: urr.quota_validity_time.clone(),
				monitoring_time: urr.monitoring_time.clone(),
				subsequent_volume_threshold: urr.subsequent_volume_threshold.clone(),
				subsequent_time_threshold: urr.subsequent_time_threshold.clone(),
				subsequent_volume_quota: urr.subsequent_volume_quota.clone(),
				subsequent_time_quota: urr.subsequent_time_quota.clone(),
				subsequent_event_threshold: urr.subsequent_event_threshold.clone(),
				subsequent_event_quota: urr.subsequent_event_quota.clone(),
				inactivity_detection_time: urr.inactivity_detection_time.clone(),
				linked_urr_id: urr.linked_urr_id.clone(),
				ethernet_inactivity_timer: urr.ethernet_inactivity_timer.clone(),
				additional_monitoring_time: urr.additional_monitoring_time.clone(),
				exempted_application_id_for_quota_action: urr.exempted_application_id_for_quota_action.clone(),
				exempted_sdf_filter_for_quota_action: urr.exempted_sdf_filter_for_quota_action.clone(),
				enable_dwd: urr.reporting_triggers.getREDWD() != 0
			}
		}).collect::<Vec<_>>();
		// step 3: create packet pipeline
		let pipelines = request
			.create_pdr
			.iter()
			.filter_map(|pdr| {
				create_packet_pipeline_from_pdr(
					sess.local_f_seid.seid,
					pdr,
					&request.create_qer,
					&request.create_urr,
					&request.create_far,
					&request.create_bar,
					None,
				)
			})
			.collect::<Vec<_>>();
		sess.active_pipelines = pipelines.clone();
		// step 4: send to backend
		let mut teid_set: Vec<u32> = Vec::new();
		for pipeline in pipelines.iter() {
			pipeline
				.pdi
				.local_f_teid
				.as_ref()
				.map(|f_teid| f_teid.teid.map(|o| teid_set.push(o)));
		}
		crate::context::TEIDPOOL
			.write()
			.unwrap()
			.mark_reference(sess.local_f_seid.seid, teid_set);
		let perf_start = std::time::Instant::now();
		let other_rule_start = std::time::Instant::now();
		let handle_session_establishment_elp = handle_session_establishment_start.elapsed().as_micros() as u64;
		let other_rule_elp;
		let result_of_update = {
			let be = context::BACKEND.get().unwrap();
			//todo!("error handling");
			if flatten_qers.len() != 0 {
				be.update_or_add_qer(sess.local_f_seid.seid, flatten_qers).await;
			}
			if flatten_urrs.len() != 0 {
				be.update_or_add_urr(sess.local_f_seid.seid, flatten_urrs).await;
			}
			other_rule_elp = other_rule_start.elapsed().as_micros() as u64;
			be.update_or_add_forwarding(sess.local_f_seid.seid, pipelines).await
		};
		let perf_session_mod_elpased = perf_start.elapsed().as_micros() as u64;
		if let Err(e) = &result_of_update {
			if let Some(tbe) = e.downcast_ref::<TofinoBackendError>() {
				error!("{:?}", tbe);
				todo!()
			}
		}
		let stats = result_of_update.unwrap();
		response.up_f_seid = Some(sess.local_f_seid.clone());
		response.ago_perf = Some(libpfcp::models::AgoUpfPerfReport { stats1: handle_session_establishment_elp, stats2: other_rule_elp, stats3: stats.stats3, stats4: stats.stats4, stats_total: perf_session_mod_elpased });
		//println!("created_pdr: {:?}", response.created_pdr);
		response.encode()
	}

	async fn handle_session_modification(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let handle_session_modification_start = std::time::Instant::now();
		let upf_para = UPF_PARAMETERS.get().unwrap();
		let mut response = PFCPSessionModificationResponse {
			cause: Cause::RequestAccepted,
			offending_ie: None,
			created_pdr: vec![],
			ago_perf: None
		};
		let mut request = match PFCPSessionModificationRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				println!("failed to decode Request, {}", e);
				return response.encode();
			}
		};
		let local_seid = match header.seid {
			Some(a) => a,
			None => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				println!("no SEID in header");
				return response.encode();
			}
		};
		let pfcp_ctx_opt = super::GLOBAL_PFCP_CONTEXT.read().await;
		let be = context::BACKEND.get().unwrap();
		let pfcp_ctx = pfcp_ctx_opt.as_ref().unwrap();
		if let Some(sess_handle) = pfcp_ctx.PfcpSessions.get(&local_seid) {
			let mut sess = sess_handle.write().await;
			let old_urrs = sess.active_rules.urr_map.values().map(|v| v.clone()).collect::<Vec<_>>();
			let old_qers = sess.active_rules.qer_map.values().map(|v| v.clone()).collect::<Vec<_>>();
			let old_pipelines = sess.active_pipelines.clone();
			// step 1: handle delete
			let pdrs_to_delete = request
				.remove_pdr
				.iter()
				.map(|o| o.pdr_id)
				.collect::<Vec<_>>();
			let fars_to_delete = request
				.remove_far
				.iter()
				.map(|o| o.far_id)
				.collect::<Vec<_>>();
			let urrs_to_delete = request
				.remove_urr
				.iter()
				.map(|o| o.urr_id)
				.collect::<Vec<_>>();
			let qers_to_delete = request
				.remove_qer
				.iter()
				.map(|o| o.qer_id)
				.collect::<Vec<_>>();
			let bar_to_delete = request.remove_bar.as_ref().map(|o| o.bar_id);
			sess.active_rules.delete_batch(
				&pdrs_to_delete,
				&fars_to_delete,
				&urrs_to_delete,
				&qers_to_delete,
				&bar_to_delete,
			);
			// step 2: handle creation
			let mut created_pdrs = vec![];
			handle_establishment_rules(
				&mut sess.active_rules,
				&mut request.create_pdr,
				&mut created_pdrs,
				sess_handle.clone(),
			);
			let predefined_content_guard = super::PREDEFINED_RULES.read().await;
			let predefined_content = predefined_content_guard.as_ref().unwrap();
			// 29.244 5.19
			for pdr in request.create_pdr.iter() {
				for rulename in pdr.activate_predefined_rules.iter() {
					if let Some(predefined_pdr) = predefined_content.PDRs.get(&rulename.name) {
						let mut predefined_pdr = predefined_pdr.clone();
						tailor_predefined_pdr(&mut predefined_pdr, pdr);
						let rule_id = sess.active_rules.predefined_pdr_alloc.allocate().unwrap();
						predefined_pdr.pdr_id.rule_id = rule_id;
						sess.active_rules
							.pdr_map
							.insert(rule_id, predefined_pdr.clone());
						if let Some(rulelist) = sess
							.active_rules
							.active_predefined_rules
							.get_mut(&(pdr.pdr_id.rule_id, rulename.name.clone()))
						{
							rulelist.push(rule_id);
						} else {
							sess.active_rules
								.active_predefined_rules
								.insert((pdr.pdr_id.rule_id, rulename.name.clone()), vec![rule_id]);
						}
					}
				}
			}
			sess.active_rules.add_batch(
				&request.create_pdr,
				&request.create_far,
				&request.create_urr,
				&request.create_qer,
				&request.create_bar,
			);
			// step 3: handle modification
			// PDR
			for update_pdr in request.update_pdr.iter() {
				let id = update_pdr.pdr_id.rule_id;
				if let Some(pdr) = sess.active_rules.pdr_map.get_mut(&id) {
					update_pdr
						.precedence
						.as_ref()
						.map(|o| pdr.precedence = o.clone());
					update_pdr.pdi.as_ref().map(|o| pdr.pdi = o.clone());
					update_pdr
						.outer_header_removal
						.as_ref()
						.map(|o| pdr.outer_header_removal.replace(o.clone()));
					update_pdr
						.far_id
						.as_ref()
						.map(|o| pdr.far_id.replace(o.clone()));
					if update_pdr.urr_id.len() != 0 {
						pdr.urr_id = update_pdr.urr_id.clone()
					}
					update_pdr
						.qer_id
						.as_ref()
						.map(|o| pdr.qer_id.replace(o.clone()));
					if update_pdr.activate_predefined_rules.len() != 0 {
						pdr.activate_predefined_rules = update_pdr.activate_predefined_rules.clone()
					}
					update_pdr
						.activation_time
						.as_ref()
						.map(|o| pdr.activation_time.replace(o.clone()));
					update_pdr
						.deactivation_time
						.as_ref()
						.map(|o| pdr.deactivation_time.replace(o.clone()));
					if update_pdr.ip_multicast_addressing_info.len() != 0 {
						pdr.ip_multicast_addressing_info =
							update_pdr.ip_multicast_addressing_info.clone()
					}
					update_pdr
						.transport_delay_reporting
						.as_ref()
						.map(|o| pdr.transport_delay_reporting.replace(o.clone()));
				}
				// handle deactivate predefined rules
				let mut ruleids_to_delete = vec![];
				for de_rulename in update_pdr.deactivate_predefined_rules.iter() {
					if let Some(rule_ids) = sess
						.active_rules
						.active_predefined_rules
						.remove(&(id, de_rulename.name.clone()))
					{
						for id in rule_ids.iter() {
							ruleids_to_delete.push(*id);
						}
					}
				}
				for id in ruleids_to_delete {
					sess.active_rules.pdr_map.remove(&id);
					sess.active_rules.predefined_pdr_alloc.free(id);
				}
				// handle activate predefined rules
				for rulename in update_pdr.activate_predefined_rules.iter() {
					if let Some(predefined_pdr) = predefined_content.PDRs.get(&rulename.name) {
						let mut predefined_pdr = predefined_pdr.clone();
						tailor_predefined_pdr_update(&mut predefined_pdr, update_pdr);
						let rule_id = sess.active_rules.predefined_pdr_alloc.allocate().unwrap();
						predefined_pdr.pdr_id.rule_id = rule_id;
						sess.active_rules
							.pdr_map
							.insert(rule_id, predefined_pdr.clone());
						if let Some(rulelist) = sess
							.active_rules
							.active_predefined_rules
							.get_mut(&(id, rulename.name.clone()))
						{
							rulelist.push(rule_id);
						} else {
							sess.active_rules
								.active_predefined_rules
								.insert((id, rulename.name.clone()), vec![rule_id]);
						}
					}
				}
			}
			// FAR
			let mut pfcpsm_map = HashMap::new();
			for update_far in request.update_far.iter() {
				let id = update_far.far_id.rule_id;
				if let Some(far) = sess.active_rules.far_map.get_mut(&id) {
					update_far
						.apply_action
						.as_ref()
						.map(|o| far.apply_action = o.clone());
					update_far
						.bar_id
						.as_ref()
						.map(|o| far.bar_id.replace(o.clone()));
					update_far
						.redundant_transmission_forwarding_parameters
						.as_ref()
						.map(|o| {
							far.redundant_transmission_forwarding_parameters
								.replace(o.clone())
						});
					update_far.update_forwarding_parameters.as_ref().map(|ufp| {
						if let Some(fp) = far.forwarding_parameters.as_mut() {
							ufp.destination_interface
								.as_ref()
								.map(|o| fp.destination_interface = o.clone());
							ufp.network_instnace
								.as_ref()
								.map(|o| fp.network_instnace.replace(o.clone()));
							ufp.redirect_information
								.as_ref()
								.map(|o| fp.redirect_information.replace(o.clone()));
							ufp.outer_header_creation
								.as_ref()
								.map(|o| fp.outer_header_creation.replace(o.clone()));
							ufp.transport_level_marking
								.as_ref()
								.map(|o| fp.transport_level_marking.replace(o.clone()));
							ufp.forwarding_policy
								.as_ref()
								.map(|o| fp.forwarding_policy.replace(o.clone()));
							ufp.header_enrichment
								.as_ref()
								.map(|o| fp.header_enrichment.replace(o.clone()));
							ufp.linked_traffic_endpoint_id
								.as_ref()
								.map(|o| fp.linked_traffic_endpoint_id.replace(o.clone()));
							ufp.destination_interface_type
								.as_ref()
								.map(|o| fp.destination_interface_type.replace(o.clone()));
						} else {
							let fp = ForwardingParameters {
								destination_interface: ufp
									.destination_interface
									.as_ref()
									.map_or(DestinationInterface::CoreSide, |f| f.clone()),
								network_instnace: ufp.network_instnace.clone(),
								redirect_information: ufp.redirect_information.clone(),
								outer_header_creation: ufp.outer_header_creation.clone(),
								transport_level_marking: ufp.transport_level_marking.clone(),
								forwarding_policy: ufp.forwarding_policy.clone(),
								header_enrichment: ufp.header_enrichment.clone(),
								linked_traffic_endpoint_id: ufp.linked_traffic_endpoint_id.clone(),
								pfcpsm_req_flags: ufp.pfcpsm_req_flags.clone(),
								proxying: None,
								destination_interface_type: ufp.destination_interface_type.clone(),
								data_network_access_identifier: ufp
									.data_network_access_identifier
									.clone(),
							};
							far.forwarding_parameters.replace(fp);
						}
						if let Some(pfcpsm) = ufp.pfcpsm_req_flags.as_ref() {
							pfcpsm_map.insert(update_far.far_id, pfcpsm.clone());
						}
					});
				}
			}
			// URR
			for update_urr in request.update_urr.iter() {
				let id = update_urr.urr_id.rule_id;
				if let Some(urr) = sess.active_rules.urr_map.get_mut(&id) {
					update_urr
						.measurement_method
						.as_ref()
						.map(|o| urr.measurement_method = o.clone());
					update_urr
						.reporting_triggers
						.as_ref()
						.map(|o| urr.reporting_triggers = o.clone());
					update_urr
						.volume_threshold
						.as_ref()
						.map(|o| urr.volume_threshold.replace(o.clone()));
					update_urr
						.volume_quota
						.as_ref()
						.map(|o| urr.volume_quota.replace(o.clone()));
					update_urr
						.dropped_dl_traffic_threshold
						.as_ref()
						.map(|o| urr.dropped_dl_traffic_threshold.replace(o.clone()));
					if update_urr.linked_urr_id.len() != 0 {
						urr.linked_urr_id = update_urr.linked_urr_id.clone()
					}
					update_urr
						.measurement_information
						.as_ref()
						.map(|o| urr.measurement_information.replace(o.clone()));
					update_urr
						.far_id_for_quota_action
						.as_ref()
						.map(|o| urr.far_id_for_quota_action.replace(o.clone()));
					update_urr
						.number_of_reports
						.as_ref()
						.map(|o| urr.number_of_reports.replace(o.clone()));
				}
			}
			// QER
			for update_qer in request.update_qer.iter() {
				let id = update_qer.qer_id.rule_id;
				if let Some(qer) = sess.active_rules.qer_map.get_mut(&id) {
					update_qer
						.qer_corrleation_id
						.as_ref()
						.map(|o| qer.qer_corrleation_id.replace(o.clone()));
					update_qer
						.gate_status
						.as_ref()
						.map(|o| qer.gate_status = o.clone());
					update_qer
						.maximum_bitrate
						.as_ref()
						.map(|o| qer.maximum_bitrate.replace(o.clone()));
					update_qer
						.guaranteed_bitrate
						.as_ref()
						.map(|o| qer.guaranteed_bitrate.replace(o.clone()));
					update_qer.qfi.as_ref().map(|o| qer.qfi.replace(o.clone()));
					update_qer.rqi.as_ref().map(|o| qer.rqi.replace(o.clone()));
					update_qer
						.paging_policy_indicator
						.as_ref()
						.map(|o| qer.paging_policy_indicator.replace(o.clone()));
					update_qer
						.averaging_window
						.as_ref()
						.map(|o| qer.averaging_window.replace(o.clone()));
					update_qer
						.qer_control_indications
						.as_ref()
						.map(|o| qer.qer_control_indications.replace(o.clone()));
				}
			}
			// BAR
			for update_bar in request.update_bar.iter() {
				let id = update_bar.bar_id.rule_id;
				if let Some(bar) = sess.active_rules.bar_map.get_mut(&id) {
					update_bar
						.downlink_data_notification_delay
						.as_ref()
						.map(|o| bar.downlink_data_notification_delay.replace(o.clone()));
					update_bar
						.suggested_buffering_packets_count
						.as_ref()
						.map(|o| bar.suggested_buffering_packets_count.replace(o.clone()));
				}
			}
			// step 4: create BE pipelines using now updated rules
			let fars = sess.active_rules.get_fars();
			let qers = sess.active_rules.get_qers();
			let urrs = sess.active_rules.get_urrs();
			let bars = sess.active_rules.get_bars();
			// create new pipelines
			let pipelines = sess
				.active_rules
				.get_pdrs()
				.iter()
				.filter_map(|pdr| {
					create_packet_pipeline_from_pdr(
						sess.local_f_seid.seid,
						pdr,
						&qers,
						&urrs,
						&fars,
						&bars.first().map(|a| a.clone()),
						Some(&pfcpsm_map),
					)
				})
				.collect::<Vec<_>>();
			sess.active_pipelines = pipelines.clone();
			for pipeline in sess.active_pipelines.iter_mut() {
				pipeline.pfcpsm_req_flags = None; // clear PFCPSMReq flag, so the next time we can still request end marker send without changing the rest of FAR
			}
			// find pipelines differences (modified_or_added / deleted)
			let mut updated_or_added_pipelines = vec![];
			let mut updated_or_added_urrs = vec![];
			let mut updated_or_added_qers = vec![];
			let mut deleted_pipelines = vec![];
			for n in pipelines.iter() {
				let mut is_new_or_updated = true;
				for o in old_pipelines.iter() {
					if *n == *o {
						is_new_or_updated = false;
						break;
					}
				}
				if is_new_or_updated {
					updated_or_added_pipelines.push(n.clone());
				}
			}
			for o in old_pipelines.iter() {
				let mut found = false;
				for n in pipelines.iter() {
					if n.pdr_id == o.pdr_id {
						found = true;
						break;
					}
				}
				if !found {
					deleted_pipelines.push(o.pdr_id);
				}
			}
			for n in urrs.iter() {
				let mut is_new_or_updated = true;
				for o in old_urrs.iter() {
					if *n == *o {
						is_new_or_updated = false;
						break;
					}
				}
				if is_new_or_updated {
					updated_or_added_urrs.push(n.clone());
				}
			}
			for n in qers.iter() {
				let mut is_new_or_updated = true;
				for o in old_qers.iter() {
					if *n == *o {
						is_new_or_updated = false;
						break;
					}
				}
				if is_new_or_updated {
					updated_or_added_qers.push(n.clone());
				}
			}
			// mark TEID reference
			let mut teid_set: Vec<u32> = Vec::new();
			for pipeline in pipelines.iter() {
				pipeline
					.pdi
					.local_f_teid
					.as_ref()
					.map(|f_teid| f_teid.teid.map(|o| teid_set.push(o)));
			}
			crate::context::TEIDPOOL
				.write()
				.unwrap()
				.mark_reference(local_seid, teid_set);
			// step 3: create QER and URR updates from updated rules
			let flatten_qers = updated_or_added_qers.iter().map(|qer| {
				FlattenedQER {
					qer_id: qer.qer_id.rule_id,
					// gate_status: qer.gate_status.clone(),
					maximum_bitrate: qer.maximum_bitrate.clone(),
					guaranteed_bitrate: qer.guaranteed_bitrate.clone(),
				}
			}).collect::<Vec<_>>();
			let flatten_urrs = updated_or_added_urrs.iter().map(|urr| {
				FlattenedURR {
					urr_id: urr.urr_id.rule_id,
					measurement_method: urr.measurement_method.clone(),
					reporting_triggers: urr.reporting_triggers.clone(),
					volume_threshold: urr.volume_threshold.clone(),
					volume_quota: urr.volume_quota.clone(),
					dropped_dl_traffic_threshold: urr.dropped_dl_traffic_threshold.clone(),
					measurement_information: urr.measurement_information.clone(),
					far_for_quota_action: None, // no longer supported!
					number_of_reports: urr.number_of_reports.clone(),
					measurement_period: urr.measurement_period.clone(),
					event_threshold: urr.event_threshold.clone(),
					event_quota: urr.event_quota.clone(),
					time_threshold: urr.time_threshold.clone(),
					time_quota: urr.time_quota.clone(),
					quota_holding_time: urr.quota_holding_time.clone(),
					quota_validity_time: urr.quota_validity_time.clone(),
					monitoring_time: urr.monitoring_time.clone(),
					subsequent_volume_threshold: urr.subsequent_volume_threshold.clone(),
					subsequent_time_threshold: urr.subsequent_time_threshold.clone(),
					subsequent_volume_quota: urr.subsequent_volume_quota.clone(),
					subsequent_time_quota: urr.subsequent_time_quota.clone(),
					subsequent_event_threshold: urr.subsequent_event_threshold.clone(),
					subsequent_event_quota: urr.subsequent_event_quota.clone(),
					inactivity_detection_time: urr.inactivity_detection_time.clone(),
					linked_urr_id: urr.linked_urr_id.clone(),
					ethernet_inactivity_timer: urr.ethernet_inactivity_timer.clone(),
					additional_monitoring_time: urr.additional_monitoring_time.clone(),
					exempted_application_id_for_quota_action: urr.exempted_application_id_for_quota_action.clone(),
					exempted_sdf_filter_for_quota_action: urr.exempted_sdf_filter_for_quota_action.clone(),
        			enable_dwd: false,
				}
			}).collect::<Vec<_>>();
			let handle_session_modification_op_elp = handle_session_modification_start.elapsed().as_micros() as u64;
			// send to backend
			let perf_start = std::time::Instant::now();
			let overhead_timer_perf;
			let update_time;
			let result_of_update = {
				let overhead_timer = std::time::Instant::now();
				///todo!("error handling");
				if qers_to_delete.len() != 0 {
					be.delete_qer(local_seid, qers_to_delete).await;
				}
				if flatten_qers.len() != 0 {
					be.update_or_add_qer(local_seid, flatten_qers).await;
				}
				if urrs_to_delete.len() != 0 {
					be.delete_urr(local_seid, urrs_to_delete).await;
				}
				if flatten_urrs.len() != 0 {
					be.update_or_add_urr(local_seid, flatten_urrs).await;
				}
				if deleted_pipelines.len() != 0 {
					be.delete_forwarding(local_seid, deleted_pipelines).await;
				}
				overhead_timer_perf = overhead_timer.elapsed().as_micros() as u64;
				let update_timer = std::time::Instant::now();
				let r = be.update_or_add_forwarding(local_seid, updated_or_added_pipelines).await;
				update_time = update_timer.elapsed().as_micros() as u64;
				r
			};
			let perf_session_mod_elpased = perf_start.elapsed().as_micros() as u64;
			if let Err(e) = &result_of_update {
				if let Some(tbe) = e.downcast_ref::<TofinoBackendError>() {
					todo!()
				}
			}
			let stats = result_of_update.unwrap();
			// step 4: create response
			response.created_pdr = created_pdrs;
			response.ago_perf = Some(libpfcp::models::AgoUpfPerfReport { stats1: handle_session_modification_op_elp, stats2: overhead_timer_perf, stats3: stats.stats3, stats4: stats.stats4, stats_total: perf_session_mod_elpased });
			response.encode()
		} else {
			response.cause = Cause::SessionContextNotFound;
			error!("PFCP Session not found");
			response.encode()
		}
	}

	async fn handle_session_deletion(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let upf_para = UPF_PARAMETERS.get().unwrap();
		let mut response = PFCPSessionDeletionResponse {
			cause: Cause::RequestAccepted,
			offending_ie: None,
			usage_report: vec![],
			ago_perf: None
		};
		let request = match PFCPSessionDeletionRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				error!("failed to decode Request, {}", e);
				return response.encode();
			}
		};
		let local_seid = match header.seid {
			Some(a) => a,
			None => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				error!("no SEID in header");
				return response.encode();
			}
		};

		//info!("[SEID={}] Deleting current PFCP session", local_seid);
		let (stats, reports) = super::delete_pfcp_session(local_seid).await;
		let mut usage_reports = vec![];
		for r in reports.into_iter() {
			match r.report {
				crate::datapath::Report::UsageReports(r) => usage_reports.append(&mut r.into_iter().map(|a| a.to_deletion_report()).collect::<Vec<_>>()),
				_ => {
					
				}
			}
		}
		// mark TEID reference (remove all TEID reference of this Session)
		crate::context::TEIDPOOL
			.write()
			.unwrap()
			.mark_reference(local_seid, Vec::new());
		response.usage_report = usage_reports;

		//info!("[SEID={}] Deleted PFCP Session", local_seid);
		response.encode()
	}

	async fn handle_session_report(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		unreachable!() // only CP receives report
	}
}
