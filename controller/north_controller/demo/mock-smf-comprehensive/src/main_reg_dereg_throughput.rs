use std::{sync::Arc, net::Ipv4Addr, time::{SystemTime, Duration, Instant, UNIX_EPOCH}, fs::{File, OpenOptions}, io::Write, collections::HashSet};

use libpfcp::{messages::{PDI, ForwardingParameters, UpdateForwardingParameters}, models::{ApplyAction, UE_IPAddress, OuterHeaderCreation, F_TEID, QFI, Precedence, OuterHeaderRemovalDescription, OuterHeaderRemoval, URR_ID, PDNType, GateStatus, FAR_ID}, PFCPError, PFCPSessionCP};
use log::{info, error};
use n4::UPFNodeContext;
use itertools::Itertools;

// comprehensive batched requests

use crate::{sender::GtpClient, n4::{handlers::N4Handlers, add_upf}};

fn generate_urr_tests_time(rules: &mut libpfcp::PFCPSessionRules, dur: u32) -> Result<URR_ID, PFCPError> {
	use libpfcp::models::*;
	let mut mm = MeasurementMethod(0);
	mm.setDURAT(1);
	let mut rt = ReportingTriggers(0);
	rt.setTIMTH(1);
	let dur_thres = TimeThreshold {
    	value: dur,
	};
	rules.create_urr(
		mm,
		rt,
		None,
		None,
		None,
		None,
		None,
		Some(dur_thres),
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		vec![],
		None,
		None,
		None,
		vec![],
		None,
		vec![],
		vec![]
	)
}


fn generate_urr_tests_vol(rules: &mut libpfcp::PFCPSessionRules, dl_thres: Option<u64>, ul_thres: Option<u64>) -> Result<URR_ID, PFCPError> {
	use libpfcp::models::*;
	let mut mm = MeasurementMethod(0);
	mm.setVOLUM(1);
	let mut rt = ReportingTriggers(0);
	rt.setVOLTH(1);
	let vol_thres = VolumeThreshold {
		flags: {
			let mut f = VolumeThresholdFlags(0);
			if dl_thres.is_some() {
				f.setDLVOL(1);
			}
			if ul_thres.is_some() {
				f.setULVOL(1);
			}
			f
		},
		total_volume: None,
		uplink_volume: ul_thres,
		downlink_volume: dl_thres
	};
	rules.create_urr(
		mm,
		rt,
		None,
		Some(vol_thres),
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		vec![],
		None,
		None,
		None,
		vec![],
		None,
		vec![],
		vec![]
	)
}

fn generate_urr_tests_start_of_traffic(rules: &mut libpfcp::PFCPSessionRules, dl_thres: Option<u64>, ul_thres: Option<u64>) -> Result<URR_ID, PFCPError> {
	use libpfcp::models::*;
	let mut mm = MeasurementMethod(0);
	let mut rt = ReportingTriggers(0);
	rt.setSTART(1);
	rules.create_urr(
		mm,
		rt,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		vec![],
		None,
		None,
		None,
		vec![],
		None,
		vec![],
		vec![]
	)
}


pub async fn select_upf() -> Option<UPFNodeContext> {
	let nodes = crate::n4::PFCP_NODES_GLOBAL_CONTEXT.nodes.read().await;
	for (ip, item) in nodes.iter() {
		return Some(item.clone());
	}
	None
}

pub const MIN_BYTES_BEFORE_THRES: u64 = 300;
pub const THRES_RANGE: u64 = 3 * 1000;
use rand::prelude::*;
pub struct UE {
	pub upf_seid: u64,
	pub vol_thres: u64,
	pub time_thres: u64,
	pub num_bytes_sent: u64,
	pub vol_thres_reach_time_ms: u64,
	pub time_thres_reach_time_ms: u64,
	pub teid: Option<u32>,
	pub ue_ip: std::net::Ipv4Addr,
	pub dl_far_id: FAR_ID,
	pub session_handle: Option<Arc<RwLock<PFCPSessionCP>>>
}

impl UE {
	pub fn new(ue_ip: std::net::Ipv4Addr, thres: u64) -> Self {
		Self {
			upf_seid: 0,
			vol_thres: MIN_BYTES_BEFORE_THRES + thres,
			time_thres: 0,
			num_bytes_sent: 0,
			vol_thres_reach_time_ms: 0,
			time_thres_reach_time_ms: 0,
			teid: None,
			ue_ip: ue_ip,
			dl_far_id: FAR_ID { rule_id: 0 },
			session_handle: None
		}
	}
	pub async fn step_1(&mut self) {
		let mut rules = libpfcp::PFCPSessionRules::new();
		let mut upf_handle = select_upf().await.unwrap();
		let mut upf = upf_handle.node.write().await;
		let qfi = 1u8;
		let urr_id = generate_urr_tests_vol(&mut rules, Some(1400), Some(1400)).unwrap();
		//let urr_id = generate_urr_tests_time(&mut rules, 5).unwrap();
		let bd_qer = rules.create_qer(
			None,
			{ let mut gs = GateStatus(0); gs.setDLDate(0); gs.setULGate(0); gs },
			None,
			None,
			Some(QFI(qfi & 0b00111111)),
			None,
			None,
			None,
			None
		).unwrap();
		let dl_far = rules.create_far(
			{
				let mut action = ApplyAction(0); // do nothing
				action.setFORW(1);
				action
			},
			None,
			None,
			None
		).unwrap();
		let dl_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::CoreSide,
			local_f_teid: None,
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![{
				let mut ue_ip = UE_IPAddress::new();
				{
					ue_ip.flags.setV4(1);
					ue_ip.flags.setSD(1);
					ue_ip.ipv4 = Some(self.ue_ip);
				}
				ue_ip
			}],
			traffic_endpoint_id: vec![],
				sdf_filter: None,
				application_id: None,
				ethernet_pdu_session_nformation: None,
				ethernet_pakcet_filter: None,
				qfi: vec![],
				framed_route: vec![],
				framed_routing: None,
				framed_ipv6_route: vec![],
				source_interface_type: None,
				ip_multicast_addressing_info: vec![],
		};
		let dl_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			dl_pdi,
			None,
			Some(dl_far),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let ul_far = rules.create_far(
			{
				let mut action = ApplyAction(0);
				action.setFORW(1); // forwarding
				action
			},
			Some(ForwardingParameters {
				destination_interface: libpfcp::models::DestinationInterface::CoreSide,
				network_instnace: None,
				redirect_information: None,
				outer_header_creation: None, // UE's IP packet goes out without any addtions
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				linked_traffic_endpoint_id: None,
				pfcpsm_req_flags: None,
				proxying: None,
				destination_interface_type: None,
				data_network_access_identifier: None,
			}),
			None,
			None
		).unwrap();
		let ul_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::AccessSide,
			local_f_teid: Some(F_TEID::new_choose(true, false)),
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![],
			traffic_endpoint_id: vec![],
			sdf_filter: None,
			application_id: None,
			ethernet_pdu_session_nformation: None,
			ethernet_pakcet_filter: None,
			qfi: vec![QFI(qfi)],
			framed_route: vec![],
			framed_routing: None,
			framed_ipv6_route: vec![],
			source_interface_type: None,
			ip_multicast_addressing_info: vec![],
		};
		let ul_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			ul_pdi,
			Some(OuterHeaderRemoval {
				desc: OuterHeaderRemovalDescription::GTP_U_UDP_IP, // remove GTP-U header
				ext_header_deletion: None,
			}),
			Some(ul_far.clone()),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let (session_handle, created_pdr, _) = upf.create_pfcp_session(Some(rules), Some(PDNType::IPv4)).await.unwrap();
		self.session_handle = Some(session_handle.clone());
		let session = session_handle.read().await;
		let cn_tunnel_endpoint = session.active_rules.pdr_map.get(&ul_pdr.rule_id).unwrap().pdi.local_f_teid.as_ref().unwrap().clone();
		self.teid = cn_tunnel_endpoint.teid;
		self.upf_seid = session.remote_seid.unwrap();
		self.dl_far_id = dl_far;
	}
	pub async fn step_2(&mut self, an_ip: std::net::Ipv4Addr) {
		let teid = F_TEID::from_ip_teid(std::net::IpAddr::V4(an_ip), 1);
		let mut ohc = OuterHeaderCreation::new();
		// use IPv4
		teid.ipv4.as_ref().map(|ip| {
			ohc.desc.setGTP_U_UDP_IPv4(1);
			ohc.ipv4 = Some(*ip);
		});
		// if IPv6 is available, use IPv6 instead
		teid.ipv6.as_ref().map(|ip| {
			ohc.desc.setGTP_U_UDP_IPv4(0);
			ohc.desc.setGTP_U_UDP_IPv6(1);
			ohc.ipv4 = None;
			ohc.ipv6 = Some(*ip);
		});
		ohc.teid = Some(*teid.teid.as_ref().unwrap());
		let mut pfcp = self.session_handle.as_ref().unwrap().write().await;
		let mut rules = pfcp.transaction_begin();
		rules.update_far(
			self.dl_far_id,
			Some({
				let mut apply_actions = ApplyAction(0);
				apply_actions.setFORW(1);
				apply_actions
			}),
			Some(UpdateForwardingParameters {
				destination_interface: Some(libpfcp::models::DestinationInterface::AccessSide),
				network_instnace: None,
				redirect_information: None,
				outer_header_creation: Some(ohc),
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				pfcpsm_req_flags: None,
				linked_traffic_endpoint_id: None,
				destination_interface_type: None,
				data_network_access_identifier: None,
			}),
			None,
			None
		);
		pfcp.transaction_commit(rules).await.unwrap();
	}
	pub async fn activate(&mut self, an_ip: std::net::Ipv4Addr) -> (u64, u64, u64, u64, u64) {
		let mut rules = libpfcp::PFCPSessionRules::new();
		let upf_handle = select_upf().await.unwrap();
		let upf = upf_handle.node.read().await;
		let qfi = 1u8;
		let urr_id = generate_urr_tests_start_of_traffic(&mut rules, None, Some(self.vol_thres)).unwrap();
		let bd_qer = rules.create_qer(
			None,
			{ let mut gs = GateStatus(0); gs.setDLDate(0); gs.setULGate(0); gs },
			None,
			None,
			Some(QFI(qfi & 0b00111111)),
			None,
			None,
			None,
			None
		).unwrap();
		let dl_far = rules.create_far(
			{
				let mut action = ApplyAction(0); // do nothing
				action.setFORW(1);
				action
			},
			Some(
				{
					let mut ohc = OuterHeaderCreation::new();
					ohc.desc.setGTP_U_UDP_IPv4(1);
					ohc.ipv4 = Some(an_ip);
					ohc.teid = Some(1);
					ForwardingParameters {
						destination_interface: libpfcp::models::DestinationInterface::AccessSide,
						network_instnace: None,
						redirect_information: None,
						outer_header_creation: Some(ohc),
						transport_level_marking: None,
						forwarding_policy: None,
						header_enrichment: None,
						linked_traffic_endpoint_id: None,
						pfcpsm_req_flags: None,
						proxying: None,
						destination_interface_type: None,
						data_network_access_identifier: None,
					}
				}
			),
			None,
			None
		).unwrap();
		let dl_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::CoreSide,
			local_f_teid: None,
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![{
				let mut ue_ip = UE_IPAddress::new();
				{
					ue_ip.flags.setV4(1);
					ue_ip.flags.setSD(1);
					ue_ip.ipv4 = Some(self.ue_ip);
				}
				ue_ip
			}],
			traffic_endpoint_id: vec![],
				sdf_filter: None,
				application_id: None,
				ethernet_pdu_session_nformation: None,
				ethernet_pakcet_filter: None,
				qfi: vec![],
				framed_route: vec![],
				framed_routing: None,
				framed_ipv6_route: vec![],
				source_interface_type: None,
				ip_multicast_addressing_info: vec![],
		};
		let dl_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			dl_pdi,
			None,
			Some(dl_far),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let ul_far = rules.create_far(
			{
				let mut action = ApplyAction(0);
				action.setFORW(1); // forwarding
				action
			},
			Some(ForwardingParameters {
				destination_interface: libpfcp::models::DestinationInterface::CoreSide,
				network_instnace: None,
				redirect_information: None,
				outer_header_creation: None, // UE's IP packet goes out without any addtions
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				linked_traffic_endpoint_id: None,
				pfcpsm_req_flags: None,
				proxying: None,
				destination_interface_type: None,
				data_network_access_identifier: None,
			}),
			None,
			None
		).unwrap();
		let ul_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::AccessSide,
			local_f_teid: Some(F_TEID::new_choose(true, false)),
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![],
			traffic_endpoint_id: vec![],
			sdf_filter: None,
			application_id: None,
			ethernet_pdu_session_nformation: None,
			ethernet_pakcet_filter: None,
			qfi: vec![QFI(qfi)],
			framed_route: vec![],
			framed_routing: None,
			framed_ipv6_route: vec![],
			source_interface_type: None,
			ip_multicast_addressing_info: vec![],
		};
		let ul_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			ul_pdi,
			Some(OuterHeaderRemoval {
				desc: OuterHeaderRemovalDescription::GTP_U_UDP_IP, // remove GTP-U header
				ext_header_deletion: None,
			}),
			Some(ul_far.clone()),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let t0 = std::time::Instant::now();
		let resp = upf.create_pfcp_session(Some(rules), Some(PDNType::IPv4)).await;
		let delay = t0.elapsed().as_micros() as u64;
		if let Err(e) = &resp {
			error!("err {}",self.ue_ip);
		}
		let (session_handle, created_pdr, ago_perf) = resp.unwrap();
		self.session_handle = Some(session_handle.clone());
		let session = session_handle.read().await;
		let cn_tunnel_endpoint = session.active_rules.pdr_map.get(&ul_pdr.rule_id).unwrap().pdi.local_f_teid.as_ref().unwrap().clone();
		self.teid = cn_tunnel_endpoint.teid;
		self.upf_seid = session.remote_seid.unwrap();
		(0, 0, 0, 0, delay)
	}
	pub async fn handover(&mut self, new_gnb_ip: Ipv4Addr) {
		let teid = F_TEID::from_ip_teid(std::net::IpAddr::V4(new_gnb_ip), 1);
		let mut ohc = OuterHeaderCreation::new();
		// use IPv4
		teid.ipv4.as_ref().map(|ip| {
			ohc.desc.setGTP_U_UDP_IPv4(1);
			ohc.ipv4 = Some(*ip);
		});
		// if IPv6 is available, use IPv6 instead
		teid.ipv6.as_ref().map(|ip| {
			ohc.desc.setGTP_U_UDP_IPv4(0);
			ohc.desc.setGTP_U_UDP_IPv6(1);
			ohc.ipv4 = None;
			ohc.ipv6 = Some(*ip);
		});
		ohc.teid = Some(*teid.teid.as_ref().unwrap());
		let mut pfcp = self.session_handle.as_ref().unwrap().write().await;
		let mut rules = pfcp.transaction_begin();
		rules.update_far(
			self.dl_far_id,
			Some({
				let mut apply_actions = ApplyAction(0);
				apply_actions.setFORW(1);
				apply_actions
			}),
			Some(UpdateForwardingParameters {
				destination_interface: Some(libpfcp::models::DestinationInterface::AccessSide),
				network_instnace: None,
				redirect_information: None,
				outer_header_creation: Some(ohc),
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				pfcpsm_req_flags: None,
				linked_traffic_endpoint_id: None,
				destination_interface_type: None,
				data_network_access_identifier: None,
			}),
			None,
			None
		);
		pfcp.transaction_commit(rules).await.unwrap();
	}
	pub async fn dereg(&mut self) -> u64 {
		let mut pfcp = self.session_handle.as_ref().unwrap().write().await;
		let t0 = std::time::Instant::now();
		pfcp.release_with_report().await;
		let delay = t0.elapsed().as_micros() as u64;
		delay
	}
}

mod n4;
mod sender;

pub const NUM_UE: usize = 150000;


use lazy_static::lazy_static;


#[derive(Clone, Copy, Debug)]
pub struct Report {
	pub vol_thres_reach_time_ms: u64,
	pub traffic_change_time_ms: u64,
	pub total_bytes: u64,
}
lazy_static! {
	pub static ref N4_REPORT_TIME: dashmap::DashMap<u64, Report> = dashmap::DashMap::new();
	pub static ref REF_REPORT_TIME: dashmap::DashMap<u64, Report> = dashmap::DashMap::new();
}

use rand::thread_rng;
use tokio::{sync::RwLock};

fn mean_stddev(numbers: &[f64]) -> (f64, f64) {
	let mean = numbers.iter().sum::<f64>() as f64 / numbers.len() as f64;
	let variance: f64 = numbers.iter().map(|&num| {
        let diff = num - mean;
        diff * diff
    }).sum::<f64>() / numbers.len() as f64;
    (mean, variance.sqrt())
}

#[tokio::main]
async fn main() {
	env_logger::init();

	let bs = 10;
	let upfname = "hip4-upf";
	let round_time_ms = format!("tp-test-budget0-bs{}", bs);

	let HO_PER_SEC = 100usize;
	let REG_PER_SEC = 100usize;
	let DEREG_PER_SEC = 100usize;

	let mut rng = rand::thread_rng();

	info!("Building PFCP Context");
	let pfcp_handler = N4Handlers {};
	let join_handles = libpfcp::setup_global_pfcp_contexts(pfcp_handler, None, tokio::runtime::Handle::current());

	let upf_ip_cp = std::net::IpAddr::V4("<IP of P4 switch>".parse().unwrap()); // IP of P4 switch 
	let smf_ip_cp = std::net::IpAddr::V4("<IP of current machine>".parse().unwrap()); // IP of current machine

	
	let self_ip = std::net::Ipv4Addr::new(10, 201, 0, 1);
	let upf_ip = std::net::Ipv4Addr::new(10, 201, 0, 2);

	info!("Connecting to UPF {}",  upf_ip_cp);
	add_upf(upf_ip_cp, smf_ip_cp).await;
	
	let ues = Arc::new(RwLock::new(Vec::with_capacity(NUM_UE)));
	let mut ue_tasks = vec![];
	info!("Creating PFCP sessions");
	for i in 0..NUM_UE {
		let thres = rng.gen_range(0..THRES_RANGE);
		let ues_cloned = ues.clone();
		ue_tasks.push(
			//tokio::spawn(
				async move {
					let ue_ip = Ipv4Addr::from((i as u32) | 0xff000000u32);
					let mut ue = UE::new(ue_ip, thres);
					ue.activate(self_ip).await;
					ues_cloned.write().await.push(RwLock::new(ue));
				}
			//)
		);
	}
	for t in ue_tasks {
		t.await;
	}
	
	// info!("wait for 2000sec!");
	// std::thread::sleep(Duration::from_secs(2000));
	info!("Sending traffic!");

	let mut indices = Vec::with_capacity(NUM_UE);
	for i in 0..NUM_UE {
		indices.push(i);
	}

	let mut dereged_ues = vec![];
	let mut num_ue_to_dereg = 0usize;

	let mut ctr = 0usize;

	let mut all_dereg_throughputs = Vec::with_capacity(500);
	let all_dereg_latencies = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));

	let mut all_reg_throughputs = Vec::with_capacity(500);
	let all_reg_latencies = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));
	let all_reg_latencies1 = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));
	let all_reg_latencies2 = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));
	let all_reg_latencies3 = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));
	let all_reg_latencies4 = Arc::new(RwLock::new(Vec::with_capacity(500 * 200)));

	let mut dereg_phase = true;

	loop {
		if dereg_phase {
			// dereg
			indices.shuffle(&mut thread_rng());
			let mut num_dereg = 0usize;
			
			let mut offset = 0;
			let mut ret = Vec::with_capacity(2000);
			let cur_time = std::time::Instant::now();
			while cur_time.elapsed().as_micros() < 1000000 && offset < NUM_UE {
				// do in 200 UE batch
				let mut tasks = Vec::with_capacity(bs);
				for i in 0..bs {
					let idx = indices[offset];
					let ue22 = ues.clone();
					let all_dereg_latencies_cloned = all_dereg_latencies.clone();
					tasks.push(
						tokio::task::spawn(async move {
							let ues2 = ue22.read().await;
							let x_locked = ues2.get(idx).unwrap();
							let mut x = x_locked.write().await;
							let delay = x.dereg().await;
							all_dereg_latencies_cloned.write().await.push(delay);
						})
					);
					ret.push(idx);
					num_dereg += 1;
					offset += 1;
				}
				for t in tasks {
					t.await.unwrap();
				}
			}
			let elp = cur_time.elapsed().as_secs_f64();

			dereged_ues = ret;

			all_dereg_throughputs.push((num_dereg as f64) / elp);
			println!("Total {} dereg", num_dereg);
			dereg_phase = false;
		} else {
			// re-reg
			let cur_time = std::time::Instant::now();
			for chunk in dereged_ues.iter().chunks(bs).into_iter() {
				let mut tasks = Vec::with_capacity(dereged_ues.len());
				for ue in chunk {
					let idx = *ue;
					let ue22 = ues.clone();
					let all_reg_latencies_cloned = all_reg_latencies.clone();
					let all_reg_latencies1_cloned = all_reg_latencies1.clone();
					let all_reg_latencies2_cloned = all_reg_latencies2.clone();
					let all_reg_latencies3_cloned = all_reg_latencies3.clone();
					let all_reg_latencies4_cloned = all_reg_latencies4.clone();
					tasks.push(
						tokio::task::spawn(async move {
							let ues2 = ue22.read().await;
							let x_locked = ues2.get(idx).unwrap();
							let mut x = x_locked.write().await;						
							let (delay1, delay2, delay3, delay4, delay_total) = x.activate(self_ip).await;
							all_reg_latencies_cloned.write().await.push(delay_total);
							all_reg_latencies1_cloned.write().await.push(delay1);
							all_reg_latencies2_cloned.write().await.push(delay2);
							all_reg_latencies3_cloned.write().await.push(delay3);
							all_reg_latencies4_cloned.write().await.push(delay4);
						})
					);
				}
				for t in tasks {
					t.await.unwrap();
				}
			}
			
			let used = cur_time.elapsed().as_secs_f64();
			let tp = dereged_ues.len() as f64 / used;
			all_reg_throughputs.push(tp);
			println!("Total {} reg", dereged_ues.len());
			dereged_ues.clear();
			dereg_phase = true;
		}
		
		ctr += 1;
		if ctr > 200 {
			break;
		}
	}

	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-latency-{}.csv", round_time_ms))
			.unwrap();
		let mut all_reg_latencies = all_reg_latencies.write().await;
		all_reg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_latencies.len() - 1) as f64) as usize;
			let val = all_reg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-latency-sub1-{}.csv", round_time_ms))
			.unwrap();
		let mut all_reg_latencies = all_reg_latencies1.write().await;
		all_reg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_latencies.len() - 1) as f64) as usize;
			let val = all_reg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-latency-sub2-{}.csv", round_time_ms))
			.unwrap();
		let mut all_reg_latencies = all_reg_latencies2.write().await;
		all_reg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_latencies.len() - 1) as f64) as usize;
			let val = all_reg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-latency-sub3-{}.csv", round_time_ms))
			.unwrap();
		let mut all_reg_latencies = all_reg_latencies3.write().await;
		all_reg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_latencies.len() - 1) as f64) as usize;
			let val = all_reg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-latency-sub4-{}.csv", round_time_ms))
			.unwrap();
		let mut all_reg_latencies = all_reg_latencies4.write().await;
		all_reg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_latencies.len() - 1) as f64) as usize;
			let val = all_reg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-reg-throughput-{}.csv", round_time_ms))
			.unwrap();
		all_reg_throughputs.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_reg_throughputs.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_reg_throughputs.len() - 1) as f64) as usize;
			let val = all_reg_throughputs[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-dereg-latency-{}.csv", round_time_ms))
			.unwrap();
		let mut all_dereg_latencies = all_dereg_latencies.write().await;
		all_dereg_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_dereg_latencies.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_dereg_latencies.len() - 1) as f64) as usize;
			let val = all_dereg_latencies[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}
	{
		let mut file_csv = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(format!("{upfname}-dereg-throughput-{}.csv", round_time_ms))
			.unwrap();
		all_dereg_throughputs.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let (avg, stddev) = mean_stddev(&all_dereg_throughputs.iter().map(|f| *f as f64).collect::<Vec<_>>());
		file_csv.write(format!("avg,{}\n", avg).as_bytes()).unwrap();
		file_csv.write(format!("stddev,{}\n", stddev).as_bytes()).unwrap();
		for p in 0..101 {
			let p2 = p as f64 / 100.0f64;
			let idx = (p2 * (all_dereg_throughputs.len() - 1) as f64) as usize;
			let val = all_dereg_throughputs[idx];
			file_csv.write(format!("{},{}\n", p2, val).as_bytes()).unwrap();
		}
	}

	println!("Done");
	std::process::exit(0);
}
