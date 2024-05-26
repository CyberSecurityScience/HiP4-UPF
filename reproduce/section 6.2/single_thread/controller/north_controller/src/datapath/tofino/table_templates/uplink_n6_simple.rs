use libpfcp::models::{SourceInterface, OuterHeaderRemovalDescription, DestinationInterface};

use crate::datapath::{FlattenedPacketPipeline, tofino::{bfruntime::{P4TableInfo, bfrt::{TableKey, TableEntry, key_field::{Exact, self}, KeyField, table_entry, DataField, TableData, data_field}}, match_action_id::ActionInstance, upf_driver_interface}};

use super::{PipelineTemplate, super::TofinoBackendError, reorder::ReorderMatchActionInstance};


#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UplinkToN6Simple {
	pub key_qfi: u8,
	pub key_teid: u32,
	pub key_priority: i32,

	pub pdr_id: u16,
}

impl PipelineTemplate for UplinkToN6Simple {
	fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> {
		if pipeline.pdi.source_interface == SourceInterface::AccessSide {
			if pipeline.pdi.dst_ipv4.is_none() &&
				pipeline.pdi.dst_ipv4_mask.is_none() &&
				pipeline.pdi.dst_ipv6.is_none() &&
				pipeline.pdi.dst_ipv6_mask.is_none() &&
				pipeline.pdi.ip_proto.is_none() &&
				pipeline.pdi.qfi.is_some() &&
				pipeline.pdi.tos.is_none() &&
			//    pipeline.pdi.src_ipv4.is_none() &&
			//    pipeline.pdi.src_ipv4_mask.is_none() &&
				pipeline.pdi.src_ipv6.is_none() &&
				pipeline.pdi.src_ipv6_mask.is_none() &&
				pipeline.pdi.ipv6_flow_label.is_none() &&
				pipeline.pdi.src_port_range.is_none() &&
				pipeline.pdi.dst_port_range.is_none()
			//    pipeline.qfi.is_none() &&
			//    pipeline.forwarding_parameters.is_none()
			{
				let mut mark_tos = None;
				if let Some(ip_mask) = &pipeline.pdi.src_ipv4_mask {
					if *ip_mask != u32::MAX {
						return Ok(None);
					}
				}
				if let Some(fow) = &pipeline.forwarding_parameters {
					if fow.outer_header_creation.is_some() {
						return Ok(None);
					}
					if fow.destination_interface != DestinationInterface::CoreSide {
						return Ok(None);
					}
					if let Some(mt) = &fow.transport_level_marking {
						mark_tos = Some(mt.to_tos());
					}
				}
				if let (Some(f_teid), Some(ohr)) = (&pipeline.pdi.local_f_teid, &pipeline.outer_header_removal) {
					let a = ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IPv4 || ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IPv6 || ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IP;
					let b = f_teid.choose_id.is_none() && (f_teid.ipv4.is_some() || f_teid.ipv6.is_some()) && f_teid.teid.is_some();
					let r = a & b;
					if r {
						let nocp = pipeline.apply_action.getNOCP() == 1;
						let mut drop = pipeline.apply_action.getDROP() == 1;
						let mut buffer = pipeline.apply_action.getBUFF() == 1;
						if buffer {
							return Ok(None);
						}
						if let Some(gate) = &pipeline.gate_status {
							if gate.getULGate() == 1 {
								drop = true;
							}
						}
						let action_tuple = if drop {
							ActionInstance::DropUl(nocp)
						} else if mark_tos.is_some() {
							ActionInstance::DecapMarkDSCP(nocp, mark_tos.unwrap())
						} else {
							ActionInstance::Decap(nocp)
						};
						Ok(Some(
							(
								UplinkToN6Simple {
									key_teid: f_teid.teid.unwrap(),
									key_qfi: pipeline.pdi.qfi.unwrap(),
									key_priority: i32::MAX - pipeline.precedence,
									
									pdr_id
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
		if self.key_teid & 0xff000000u32 != 0 {
			return Err(TofinoBackendError::TeidOutOfRange);
		}
		Ok(())
	}
	fn priority(&self) -> i32 {
		self.key_priority
	}
	fn generate_table_key(&self, info: &P4TableInfo) -> TableKey {
		TableKey {
			fields: vec![
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_simple_ipv4", "hdr.gtpu.teid[23:0]"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: self.key_teid.to_be_bytes()[1..].to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_simple_ipv4", "hdr.gtpu_ext_psc.qfi"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: vec![self.key_qfi]
					}))
				}
			]
		}
	}
	fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> TableData {
		TableData {
			action_id: info.get_action_id_by_name("pipe.Ingress.pdr.ul_N6_simple_ipv4", "Ingress.pdr.set_ma_id_ul_N6_simple_ipv4"),
			fields: vec![
				DataField {
					field_id: 1,
					value: Some(data_field::Value::Stream(maid.to_be_bytes()[1..].to_vec()))
				},
				DataField {
					field_id: 2,
					value: Some(data_field::Value::Stream(qer_id.to_be_bytes().to_vec()))
				}
			]
		}
	}
	fn p4_table_name(&self) -> &'static str {
		"pipe.Ingress.pdr.ul_N6_simple_ipv4"
	}
	fn get_p4_table_order(&self) -> i32 {
		3
	}
	fn generate_upf_driver_update_remove(&self, old_ma_id: u32) -> upf_driver_interface::PDRUpdate {
		upf_driver_interface::PDRUpdate::UL_N6_SimpleRemove(
			super::super::requests_generated::upfdriver::requests::UL_N6_SimpleRemoveRequestArgs {
				key_teid: self.key_teid,
				key_qfi: self.key_qfi,
				data_old_ma_id: old_ma_id,
			}
		)
	}
	fn generate_upf_driver_update_update(&self, new_ma_id: u32, old_ma_id: u32, global_qer_id: u16) -> upf_driver_interface::PDRUpdate {
		upf_driver_interface::PDRUpdate::UL_N6_SimpleUpdate(
			super::super::requests_generated::upfdriver::requests::UL_N6_SimpleUpdateRequestArgs {
				key_teid: self.key_teid,
				key_qfi: self.key_qfi,
				data_ma_id: new_ma_id,
				data_old_ma_id: old_ma_id,
				data_qer_id: global_qer_id,
				vol_thres: 0
			}
		)
	}
	fn generate_upf_driver_update_insert(&self, new_ma_id: u32, global_qer_id: u16) -> upf_driver_interface::PDRUpdate {
		upf_driver_interface::PDRUpdate::UL_N6_SimpleInsert(
			super::super::requests_generated::upfdriver::requests::UL_N6_SimpleInsertRequestArgs {
				key_teid: self.key_teid,
				key_qfi: self.key_qfi,
				data_ma_id: new_ma_id,
				data_qer_id: global_qer_id,
				vol_thres: 0
			}
		)
	}
}

impl ReorderMatchActionInstance for UplinkToN6Simple {
	fn to_p4_table_order(&self, order: i32) -> super::MatchInstance {
		assert_eq!(order, 4);
		let new_fow = super::uplink_n9_complex::UplinkToN9Complex {
			key_teid: self.key_teid,
			key_qfi: self.key_qfi,
			key_qfi_mask: 0x3F,
			key_dst_ip: std::net::Ipv4Addr::new(0, 0, 0, 0),
			key_dst_ip_mask: 0,
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
			teid: 0
		};
		super::MatchInstance::UplinkToN9Complex(new_fow)
	}
}
