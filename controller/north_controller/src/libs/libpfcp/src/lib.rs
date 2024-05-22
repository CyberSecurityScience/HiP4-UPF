
extern crate num;
#[macro_use]
extern crate num_derive;

use std::{collections::{HashMap, HashSet}, future::Future, net::UdpSocket, ops::{AddAssign, BitAndAssign, Shl, ShlAssign}, task::{Poll, Waker}, thread::JoinHandle};
use std::sync::Arc;
use tokio::sync::RwLock;
use handlers::{NodeRequestHandlers, SessionRequestHandlers};
use messages::*;
use models::*;
use num::{Bounded, Integer};
use once_cell::sync::OnceCell;
use tokio::sync;

use lazy_static::lazy_static;

pub trait PFCPModel {
	const ID: u16;
	fn encode(&self) -> Vec<u8>;
	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized;
}

pub mod models;
pub mod messages;
pub mod senders;
pub mod handlers;
pub mod helpers;

use std::error::Error;
use std::fmt;

use self::models::PFCPHeader;

#[derive(Debug)]
pub struct PFCPError {
	details: String,
}

impl PFCPError {
	pub fn new_boxed(msg: &str) -> Box<PFCPError> {
		Box::new(PFCPError {
			details: msg.to_string(),
		})
	}
	pub fn new(msg: &str) -> PFCPError {
		PFCPError {
			details: msg.to_string(),
		}
	}
}

impl fmt::Display for PFCPError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.details)
	}
}

impl Error for PFCPError {
	fn description(&self) -> &str {
		&self.details
	}
}

#[derive(Debug, Clone)]
pub struct IDAllocator<T: Copy + Integer + AddAssign + Bounded + Shl + BitAndAssign + PartialEq + ShlAssign> {
	pub counter: T,
	pub freed: Vec<T>,
	pub freed_unusable: Vec<T>,
	pub transaction: bool,
	pub forbidden_bit: Option<T>
}
impl<T: Copy + Integer + AddAssign + Bounded + Shl + BitAndAssign + PartialEq + ShlAssign> IDAllocator<T> {
	pub fn reset(&mut self) {
		self.counter = T::one();
		self.freed = Vec::new();
		self.freed_unusable = Vec::new();
		self.transaction = false;
	}
	pub fn new() -> IDAllocator<T> {
		Self {
			counter: T::one(),
			freed: Vec::new(),
			freed_unusable: Vec::new(),
			transaction: false,
			forbidden_bit: None
		}
	}
	pub fn new_with_forbidden_bit(bitpos: T) -> IDAllocator<T> {
		Self {
			counter: T::one(),
			freed: Vec::new(),
			freed_unusable: Vec::new(),
			transaction: false,
			forbidden_bit: Some(bitpos)
		}
	}
	pub fn new_with_counter(counter: T) -> IDAllocator<T> {
		Self {
			counter: counter,
			freed: Vec::new(),
			freed_unusable: Vec::new(),
			transaction: false,
			forbidden_bit: None
		}
	}
	pub fn allocate(&mut self) -> Result<T, PFCPError> {
		if let Some(id) = self.freed.pop() {
			return Ok(id);
		}
		let ret = self.counter;
		if ret == T::max_value() {
			return Err(PFCPError::new("too many IDs"));
		}
		self.counter += T::one();
		if let Some(fb) = self.forbidden_bit {
			let mut cmp = self.counter;
			let mut mask = T::one();
			mask <<= fb;
			cmp &= mask;
			if cmp != T::zero() {
				self.counter += mask; // skip this range
			}
		}
		Ok(ret)
	}
	pub fn free(&mut self, id: T) {
		if self.transaction {
			self.freed_unusable.push(id);
		} else {
			self.freed.push(id);
		}
	}
	pub fn transaction_begin(&mut self) {
		self.transaction = true;
	}
	pub fn transaction_commit(&mut self) {
		self.transaction = false;
		self.freed.append(&mut self.freed_unusable);
	}
}

#[derive(Debug, Clone)]
pub struct IDAllocatorRc<T: Copy + Integer + AddAssign + Bounded + Shl + BitAndAssign + PartialEq + ShlAssign> {
	pub alloc: IDAllocator<T>
}
impl<T: Copy + Integer + AddAssign + Bounded + Shl + BitAndAssign + PartialEq + ShlAssign> IDAllocatorRc<T> {
	pub fn new() -> Self {
		Self {
			alloc: IDAllocator::new()
		}
	}
}

/// Representing a CP PFCP entity
#[derive(Debug)]
pub struct PFCPNodeManager {
	/// IP of SMF
	pub local_ip: std::net::IpAddr,
	/// IP of UPF
	pub remote_ip: std::net::IpAddr,
	/// Sequence number for all messages
	pub seq: Arc<sync::Mutex<u32>>,
	/// Make sure only one PFCP request is on going
	pub up_lock: sync::Mutex<i32>,
	pub seid_pool: Arc<sync::Mutex<IDAllocator<u64>>>,

	pub dst_port_override: Option<u16>,

	/// Map CP-SEID to PFCPSession
	pub managed_pfcp_sessions: dashmap::DashMap<u64, Arc<RwLock<PFCPSessionCP>>>
}

#[derive(Clone, Debug)]
pub struct PFCPSessionRulesUP {
	/// PDR Allcoator
	pub predefined_pdr_alloc: IDAllocator<u16>,

	/// PDR Map
	pub pdr_map: HashMap<u16, CreatePDR>,
	/// FAR Map
	pub far_map: HashMap<u32, CreateFAR>,
	/// URR Map
	pub urr_map: HashMap<u32, CreateURR>,
	/// QER Map
	pub qer_map: HashMap<u32, CreateQER>,
	/// BAR Map
	pub bar_map: HashMap<u8, CreateBAR>,

	/// Predefined PDR rules
	pub active_predefined_rules: HashMap<(u16, String), Vec<u16>>,
}
impl PFCPSessionRulesUP {
	pub fn new() -> PFCPSessionRulesUP {
		PFCPSessionRulesUP {
			predefined_pdr_alloc: IDAllocator::new_with_counter(0b1000_0000_0000_0000),
			pdr_map: HashMap::new(),
			far_map: HashMap::new(),
			urr_map: HashMap::new(),
			qer_map: HashMap::new(),
			bar_map: HashMap::new(),
			active_predefined_rules: HashMap::new(),
		}
	}
	// pub fn activate_predefined_pdr(&mut self, name: &str) {

	// }
	// pub fn deactivate_predefined_pdr(&mut self, name: &str) {

	// }
	pub fn get_pdrs(&self) -> Vec<CreatePDR> {
		self.pdr_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_fars(&self) -> Vec<CreateFAR> {
		self.far_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_urrs(&self) -> Vec<CreateURR> {
		self.urr_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_qers(&self) -> Vec<CreateQER> {
		self.qer_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_bars(&self) -> Vec<CreateBAR> {
		self.bar_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn add_batch(&mut self, pdrs: &Vec<CreatePDR>, fars: &Vec<CreateFAR>, urrs: &Vec<CreateURR>, qers: &Vec<CreateQER>, bars: &Option<CreateBAR>) {
		for it in pdrs.iter() {
			self.pdr_map.insert(it.pdr_id.rule_id, it.clone());
		}
		for it in fars.iter() {
			if !it.far_id.is_predefined() {
				self.far_map.insert(it.far_id.rule_id, it.clone());
			}
		}
		for it in urrs.iter() {
			if !it.urr_id.is_predefined() {
				self.urr_map.insert(it.urr_id.rule_id, it.clone());
			}
		}
		for it in qers.iter() {
			if !it.qer_id.is_predefined() {
				self.qer_map.insert(it.qer_id.rule_id, it.clone());
			}
		}
		if let Some(it) = bars {
			self.bar_map.insert(it.bar_id.rule_id, it.clone());
		}
	}
	pub fn delete_batch(&mut self, pdrs: &Vec<PDR_ID>, fars: &Vec<FAR_ID>, urrs: &Vec<URR_ID>, qers: &Vec<QER_ID>, bar: &Option<BAR_ID>) {
		for it in pdrs.iter() {
			self.pdr_map.remove(&it.rule_id);
		}
		for it in fars.iter() {
			if !it.is_predefined() {
				self.far_map.remove(&it.rule_id);
			}
		}
		for it in urrs.iter() {
			if !it.is_predefined() {
				self.urr_map.remove(&it.rule_id);
			}
		}
		for it in qers.iter() {
			if !it.is_predefined() {
				self.qer_map.remove(&it.rule_id);
			}
		}
		if let Some(it) = bar {
			self.bar_map.remove(&it.rule_id);
		}
	}
}

#[derive(Clone, Debug)]
pub struct PFCPSessionRules {
	/// PDR Allcoator
	pub pdr_alloc: IDAllocator<u16>,
	/// FAR Allcoator
	pub far_alloc: IDAllocator<u32>,
	/// URR Allcoator
	pub urr_alloc: IDAllocator<u32>,
	/// QER Allcoator
	pub qer_alloc: IDAllocator<u32>,
	/// BAR Allcoator
	pub bar_alloc: IDAllocator<u8>,

	/// PDR Map
	pub pdr_map: HashMap<u16, CreatePDR>,
	/// FAR Map
	pub far_map: HashMap<u32, CreateFAR>,
	/// URR Map
	pub urr_map: HashMap<u32, CreateURR>,
	/// QER Map
	pub qer_map: HashMap<u32, CreateQER>,
	/// BAR Map
	pub bar_map: HashMap<u8, CreateBAR>,

	/// Predefined PDR rules
	pub active_predefined_rules: HashSet<String>,

	/// Update PDR Map
	pub update_pdr_map: HashMap<u16, Vec<UpdatePDR>>,
	/// Update FAR Map
	pub update_far_map: HashMap<u32, Vec<UpdateFAR>>,
	/// Update URR Map
	pub update_urr_map: HashMap<u32, Vec<UpdateURR>>,
	/// Update QER Map
	pub update_qer_map: HashMap<u32, Vec<UpdateQER>>,
	/// Update BAR Map
	pub update_bar_map: HashMap<u8, Vec<UpdateBAR>>,
}
impl PFCPSessionRules {
	pub fn new() -> PFCPSessionRules {
		PFCPSessionRules {
			pdr_alloc: IDAllocator::new(),
			far_alloc: IDAllocator::new_with_forbidden_bit(7),
			urr_alloc: IDAllocator::new_with_forbidden_bit(7),
			qer_alloc: IDAllocator::new_with_forbidden_bit(7),
			bar_alloc: IDAllocator::new(),
			pdr_map: HashMap::new(),
			far_map: HashMap::new(),
			urr_map: HashMap::new(),
			qer_map: HashMap::new(),
			bar_map: HashMap::new(),
			active_predefined_rules: HashSet::new(),
			update_pdr_map: HashMap::new(),
			update_far_map: HashMap::new(),
			update_urr_map: HashMap::new(),
			update_qer_map: HashMap::new(),
			update_bar_map: HashMap::new(),
		}
	}
	// pub fn activate_predefined_pdr(&mut self, name: &str) {

	// }
	// pub fn deactivate_predefined_pdr(&mut self, name: &str) {

	// }
	pub fn get_pdrs(&self) -> Vec<CreatePDR> {
		self.pdr_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_fars(&self) -> Vec<CreateFAR> {
		self.far_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_urrs(&self) -> Vec<CreateURR> {
		self.urr_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_qers(&self) -> Vec<CreateQER> {
		self.qer_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn get_bars(&self) -> Vec<CreateBAR> {
		self.bar_map.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
	}
	pub fn add_batch(&mut self, pdrs: &Vec<CreatePDR>, fars: &Vec<CreateFAR>, urrs: &Vec<CreateURR>, qers: &Vec<CreateQER>, bars: &Option<CreateBAR>) {
		for it in pdrs.iter() {
			self.pdr_map.insert(it.pdr_id.rule_id, it.clone());
		}
		for it in fars.iter() {
			self.far_map.insert(it.far_id.rule_id, it.clone());
		}
		for it in urrs.iter() {
			self.urr_map.insert(it.urr_id.rule_id, it.clone());
		}
		for it in qers.iter() {
			self.qer_map.insert(it.qer_id.rule_id, it.clone());
		}
		if let Some(it) = bars {
			self.bar_map.insert(it.bar_id.rule_id, it.clone());
		}
	}
	pub fn delete_batch(&mut self, pdrs: &Vec<PDR_ID>, fars: &Vec<FAR_ID>, urrs: &Vec<URR_ID>, qers: &Vec<QER_ID>, bar: &Option<BAR_ID>) {
		for it in pdrs.iter() {
			self.pdr_map.remove(&it.rule_id);
		}
		for it in fars.iter() {
			self.far_map.remove(&it.rule_id);
		}
		for it in urrs.iter() {
			self.urr_map.remove(&it.rule_id);
		}
		for it in qers.iter() {
			self.qer_map.remove(&it.rule_id);
		}
		if let Some(it) = bar {
			self.bar_map.remove(&it.rule_id);
		}
	}
	pub fn create_far(&mut self,
		apply_action: ApplyAction,
		forwarding_parameters: Option<ForwardingParameters>,
		bar_id: Option<BAR_ID>,
		redundant_transmission_forwarding_parameters: Option<RedundantTransmissionForwardingParameters>,
	) -> Result<FAR_ID, PFCPError> {
		let far_id = self.far_alloc.allocate()?;
		let far = CreateFAR {
			far_id: FAR_ID { rule_id: far_id },
		    apply_action: apply_action,
		    forwarding_parameters: forwarding_parameters,
		    bar_id: bar_id,
		    redundant_transmission_forwarding_parameters: redundant_transmission_forwarding_parameters,
		};
		self.far_map.insert(far_id, far);
		Ok(FAR_ID { rule_id: far_id })
	}
	pub fn create_pdr(&mut self,
		precedence: Precedence,
		pdi: PDI,
		outer_header_removal: Option<OuterHeaderRemoval>,
		far_id: Option<FAR_ID>,
		urr_id: Vec<URR_ID>,
		qer_id: Option<QER_ID>,
		activate_predefined_rules: Vec<ActivatePredefinedRules>,
		activation_time: Option<ActivationTime>,
		deactivation_time: Option<DeactivationTime>,
		mar_id: Option<MAR_ID>,
		packet_replication_and_detection_carry_on_information: Option<PacketReplicationAndDetectionCarryOnInformation>,
		ip_multicast_addressing_info: Vec<IPMulticastAddressingInfo>,
		ue_ip_address_pool_identity: Option<UE_IPAddressPoolIdentity>,
		mptcp_applicable_indication: Option<MPTCPApplicableIndication>,
		transport_delay_reporting: Option<TransportDelayReporting>,
	) -> Result<PDR_ID, PFCPError> {
		let pdr_id = self.pdr_alloc.allocate()?;
		let pdr = CreatePDR {
			pdr_id: PDR_ID { rule_id: pdr_id },
			precedence: precedence,
			pdi: pdi,
			outer_header_removal: outer_header_removal,
			far_id: far_id,
			urr_id: urr_id,
			qer_id: qer_id,
			activate_predefined_rules: activate_predefined_rules,
			//deactivate_predefined_rules: vec![],
			activation_time: activation_time,
			deactivation_time: deactivation_time,
			mar_id: mar_id,
			packet_replication_and_detection_carry_on_information: packet_replication_and_detection_carry_on_information,
			ip_multicast_addressing_info: ip_multicast_addressing_info,
			ue_ip_address_pool_identity: ue_ip_address_pool_identity,
			mptcp_applicable_indication: mptcp_applicable_indication,
			transport_delay_reporting: transport_delay_reporting,
		};
		self.pdr_map.insert(pdr_id, pdr);
		Ok(PDR_ID { rule_id: pdr_id })
	}
	pub fn create_qer(&mut self,
		qer_corrleation_id: Option<QERCorrelationID>,
		gate_status: GateStatus,
		maximum_bitrate: Option<MBR>,
		guaranteed_bitrate: Option<GBR>,
		// MAYBE TODO: Packet Rate Status
		qfi: Option<QFI>,
		rqi: Option<RQI>,
		paging_policy_indicator: Option<PagingPolicyIndicator>,
		averaging_window: Option<AveragingWindow>,
		qer_control_indications: Option<QERControlIndications>
	) -> Result<QER_ID, PFCPError> {
		let qer_id = self.qer_alloc.allocate()?;
		let qer = CreateQER {
			qer_id: QER_ID { rule_id: qer_id },
			qer_corrleation_id: qer_corrleation_id,
			gate_status: gate_status,
			maximum_bitrate: maximum_bitrate,
			guaranteed_bitrate: guaranteed_bitrate,
			qfi: qfi,
			rqi: rqi,
			paging_policy_indicator: paging_policy_indicator,
			averaging_window: averaging_window,
			qer_control_indications: qer_control_indications,
		};
		self.qer_map.insert(qer_id, qer);
		Ok(QER_ID { rule_id: qer_id })
	}
	pub fn create_urr(&mut self,
		measurement_method: MeasurementMethod,
		reporting_triggers: ReportingTriggers,
		measurement_period: Option<MeasurementPeriod>,
		volume_threshold: Option<VolumeThreshold>,
		volume_quota: Option<VolumeQuota>,
		event_threshold: Option<EventThreshold>,
		event_quota: Option<EventQuota>,
		time_threshold: Option<TimeThreshold>,
		time_quota: Option<TimeQuota>,
		quota_holding_time: Option<QuotaHoldingTime>,
		dropped_dl_traffic_threshold: Option<DroppedDLTrafficThreshold>,
		quota_validity_time: Option<QuotaValidityTime>,
		monitoring_time: Option<MonitoringTime>,
		subsequent_volume_threshold: Option<SubsequentVolumeThreshold>,
		subsequent_time_threshold: Option<SubsequentTimeThreshold>,
		subsequent_volume_quota: Option<SubsequentVolumeQuota>,
		subsequent_time_quota: Option<SubsequentTimeQuota>,
		subsequent_event_threshold: Option<SubsequentEventThreshold>,
		subsequent_event_quota: Option<SubsequentEventQuota>,
		inactivity_detection_time: Option<InactivityDetectionTime>,
		linked_urr_id: Vec<LinkedURR_ID>,
		measurement_information: Option<MeasurementInformation>,
		far_id_for_quota_action: Option<FAR_ID>,
		ethernet_inactivity_timer: Option<EthernetInactivityTimer>,
		additional_monitoring_time: Vec<AdditionalMonitoringTime>,
		number_of_reports: Option<NumberOfReports>,
		exempted_application_id_for_quota_action: Vec<ApplicationID>,
		exempted_sdf_filter_for_quota_action: Vec<SDFFilter>
	) -> Result<URR_ID, PFCPError> {
		let urr_id = self.urr_alloc.allocate()?;
		let urr = CreateURR {
			urr_id: URR_ID { rule_id: urr_id },
			measurement_method,
			reporting_triggers,
			measurement_period,
			volume_threshold,
			volume_quota,
			event_threshold,
			event_quota,
			time_threshold,
			time_quota,
			quota_holding_time,
			dropped_dl_traffic_threshold,
			quota_validity_time,
			monitoring_time,
			subsequent_volume_threshold,
			subsequent_time_threshold,
			subsequent_volume_quota,
			subsequent_time_quota,
			subsequent_event_threshold,
			subsequent_event_quota,
			inactivity_detection_time,
			linked_urr_id,
			measurement_information,
			far_id_for_quota_action,
			ethernet_inactivity_timer,
			additional_monitoring_time,
			number_of_reports,
			exempted_application_id_for_quota_action,
			exempted_sdf_filter_for_quota_action,
		};
		self.urr_map.insert(urr_id, urr);
		Ok(URR_ID { rule_id: urr_id })
	}
	pub fn update_far(&mut self,
		far_id: FAR_ID,
		apply_action: Option<ApplyAction>,
		update_forwarding_parameters: Option<UpdateForwardingParameters>,
		bar_id: Option<BAR_ID>,
		redundant_transmission_forwarding_parameters: Option<RedundantTransmissionForwardingParameters>,
	) {
		if let Some(far) = self.far_map.get_mut(&far_id.rule_id) {
			apply_action.as_ref().map(|o| far.apply_action = o.clone());
			bar_id.as_ref().map(|o| far.bar_id.replace(o.clone()));
			redundant_transmission_forwarding_parameters.as_ref().map(|o| far.redundant_transmission_forwarding_parameters.replace(o.clone()));
			update_forwarding_parameters.as_ref().map(|ufp| {
				if let Some(fp) = far.forwarding_parameters.as_mut() {
					ufp.destination_interface.as_ref().map(|o| fp.destination_interface = o.clone());
					ufp.network_instnace.as_ref().map(|o| fp.network_instnace.replace(o.clone()));
					ufp.redirect_information.as_ref().map(|o| fp.redirect_information.replace(o.clone()));
					ufp.outer_header_creation.as_ref().map(|o| fp.outer_header_creation.replace(o.clone()));
					ufp.transport_level_marking.as_ref().map(|o| fp.transport_level_marking.replace(o.clone()));
					ufp.forwarding_policy.as_ref().map(|o| fp.forwarding_policy.replace(o.clone()));
					ufp.header_enrichment.as_ref().map(|o| fp.header_enrichment.replace(o.clone()));
					ufp.linked_traffic_endpoint_id.as_ref().map(|o| fp.linked_traffic_endpoint_id.replace(o.clone()));
					ufp.destination_interface_type.as_ref().map(|o| fp.destination_interface_type.replace(o.clone()));
				} else {
					let fp = ForwardingParameters {
						destination_interface: ufp.destination_interface.as_ref().map_or(DestinationInterface::CoreSide, |f| f.clone()),
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
						data_network_access_identifier: ufp.data_network_access_identifier.clone(),
					};
					far.forwarding_parameters.replace(fp);
				}
			});
			let udapte_far = UpdateFAR {
			    far_id: far_id,
			    apply_action: apply_action,
			    update_forwarding_parameters: update_forwarding_parameters,
			    bar_id: bar_id,
			    redundant_transmission_forwarding_parameters: redundant_transmission_forwarding_parameters,
			};
			if let Some(update_history) = self.update_far_map.get_mut(&far_id.rule_id) {
				update_history.push(udapte_far);
			} else {
				self.update_far_map.insert(far_id.rule_id, vec![udapte_far]);
			}
		}
	}
	pub fn update_pdr(&mut self,
		pdr_id: PDR_ID,
		precedence: Option<Precedence>,
		pdi: Option<PDI>,
		outer_header_removal: Option<OuterHeaderRemoval>,
		far_id: Option<FAR_ID>,
		urr_id: Vec<URR_ID>,
		qer_id: Option<QER_ID>,
		activate_predefined_rules: Vec<ActivatePredefinedRules>,
		deactivate_predefined_rules: Vec<DeactivatePredefinedRules>,
		activation_time: Option<ActivationTime>,
		deactivation_time: Option<DeactivationTime>,
		ip_multicast_addressing_info: Vec<IPMulticastAddressingInfo>,
		transport_delay_reporting: Option<TransportDelayReporting>,
	) {
		if let Some(pdr) = self.pdr_map.get_mut(&pdr_id.rule_id) {
			precedence.as_ref().map(|o| pdr.precedence = o.clone());
			pdi.as_ref().map(|o| pdr.pdi = o.clone());
			outer_header_removal.as_ref().map(|o| pdr.outer_header_removal.replace(o.clone()));
			far_id.as_ref().map(|o| pdr.far_id.replace(o.clone()));
			if urr_id.len() != 0 { pdr.urr_id = urr_id.clone() }
			qer_id.as_ref().map(|o| pdr.qer_id.replace(o.clone()));
			if activate_predefined_rules.len() != 0 { pdr.activate_predefined_rules = activate_predefined_rules.clone() }
			//if deactivate_predefined_rules.len() != 0 { pdr.deactivate_predefined_rules = deactivate_predefined_rules.clone() }
			activation_time.as_ref().map(|o| pdr.activation_time.replace(o.clone()));
			deactivation_time.as_ref().map(|o| pdr.deactivation_time.replace(o.clone()));
			if ip_multicast_addressing_info.len() != 0 { pdr.ip_multicast_addressing_info = ip_multicast_addressing_info.clone() }
			transport_delay_reporting.as_ref().map(|o| pdr.transport_delay_reporting.replace(o.clone()));
			let update_pdr = UpdatePDR {
			    pdr_id: pdr_id,
			    precedence: precedence,
			    pdi: pdi,
			    outer_header_removal: outer_header_removal,
			    far_id: far_id,
			    urr_id: urr_id,
			    qer_id: qer_id,
			    activate_predefined_rules: activate_predefined_rules,
			    deactivate_predefined_rules: deactivate_predefined_rules,
			    activation_time: activation_time,
			    deactivation_time: deactivation_time,
			    ip_multicast_addressing_info: ip_multicast_addressing_info,
			    transport_delay_reporting: transport_delay_reporting,
			};
			if let Some(update_history) = self.update_pdr_map.get_mut(&pdr_id.rule_id) {
				update_history.push(update_pdr);
			} else {
				self.update_pdr_map.insert(pdr_id.rule_id, vec![update_pdr]);
			}
		}
	}
	pub fn remove_pdr(&mut self, pdr_id: PDR_ID) {
		self.pdr_map.remove(&pdr_id.rule_id);
		self.update_pdr_map.remove(&pdr_id.rule_id);
		self.pdr_alloc.free(pdr_id.rule_id);
	}
	pub fn remove_far(&mut self, far_id: FAR_ID) {
		self.far_map.remove(&far_id.rule_id);
		self.far_alloc.free(far_id.rule_id);
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PFCPSessionState {
	Creating,
	Activated,
	Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PFCPSessionTransactionState {
	Commited,
	InProgress
}

/// Represnting a PFCP Session in CP
#[derive(Debug)]
pub struct PFCPSessionCP {
	pub local_f_seid: F_SEID,
	pub remote_ip: std::net::IpAddr,
	pub remote_seid: Option<u64>,

	/// Sequence number for all messages, shares with parent PFCPNodeManager/UPF's sequence counter
	pub seq: Arc<sync::Mutex<u32>>,
	/// Make sure only one PFCP request is on going
	pub request_lock: sync::Mutex<i32>,
	/// State
	pub state: PFCPSessionState,
	/// Transaction State
	pub transaction_state: PFCPSessionTransactionState,
	/// SEID Allocator (used for deletion in CP)
	pub seid_alloc: Option<Arc<sync::Mutex<IDAllocator<u64>>>>,

	pub active_rules: PFCPSessionRules,
	pub last_checkpoint: Option<PFCPSessionRules>,
	pub dst_port_override: Option<u16>
}
impl PFCPSessionCP {
	/// Release this session and get session report, if you just want to release this session call drop() on me
	pub async fn release_with_report(&mut self) {
		let request = PFCPSessionDeletionRequest {};
		let (_header, _response_body) = match self.SendSessionDeletionRequest(request.encode()).await {
			Ok(a) => a,
			Err(_e) => {
				println!("Timeout sending SendSessionDeletionRequest");
				return;
			}
		};
		self.state = PFCPSessionState::Deleted;
		if let Some(alloc) = self.seid_alloc.as_mut() {
			alloc.lock().await.free(self.local_f_seid.seid)
		}
		// TODO: handle report after session release
	}
	pub fn transaction_begin(&mut self) -> PFCPSessionRules {
		assert_eq!(self.transaction_state, PFCPSessionTransactionState::Commited);
		self.transaction_state = PFCPSessionTransactionState::InProgress;
		let mut ret = self.active_rules.clone();
		ret.pdr_alloc.transaction_begin();
		ret.far_alloc.transaction_begin();
		ret.urr_alloc.transaction_begin();
		ret.qer_alloc.transaction_begin();
		ret.bar_alloc.transaction_begin();
		ret
	}
	pub async fn transaction_commit(&mut self, rules: PFCPSessionRules) -> Result<(Vec<CreatedPDR>, Option<AgoUpfPerfReport>), ()> {
		self.transaction_state = PFCPSessionTransactionState::Commited;
		// step 1: find diff (added, deleted, modified)
		let mut added_pdr = vec![];
		let mut modified_pdr = vec![];
		let mut deleted_pdr = vec![];
		for (k, _) in self.active_rules.pdr_map.iter() {
			if let Some(_) = rules.pdr_map.get(k) {
				if let Some(updates) = rules.update_pdr_map.get(k) {
					let mut final_update = updates[0].clone();
					for update in updates[1..].iter() {
						final_update.assign_new_update(update);
					}
					modified_pdr.push(final_update);
				}
			} else {
				deleted_pdr.push(*k);
			}
		}
		for (k, v) in rules.pdr_map.iter() {
			if let None = self.active_rules.pdr_map.get(k) {
				added_pdr.push(v.clone());
			}
		}
		let mut added_far = vec![];
		let mut modified_far = vec![];
		let mut deleted_far = vec![];
		for (k, _) in self.active_rules.far_map.iter() {
			if let Some(_) = rules.far_map.get(k) {
				if let Some(updates) = rules.update_far_map.get(k) {
					let mut final_update = updates[0].clone();
					for update in updates[1..].iter() {
						final_update.assign_new_update(update);
					}
					modified_far.push(final_update);
				}
			} else {
				deleted_far.push(*k);
			}
		}
		for (k, v) in rules.far_map.iter() {
			if let None = self.active_rules.far_map.get(k) {
				added_far.push(v.clone());
			}
		}
		let mut added_urr = vec![];
		let mut modified_urr = vec![];
		let mut deleted_urr = vec![];
		for (k, _) in self.active_rules.urr_map.iter() {
			if let Some(_) = rules.urr_map.get(k) {
				if let Some(updates) = rules.update_urr_map.get(k) {
					let mut final_update = updates[0].clone();
					for update in updates[1..].iter() {
						final_update.assign_new_update(update);
					}
					modified_urr.push(final_update);
				}
			} else {
				deleted_urr.push(*k);
			}
		}
		for (k, v) in rules.urr_map.iter() {
			if let None = self.active_rules.urr_map.get(k) {
				added_urr.push(v.clone());
			}
		}
		let mut added_qer = vec![];
		let mut modified_qer = vec![];
		let mut deleted_qer = vec![];
		for (k, _) in self.active_rules.qer_map.iter() {
			if let Some(_) = rules.qer_map.get(k) {
				if let Some(updates) = rules.update_qer_map.get(k) {
					let mut final_update = updates[0].clone();
					for update in updates[1..].iter() {
						final_update.assign_new_update(update);
					}
					modified_qer.push(final_update);
				}
			} else {
				deleted_qer.push(*k);
			}
		}
		for (k, v) in rules.qer_map.iter() {
			if let None = self.active_rules.qer_map.get(k) {
				added_qer.push(v.clone());
			}
		}
		let mut added_bar = None;
		let mut modified_bar = None;
		let mut deleted_bar = None;
		for (k, _) in self.active_rules.bar_map.iter() {
			if let Some(_) = rules.bar_map.get(k) {
				if let Some(updates) = rules.update_bar_map.get(k) {
					let mut final_update = updates[0].clone();
					for update in updates[1..].iter() {
						final_update.assign_new_update(update);
					}
					modified_bar = Some(final_update);
				}
			} else {
				deleted_bar.replace(*k);
				break;
			}
		}
		for (k, v) in rules.bar_map.iter() {
			if let None = self.active_rules.bar_map.get(k) {
				added_bar.replace(v.clone());
				break;
			}
		}
		// step 2: create modification request
		let request = PFCPSessionModificationRequest {
		    cp_f_seid: None,
		    remove_pdr: deleted_pdr.iter().map(|o| RemovePDR { pdr_id: PDR_ID { rule_id: *o } } ).collect::<Vec<_>>(),
		    remove_far: deleted_far.iter().map(|o| RemoveFAR { far_id: FAR_ID { rule_id: *o } } ).collect::<Vec<_>>(),
		    remove_urr: deleted_urr.iter().map(|o| RemoveURR { urr_id: URR_ID { rule_id: *o } } ).collect::<Vec<_>>(),
		    remove_qer: deleted_qer.iter().map(|o| RemoveQER { qer_id: QER_ID { rule_id: *o } } ).collect::<Vec<_>>(),
		    remove_bar: deleted_bar.as_ref().map(|o| RemoveBAR { bar_id: BAR_ID { rule_id: *o } } ),
		    remove_traffic_endpoint: None,
		    create_pdr: added_pdr,
		    create_far: added_far,
		    create_urr: added_urr,
		    create_qer: added_qer,
		    create_bar: added_bar,
		    create_traffic_endpoint: None,
		    update_pdr: modified_pdr,
		    update_far: modified_far,
		    update_urr: modified_urr,
		    update_qer: modified_qer,
		    update_bar: modified_bar,
		    update_traffic_endpoint: None,
		};
		// step 3: send and wait
		let (_header, response_body) = match self.SendSessionModificationRequest(request.encode()).await {
			Ok(a) => a,
			Err(_e) => {
				println!("Timeout sending SessionModificationRequest");
				return Err(());
			}
		};

		let response = match PFCPSessionModificationResponse::decode(response_body.clone().as_mut_slice()) {
			Ok(a) => a,
			Err(e) => { 
				println!("failed to decode response, {}", e);
				return Err(());
			}
		};
		// step 4: update current active_rules using response and input rules
		let mut new_rules = rules.clone();
		new_rules.pdr_alloc.transaction_commit();
		new_rules.far_alloc.transaction_commit();
		new_rules.urr_alloc.transaction_commit();
		new_rules.qer_alloc.transaction_commit();
		new_rules.bar_alloc.transaction_commit();
		// for (_, v) in new_rules.pdr_map.iter_mut() { v.apply_update(); }
		// for (_, v) in new_rules.far_map.iter_mut() { v.apply_update(); }
		// for (_, v) in new_rules.urr_map.iter_mut() { v.apply_update(); }
		// for (_, v) in new_rules.qer_map.iter_mut() { v.apply_update(); }
		// for (_, v) in new_rules.bar_map.iter_mut() { v.apply_update(); }
		// write response
		for resp_pdr in response.created_pdr.iter() {
			if let Some(pdr) = new_rules.pdr_map.get_mut(&resp_pdr.pdr_id.rule_id) {
				pdr.pdi.local_f_teid = resp_pdr.local_f_teid.clone();
			}
		}
		self.active_rules = new_rules;
		Ok((response.created_pdr, response.ago_perf))
	}
	pub fn checkpoint_create(&mut self) {
		// replace last checkpoint
		self.last_checkpoint.replace(self.active_rules.clone());
	}
	pub fn checkpoint_restore_last(&mut self) {

	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ResponseMatchingTuple {
	pub remote_ip: std::net::IpAddr,
	pub seq: u32
}

pub struct PFCPNodeGlobalContext {
	pub ongoing_requests: dashmap::DashMap<ResponseMatchingTuple, Arc<std::sync::RwLock<PFCPRequestFutureSharedState>>>,
	pub receive_seq_counters: dashmap::DashMap<std::net::IpAddr, u32>
}

impl PFCPNodeGlobalContext {
	pub fn new() -> PFCPNodeGlobalContext {
		PFCPNodeGlobalContext {
			ongoing_requests: dashmap::DashMap::new(),
			receive_seq_counters: dashmap::DashMap::new(),
		}
	}
}

pub struct PFCPRequestFutureSharedState {
	pub response: Option<(PFCPHeader, Vec<u8>)>,
	pub waker: Option<Waker>
}

pub struct PFCPRequestFuture {
	pub shared_state: Arc<std::sync::RwLock<PFCPRequestFutureSharedState>>,
}

impl Future for PFCPRequestFuture {
	type Output = (PFCPHeader, Vec<u8>);

	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		let mut guard = self.shared_state.write().unwrap();
		if guard.response.is_some() {
			Poll::Ready(guard.response.as_ref().unwrap().clone())
		} else {
			guard.waker = Some(cx.waker().clone());
			Poll::Pending
		}
	}
}


lazy_static! {
	pub static ref PFCP_NODE_GLOBAL_CONTEXT: PFCPNodeGlobalContext = PFCPNodeGlobalContext::new();
}

pub static PFCP_GLOBAL_REQUEST_SOCKET: OnceCell<UdpSocket> = OnceCell::new();

pub fn setup_global_pfcp_contexts<C: 'static>(handlers: C, request_recv_port_override: Option<u16>, async_runtime: tokio::runtime::Handle) -> Vec<JoinHandle<()>> where C: SessionRequestHandlers + NodeRequestHandlers + Send + Sync + Clone {
	let send_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	let send_socket_recv_thread = send_socket.try_clone().unwrap();
	PFCP_GLOBAL_REQUEST_SOCKET.set(send_socket).unwrap();
	handlers::create_handler_thread(handlers, send_socket_recv_thread, request_recv_port_override, async_runtime)
}
