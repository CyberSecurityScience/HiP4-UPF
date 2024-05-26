use libpfcp::models::{SourceInterface, OuterHeaderRemovalDescription, DestinationInterface};

use crate::datapath::{FlattenedPacketPipeline, tofino::{bfruntime::{P4TableInfo, bfrt::{TableKey, TableEntry, key_field::{Exact, self}, KeyField, table_entry, TableData, DataField, data_field}}, match_action_id::ActionInstance}};

use super::{PipelineTemplate, super::TofinoBackendError, UsageReportingKey, reorder::ReorderMatchActionInstance};

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DownlinkFromN6Simple {
	pub key_ue_ip: std::net::Ipv4Addr,
	pub key_priority: i32,

	pub pdr_id: u16,
	pub teid: u32,
}

impl PipelineTemplate for DownlinkFromN6Simple {
	fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> {
		if pipeline.pdi.source_interface == SourceInterface::CoreSide {
			if pipeline.pdi.dst_ipv4.is_some() &&
				// pipeline.pdi.dst_ipv4_mask.is_none() &&
				pipeline.pdi.dst_ipv6.is_none() &&
				pipeline.pdi.dst_ipv6_mask.is_none() &&
				pipeline.pdi.dst_port_range.is_none() &&
				pipeline.pdi.ip_proto.is_none() &&
				pipeline.pdi.qfi.is_none() &&
				pipeline.pdi.tos.is_none() &&
				pipeline.pdi.src_ipv4.is_none() &&
				pipeline.pdi.src_ipv4_mask.is_none() &&
				pipeline.pdi.src_ipv6.is_none() &&
				pipeline.pdi.src_ipv6_mask.is_none() &&
				pipeline.pdi.ipv6_flow_label.is_none() &&
				pipeline.pdi.src_port_range.is_none() &&
				pipeline.pdi.dst_port_range.is_none() &&
				pipeline.qfi.is_some() &&
				pipeline.outer_header_removal.is_none() &&
				pipeline.forwarding_parameters.is_some()
			{
				if let Some(ip_mask) = &pipeline.pdi.dst_ipv4_mask {
					if *ip_mask != u32::MAX {
						return Ok(None);
					}
				}
				if let Some(forwarding_parameters) = &pipeline.forwarding_parameters {
					if forwarding_parameters.destination_interface != DestinationInterface::AccessSide {
						return Ok(None);
					}
					if forwarding_parameters.transport_level_marking.is_some() {
						return Ok(None);
					}
					if let Some(ohc) = &forwarding_parameters.outer_header_creation {
						let ue_ip = pipeline.pdi.dst_ipv4.unwrap();
						let nocp = pipeline.apply_action.getNOCP() == 1;
						let mut drop = pipeline.apply_action.getDROP() == 1;
						let mut buffer = pipeline.apply_action.getBUFF() == 1;
						if let Some(gate) = &pipeline.gate_status {
							if gate.getDLGate() == 1 {
								drop = true;
							}
						}
						let mut teid = 0;
						let action_tuple = if drop {
							ActionInstance::DropDl(nocp)
						} else if buffer {
							ActionInstance::Buffer(nocp)
						} else if pipeline.qfi.is_some() {
							if let Some(forwarding_parameters) = &pipeline.forwarding_parameters {
								if let Some(ohc) = &forwarding_parameters.outer_header_creation {
									if ohc.ipv4.is_some() && ohc.teid.is_some() && ohc.desc.getGTP_U_UDP_IPv4() == 1 && ohc.ipv4.is_some() {
										teid = ohc.teid.unwrap();
										ActionInstance::EncapDl(nocp, ohc.ipv4.unwrap(), pipeline.qfi.unwrap())
									} else {
										return Ok(None);
									}
								} else {
									return Ok(None);
								}
							} else {
								return Ok(None);
							}
						} else {
							return Ok(None);
						};
						Ok(Some(
							(
								Self {
									key_ue_ip: ue_ip,
									key_priority: i32::MAX - pipeline.precedence,

									pdr_id,
									teid
								},
								action_tuple
							)
						))
					} else {
						Ok(None)
					}
				} else {
					Ok(None)
				}
			} else {
				Ok(None)
			}
		} else {
			Ok(None)
		}
	}
	fn validate(&self) -> Result<(), TofinoBackendError> {
		Ok(())
	}
	fn priority(&self) -> i32 {
		self.key_priority
	}
	fn generate_table_key(&self, info: &P4TableInfo) -> TableKey {
		TableKey {
			fields: vec![
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.dl_N6_simple_ipv4", "hdr.overlay_ipv4.dstAddr"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: self.key_ue_ip.octets().to_vec()
					}))
				}
			]
		}
	}
	fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> TableData {
		TableData {
			action_id: info.get_action_id_by_name("pipe.Ingress.pdr.dl_N6_simple_ipv4", "Ingress.pdr.set_ma_id_and_tunnel_dl_N6_simple_ipv4"),
			fields: vec![
				DataField {
					field_id: 1,
					value: Some(data_field::Value::Stream(maid.to_be_bytes()[1..].to_vec()))
				},
				DataField {
					field_id: 2,
					value: Some(data_field::Value::Stream(self.teid.to_be_bytes().to_vec()))
				},
				DataField {
					field_id: 3,
					value: Some(data_field::Value::Stream(qer_id.to_be_bytes().to_vec()))
				}
			]
		}
	}
	fn p4_table_name(&self) -> &'static str {
		"pipe.Ingress.pdr.dl_N6_simple_ipv4"
	}
	fn get_p4_table_order(&self) -> i32 {
		3
	}
	fn generate_upf_driver_update_remove(&self, old_ma_id: u32) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        crate::datapath::tofino::upf_driver_interface::PDRUpdate::DL_N6_SimpleRemove(
			super::super::requests_generated::upfdriver::requests::DL_N6_SimpleRemoveRequestArgs {
				key_ipv4: u32::from_be_bytes(self.key_ue_ip.octets()),
				data_old_ma_id: old_ma_id,
			}
		)
    }
	fn generate_upf_driver_update_update(&self, new_ma_id: u32, old_ma_id: u32, global_qer_id: u16) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        crate::datapath::tofino::upf_driver_interface::PDRUpdate::DL_N6_SimpleUpdate(
			super::super::requests_generated::upfdriver::requests::DL_N6_SimpleUpdateRequestArgs {
				key_ipv4: u32::from_be_bytes(self.key_ue_ip.octets()),
				data_ma_id: new_ma_id,
				data_old_ma_id: old_ma_id,
				data_teid: self.teid,
				data_qer_id: global_qer_id,
				vol_thres: 0
			}
		)
    }
	fn generate_upf_driver_update_insert(&self, new_ma_id: u32, global_qer_id: u16) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        crate::datapath::tofino::upf_driver_interface::PDRUpdate::DL_N6_SimpleInsert(
			super::super::requests_generated::upfdriver::requests::DL_N6_SimpleInsertRequestArgs {
				key_ipv4: u32::from_be_bytes(self.key_ue_ip.octets()),
				data_ma_id: new_ma_id,
				data_teid: self.teid,
				data_qer_id: global_qer_id,
				vol_thres: 0
			}
		)
    }
}

impl ReorderMatchActionInstance for DownlinkFromN6Simple {
	fn to_p4_table_order(&self, order: i32) -> super::MatchInstance {
		let new_fow = super::downlink_n6_complex::DownlinkFromN6Complex {
			key_dst_ip: self.key_ue_ip,
			key_dst_ip_mask: u32::MAX,
			key_src_ip: std::net::Ipv4Addr::new(0, 0, 0, 0),
			key_src_ip_mask: 0,
			key_ip_proto: 0,
			key_ip_proto_mask: 0,
			key_tos: 0,
			key_tos_mask: 0,
			key_src_port_range: (u16::MIN, u16::MAX),
			key_dst_port_range: (u16::MIN, u16::MAX),
			key_priority: self.key_priority,

			pdr_id: self.pdr_id,
			teid: self.teid
		};
		super::MatchInstance::DownlinkFromN6Complex(new_fow)
	}
}


