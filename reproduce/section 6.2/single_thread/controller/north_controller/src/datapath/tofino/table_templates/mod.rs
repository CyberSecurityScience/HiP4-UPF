
use libpfcp::models::{ApplyAction, SourceInterface, OuterHeaderRemovalDescription, DestinationInterface};
use log::{info, warn, error};

use crate::datapath::{FlattenedPacketPipeline, FlattenedURR};
use crate::datapath::tofino::bfruntime::bfrt::key_field::Exact;

use self::reorder::ReorderMatchActionInstance;

use super::match_action_id::ActionInstance;
use super::{SendEndMarkerRequest, TofinoBackendError, upf_driver_interface};
use super::{super::PacketPipeline};

use super::bfruntime::{bfrt::*, P4TableInfo};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PipelineTemplateDirection {
	Uplink,
	Downlink,
	None
}

pub trait PipelineTemplate {
	fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> where Self: Sized;
	fn validate(&self) -> Result<(), TofinoBackendError>;
	fn p4_table_name(&self) -> &'static str;
	fn generate_table_key(&self, info: &P4TableInfo) -> TableKey;
	fn priority(&self) -> i32;
	fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> TableData;
	fn get_p4_table_order(&self) -> i32;
	fn generate_upf_driver_update_remove(&self, old_ma_id: u32) -> upf_driver_interface::PDRUpdate;
	fn generate_upf_driver_update_update(&self, new_ma_id: u32, old_ma_id: u32, global_qer_id: u16) -> upf_driver_interface::PDRUpdate;
	fn generate_upf_driver_update_insert(&self, new_ma_id: u32, global_qer_id: u16) -> upf_driver_interface::PDRUpdate;
}

pub mod reorder;

pub mod do_nothing;
pub mod uplink_n6_simple;
pub mod uplink_n6_complex;
pub mod uplink_n9_simple;
pub mod uplink_n9_complex;
pub mod downlink_n6_simple;
pub mod downlink_n6_complex;
pub mod downlink_n9_simple;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MatchInstance {
	UplinkToN6Simple(uplink_n6_simple::UplinkToN6Simple),
	UplinkToN6Complex(uplink_n6_complex::UplinkToN6Complex),
	UplinkToN9Simple(uplink_n9_simple::UplinkToN9Simple),
	UplinkToN9Complex(uplink_n9_complex::UplinkToN9Complex),
	DownlinkFromN6Simple(downlink_n6_simple::DownlinkFromN6Simple),
	DownlinkFromN6Complex(downlink_n6_complex::DownlinkFromN6Complex),
	DownlinkFromN9Simple(downlink_n9_simple::DownlinkFromN9Simple),
	DoNothing(do_nothing::DoNothing)
}

impl MatchInstance {
	pub fn generate_upf_driver_update_remove(&self, old_ma_id: u32) -> Option<upf_driver_interface::PDRUpdate> {
		match self {
			MatchInstance::UplinkToN6Simple(x) => Some(x.generate_upf_driver_update_remove(old_ma_id)),
			MatchInstance::UplinkToN6Complex(_) => todo!(),
			MatchInstance::UplinkToN9Simple(_) => todo!(),
			MatchInstance::UplinkToN9Complex(_) => todo!(),
			MatchInstance::DownlinkFromN6Simple(x) => Some(x.generate_upf_driver_update_remove(old_ma_id)),
			MatchInstance::DownlinkFromN6Complex(_) => todo!(),
			MatchInstance::DownlinkFromN9Simple(_) => todo!(),
			MatchInstance::DoNothing(_) => None,
		}
	}
	pub fn generate_upf_driver_update_update(&self, new_ma_id: u32, old_ma_id: u32, global_qer_id: u16) -> Option<upf_driver_interface::PDRUpdate> {
		match self {
			MatchInstance::UplinkToN6Simple(x) => Some(x.generate_upf_driver_update_update(new_ma_id, old_ma_id, global_qer_id)),
			MatchInstance::UplinkToN6Complex(_) => todo!(),
			MatchInstance::UplinkToN9Simple(_) => todo!(),
			MatchInstance::UplinkToN9Complex(_) => todo!(),
			MatchInstance::DownlinkFromN6Simple(x) => Some(x.generate_upf_driver_update_update(new_ma_id, old_ma_id, global_qer_id)),
			MatchInstance::DownlinkFromN6Complex(_) => todo!(),
			MatchInstance::DownlinkFromN9Simple(_) => todo!(),
			MatchInstance::DoNothing(_) => None,
		}
	}
	pub fn generate_upf_driver_update_insert(&self, new_ma_id: u32, global_qer_id: u16) -> Option<upf_driver_interface::PDRUpdate> {
		match self {
			MatchInstance::UplinkToN6Simple(x) => Some(x.generate_upf_driver_update_insert(new_ma_id, global_qer_id)),
			MatchInstance::UplinkToN6Complex(_) => todo!(),
			MatchInstance::UplinkToN9Simple(_) => todo!(),
			MatchInstance::UplinkToN9Complex(_) => todo!(),
			MatchInstance::DownlinkFromN6Simple(x) => Some(x.generate_upf_driver_update_insert(new_ma_id, global_qer_id)),
			MatchInstance::DownlinkFromN6Complex(_) => todo!(),
			MatchInstance::DownlinkFromN9Simple(_) => todo!(),
			MatchInstance::DoNothing(_) => None,
		}
	}
	pub fn priority(&self) -> i32 {
		match self {
			MatchInstance::UplinkToN6Simple(x) => x.priority(),
			MatchInstance::UplinkToN6Complex(x) => x.priority(),
			MatchInstance::UplinkToN9Simple(x) => x.priority(),
			MatchInstance::UplinkToN9Complex(x) => x.priority(),
			MatchInstance::DownlinkFromN6Simple(x) => x.priority(),
			MatchInstance::DownlinkFromN6Complex(x) => x.priority(),
			MatchInstance::DownlinkFromN9Simple(x) => x.priority(),
			MatchInstance::DoNothing(_) => i32::MAX,
		}
	}
	pub fn generate_table_key(&self, info: &P4TableInfo) -> Option<TableKey> {
		// for 5G, lower value means higher precedence, we call precedence
		// for Tofino, higher value means higher precedence, we call priority
		match self {
			MatchInstance::UplinkToN6Simple(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::UplinkToN6Complex(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::UplinkToN9Simple(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::UplinkToN9Complex(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::DownlinkFromN6Simple(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::DownlinkFromN6Complex(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::DownlinkFromN9Simple(t) => {
				Some(t.generate_table_key(info))
			},
			MatchInstance::DoNothing(t) => {
				None
			}
		}
	}
	pub fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> Option<TableData> {
		match self {
			MatchInstance::UplinkToN6Simple(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::UplinkToN6Complex(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::UplinkToN9Simple(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::UplinkToN9Complex(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::DownlinkFromN6Simple(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::DownlinkFromN6Complex(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::DownlinkFromN9Simple(t) => {
				Some(t.generate_table_action_data(info, maid, qer_id))
			},
			MatchInstance::DoNothing(t) => {
				None
			}
		}
	}
	pub fn p4_table_name(&self) -> &'static str {
		match self {
			MatchInstance::UplinkToN6Simple(t) => {
				t.p4_table_name()
			},
			MatchInstance::UplinkToN6Complex(t) => {
				t.p4_table_name()
			},
			MatchInstance::UplinkToN9Simple(t) => {
				t.p4_table_name()
			},
			MatchInstance::UplinkToN9Complex(t) => {
				t.p4_table_name()
			},
			MatchInstance::DownlinkFromN6Simple(t) => {
				t.p4_table_name()
			},
			MatchInstance::DownlinkFromN6Complex(t) => {
				t.p4_table_name()
			},
			MatchInstance::DownlinkFromN9Simple(t) => {
				t.p4_table_name()
			},
			MatchInstance::DoNothing(t) => {
				"none"
			}
		}
	}
	pub fn get_direction(&self) -> PipelineTemplateDirection {
		match self {
			MatchInstance::UplinkToN6Simple(t) => PipelineTemplateDirection::Uplink,
			MatchInstance::UplinkToN6Complex(t) => PipelineTemplateDirection::Uplink,
			MatchInstance::UplinkToN9Simple(t) => PipelineTemplateDirection::Uplink,
			MatchInstance::UplinkToN9Complex(t) => PipelineTemplateDirection::Uplink,
			MatchInstance::DownlinkFromN6Simple(t) => PipelineTemplateDirection::Downlink,
			MatchInstance::DownlinkFromN6Complex(t) => PipelineTemplateDirection::Downlink,
			MatchInstance::DownlinkFromN9Simple(t) => PipelineTemplateDirection::Downlink,
			MatchInstance::DoNothing(_) => PipelineTemplateDirection::None,
		}
	}
	pub fn get_p4_table_order(&self) -> i32 {
		match self {
			MatchInstance::UplinkToN6Simple(t) => t.get_p4_table_order(),
			MatchInstance::UplinkToN6Complex(t) => t.get_p4_table_order(),
			MatchInstance::UplinkToN9Simple(t) => t.get_p4_table_order(),
			MatchInstance::UplinkToN9Complex(t) => t.get_p4_table_order(),
			MatchInstance::DownlinkFromN6Simple(t) => t.get_p4_table_order(),
			MatchInstance::DownlinkFromN6Complex(t) => t.get_p4_table_order(),
			MatchInstance::DownlinkFromN9Simple(t) => t.get_p4_table_order(),
			MatchInstance::DoNothing(t) => t.get_p4_table_order(),
		}
	}
	pub fn to_p4_table_order(&self, order: i32) -> MatchInstance {
		match self {
			MatchInstance::UplinkToN6Simple(t) => t.to_p4_table_order(order),
			MatchInstance::UplinkToN6Complex(t) => t.to_p4_table_order(order),
			MatchInstance::UplinkToN9Simple(t) => t.to_p4_table_order(order),
			MatchInstance::UplinkToN9Complex(t) => t.to_p4_table_order(order),
			MatchInstance::DownlinkFromN6Simple(t) => t.to_p4_table_order(order),
			MatchInstance::DownlinkFromN6Complex(t) => t.to_p4_table_order(order),
			MatchInstance::DownlinkFromN9Simple(t) => t.to_p4_table_order(order),
			MatchInstance::DoNothing(t) => self.clone(),
		}
	}
	pub fn is_empty(&self) -> bool {
		match self {
			MatchInstance::UplinkToN6Simple(_) => false,
			MatchInstance::UplinkToN6Complex(_) => false,
			MatchInstance::UplinkToN9Simple(_) => false,
			MatchInstance::UplinkToN9Complex(_) => false,
			MatchInstance::DownlinkFromN6Simple(_) => false,
			MatchInstance::DownlinkFromN6Complex(_) => false,
			MatchInstance::DownlinkFromN9Simple(_) => false,
			MatchInstance::DoNothing(_) => true,
		}
	}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsageReportingKey {
	pub upf_self_id: u8,
	pub direction: u8,
	pub short_teid: u32
}

#[derive(Debug, Clone)]
pub struct UsageReportingContent {
	pub urrs: Vec<FlattenedURR>
}

#[derive(Debug, Clone)]
pub enum UsageReportingTemplate {
	NoReporting,
	ExactBytesAndPackets
}

pub fn match_pipeline(pipeline: &FlattenedPacketPipeline) -> Result<(MatchInstance, ActionInstance), TofinoBackendError> {
	// step 1: get direction
	let uplink = {
		match pipeline.pdi.source_interface {
			libpfcp::models::SourceInterface::AccessSide => true,
			libpfcp::models::SourceInterface::CoreSide => false,
			libpfcp::models::SourceInterface::SGi_LAN_N6_LAN => return Err(TofinoBackendError::UnsupportedSourceInterface),
			libpfcp::models::SourceInterface::CP_Function => return Err(TofinoBackendError::UnsupportedSourceInterface),
			libpfcp::models::SourceInterface::_5G_VN_internal => return Err(TofinoBackendError::UnsupportedSourceInterface),
		}
	};
	let pdr_id = pipeline.pdr_id;
	// step 2: match forwarding decision
	let (match_instance, action_instance) = if let Some((mut t, at)) = do_nothing::DoNothing::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::DoNothing(t), at)
	} else if let Some((mut t, at)) = uplink_n6_simple::UplinkToN6Simple::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::UplinkToN6Simple(t), at)
	} else if let Some((mut t, at)) = uplink_n6_complex::UplinkToN6Complex::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::UplinkToN6Complex(t), at)
	} else if let Some((mut t, at)) = uplink_n9_simple::UplinkToN9Simple::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::UplinkToN9Simple(t), at)
	} else if let Some((mut t, at)) = uplink_n9_complex::UplinkToN9Complex::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::UplinkToN9Complex(t), at)
	} else if let Some((mut t, at)) = downlink_n6_simple::DownlinkFromN6Simple::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::DownlinkFromN6Simple(t), at)
	} else if let Some((mut t, at)) = downlink_n6_complex::DownlinkFromN6Complex::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::DownlinkFromN6Complex(t), at)
	} else if let Some((mut t, at)) = downlink_n9_simple::DownlinkFromN9Simple::hit(pipeline, pdr_id)? {
		t.validate()?;
		(MatchInstance::DownlinkFromN9Simple(t), at)
	} else {
		error!("No matching template found");
		return Err(TofinoBackendError::Todo);
	};
	Ok((match_instance, action_instance))
}
