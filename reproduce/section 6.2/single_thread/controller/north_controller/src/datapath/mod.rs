
use async_trait::async_trait;

pub mod tofino;

use libpfcp::{messages::{CreateFAR, CreatePDR, AdditionalMonitoringTime, ForwardingParameters, PDI, UpdateFAR, UpdatePDR, UpdateQER}, models::{ApplyAction, DroppedDLTrafficThreshold, FAR_ID, F_TEID, GBR, GateStatus, LinkedURR_ID, MBR, MeasurementInformation, MeasurementMethod, NumberOfReports, OuterHeaderRemoval, PDR_ID, PFCPSMReqFlags, Precedence, QER_ID, QFI, RQI, *, SourceInterface, SuggestedBufferingPacketsCount, VolumeQuota, VolumeThreshold}};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::routing::RoutingTable;

use self::tofino::URRTestAuxSettings;

#[derive(Clone, Debug)]
pub enum Report {
	BufferNoCP,
	DropNoCP,
	NoCP,
	DlArrival(PDR_ID),
	Error,
	UsageReports(Vec<libpfcp::messages::UsageReport>)
}

/// URR or packet arrival report or error report from backend
#[derive(Clone, Debug)]
pub struct BackendReport {
	pub report: Report,
	pub seid: u64,
}

pub struct BackendSettings {
	pub primary_config_file: String,
	pub auxiliary_config_file: Option<String>,
	pub target_addr: String,
	pub upf_ip: std::net::IpAddr,
	pub routing: RoutingTable,
	pub cpu_pcie_port: u16,
	pub ur_tx: Sender<BackendReport>,
	pub enable_univmon: bool,
	pub enable_qos: bool,
	pub enable_deferred_id_del: bool,
	pub urr_aux: URRTestAuxSettings,
	pub est_log_filename: String,
	pub qos_and_port_data: tofino::qos::QosAndPortData
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedFAR {
	pub apply_action: ApplyAction,
	pub forwarding_parameters: Option<ForwardingParameters>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedURR {
	pub urr_id: u32,
	pub measurement_method: MeasurementMethod,
	pub reporting_triggers: ReportingTriggers,
	pub measurement_period: Option<MeasurementPeriod>,
	pub volume_threshold: Option<VolumeThreshold>,
	pub volume_quota: Option<VolumeQuota>,
	pub event_threshold: Option<EventThreshold>,
	pub event_quota: Option<EventQuota>,
	pub time_threshold: Option<TimeThreshold>,
	pub time_quota: Option<TimeQuota>,
	pub quota_holding_time: Option<QuotaHoldingTime>,
	pub dropped_dl_traffic_threshold: Option<DroppedDLTrafficThreshold>,
	pub quota_validity_time: Option<QuotaValidityTime>,
	pub monitoring_time: Option<MonitoringTime>,
	pub subsequent_volume_threshold: Option<SubsequentVolumeThreshold>,
	pub subsequent_time_threshold: Option<SubsequentTimeThreshold>,
	pub subsequent_volume_quota: Option<SubsequentVolumeQuota>,
	pub subsequent_time_quota: Option<SubsequentTimeQuota>,
	pub subsequent_event_threshold: Option<SubsequentEventThreshold>,
	pub subsequent_event_quota: Option<SubsequentEventQuota>,
	pub inactivity_detection_time: Option<InactivityDetectionTime>,
	pub linked_urr_id: Vec<LinkedURR_ID>,
	pub measurement_information: Option<MeasurementInformation>,
	pub far_for_quota_action: Option<FlattenedFAR>,
	pub ethernet_inactivity_timer: Option<EthernetInactivityTimer>,
	pub additional_monitoring_time: Vec<AdditionalMonitoringTime>,
	pub number_of_reports: Option<NumberOfReports>,
	pub exempted_application_id_for_quota_action: Vec<ApplicationID>,
	pub exempted_sdf_filter_for_quota_action: Vec<SDFFilter>,
	pub enable_dwd: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlattenedPDI {
	pub source_interface: SourceInterface,
	pub local_f_teid: Option<F_TEID>,
	pub qfi: Option<u8>,
	/// if this exists, then we match IP proto other than TCP/UDP
	pub ip_proto: Option<u8>,
	pub tos: Option<(u8, u8)>,
	/// LSB 20 bits are used
	pub ipv6_flow_label: Option<[u8; 3]>,
	pub src_ipv4: Option<std::net::Ipv4Addr>,
	pub src_ipv4_mask: Option<u32>,
	pub src_ipv6: Option<std::net::Ipv6Addr>,
	pub src_ipv6_mask: Option<u128>,
	pub dst_ipv4: Option<std::net::Ipv4Addr>,
	pub dst_ipv4_mask: Option<u32>,
	pub dst_ipv6: Option<std::net::Ipv6Addr>,
	pub dst_ipv6_mask: Option<u128>,
	pub src_port_range: Option<(u16, u16)>,
	pub dst_port_range: Option<(u16, u16)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketPipeline {
	pub seid: u64,
	// includes content of PDR, QER, FAR and BAR

	// PDR
	pub pdr_id: PDR_ID,
	pub precedence: Precedence,
	pub pdi: PDI,
	pub outer_header_removal: Option<OuterHeaderRemoval>,

	// QER
	pub qer_id: Option<QER_ID>,
	pub gate_status: Option<GateStatus>,
	// pub maximum_bitrate: Option<MBR>,
	// pub guaranteed_bitrate: Option<GBR>,
	pub qfi: Option<QFI>,

	// FAR
	pub apply_action: ApplyAction,
	pub forwarding_parameters: Option<ForwardingParameters>,
	pub pfcpsm_req_flags: Option<PFCPSMReqFlags>,

	// URR
	pub urr_ids: Vec<URR_ID>,
	//pub urrs: Vec<FlattenedURR>,

	// BAR
	pub suggested_buffering_packets_count: Option<SuggestedBufferingPacketsCount>
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedPacketPipeline {
	pub seid: u64,
	// includes content of PDR, QER, FAR and BAR

	// PDR
	pub pdr_id: u16,
	/// Lower value higher priority, must be positive
	pub precedence: i32,
	pub pdi: FlattenedPDI,
	pub outer_header_removal: Option<OuterHeaderRemoval>,

	// QER
	pub qfi: Option<u8>,
	pub qer_id: Option<u32>,
	// pub qer_id: Option<u32>,
	pub gate_status: Option<GateStatus>,
	// pub maximum_bitrate: Option<MBR>,
	// pub guaranteed_bitrate: Option<GBR>,
	// pub qfi: Option<u8>,

	// FAR
	pub apply_action: ApplyAction,
	pub forwarding_parameters: Option<ForwardingParameters>,
	pub pfcpsm_req_flags: Option<PFCPSMReqFlags>,

	// URR
	pub urr_ids: Vec<u32>,
	//pub urrs: Vec<FlattenedURR>,

	// BAR
	pub suggested_buffering_packets_count: Option<u8>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlattenedQER {
	pub qer_id: u32,
	// pub gate_status: GateStatus,
	pub maximum_bitrate: Option<MBR>,
	pub guaranteed_bitrate: Option<GBR>,
}

impl FlattenedQER {
	pub fn is_meter_session(&self) -> bool {
		return self.maximum_bitrate.is_some() || self.guaranteed_bitrate.is_some()
	}
}

pub struct OperationPerfStats {
	pub stats1: u64,
	pub stats2: u64,
	pub stats3: u64,
	pub stats4: u64
}

#[async_trait]
pub trait BackendInterface {
	async fn new(settings: BackendSettings) -> Box<dyn BackendInterface + Sync + Send> where Self: Sized;
	async fn on_to_cpu_packet_received(&self, packet: &[u8]);
	async fn update_or_add_forwarding(&self, seid: u64, pipelines: Vec<PacketPipeline>) -> Result<OperationPerfStats, Box<dyn std::error::Error + Send + Sync>>;
	async fn delete_forwarding(&self, seid: u64, pipelines: Vec<PDR_ID>);
	async fn update_or_add_qer(&self, seid: u64, pipelines: Vec<FlattenedQER>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
	async fn delete_qer(&self, seid: u64, pipelines: Vec<QER_ID>);
	async fn update_or_add_urr(&self, seid: u64, pipelines: Vec<FlattenedURR>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
	async fn delete_urr(&self, seid: u64, pipelines: Vec<URR_ID>) -> Vec<Option<BackendReport>>;
	async fn delete_session(&self, seid: u64) -> (OperationPerfStats, Vec<BackendReport>);
	async fn release_all_sessions(&self);
	async fn reset_all(&self, settings: BackendSettings);
	async fn stop(&self);
}
