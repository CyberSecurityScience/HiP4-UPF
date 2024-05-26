
use libpfcp::models::{SourceInterface, OuterHeaderRemovalDescription, ApplyAction};

use crate::datapath::{FlattenedPacketPipeline, tofino::{bfruntime::{P4TableInfo, bfrt::{TableKey, TableEntry, key_field::{Exact, self}, KeyField, table_entry}}, match_action_id::ActionInstance}};

use super::{PipelineTemplate, super::TofinoBackendError};


#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoNothing {
}
impl DoNothing {
	pub fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> {
		let mut apply_action = ApplyAction(0);
		if pipeline.apply_action == apply_action {
			Ok(Some((Self {}, ActionInstance::Nop)))
		} else {
			apply_action.setFORW(1);
			if pipeline.apply_action == apply_action && pipeline.pdi.source_interface == SourceInterface::CoreSide && pipeline.forwarding_parameters.is_none() {
				return Ok(Some((Self {}, ActionInstance::Nop)));
			}
			Ok(None)
		}
	}
	pub fn validate(&self) -> Result<(), TofinoBackendError> {
		Ok(())
	}
	pub fn get_p4_table_order(&self) -> i32 {
		0
	}
}
