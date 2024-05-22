use libpfcp::models::{SourceInterface, OuterHeaderRemovalDescription, DestinationInterface};

use crate::datapath::{FlattenedPacketPipeline, tofino::{bfruntime::{P4TableInfo, bfrt::{TableKey, TableEntry, key_field::{Exact, self}, KeyField, table_entry, DataField, TableData, data_field}}, match_action_id::ActionInstance}};

use super::{PipelineTemplate, super::TofinoBackendError, reorder::ReorderMatchActionInstance};


#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UplinkToN9Simple {
	pub key_qfi: u8,
	pub key_teid: u32,
	pub key_priority: i32,

	pub pdr_id: u16,
	pub teid: u32,
}

impl PipelineTemplate for UplinkToN9Simple {
	fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> {
		if pipeline.pdi.source_interface == SourceInterface::AccessSide {
			if pipeline.pdi.dst_ipv4.is_none() &&
				pipeline.pdi.dst_ipv4_mask.is_none() &&
				pipeline.pdi.dst_ipv6.is_none() &&
				pipeline.pdi.dst_ipv6_mask.is_none() &&
				pipeline.pdi.dst_port_range.is_none() &&
				pipeline.pdi.ip_proto.is_none() &&
				pipeline.pdi.qfi.is_some() &&
				pipeline.pdi.tos.is_none() &&
			//    pipeline.pdi.src_ipv4.is_none() &&
			//    pipeline.pdi.src_ipv4_mask.is_none() &&
				pipeline.pdi.src_ipv6.is_none() &&
				pipeline.pdi.src_ipv6_mask.is_none() &&
				pipeline.pdi.ipv6_flow_label.is_none() &&
				pipeline.pdi.src_port_range.is_none() &&
				pipeline.pdi.dst_port_range.is_none() &&
				pipeline.forwarding_parameters.is_some()
			{
				if let Some(ip_mask) = &pipeline.pdi.src_ipv4_mask {
					if *ip_mask != u32::MAX {
						return Ok(None);
					}
				}
				if let (Some(f_teid), Some(ohr), Some(fow)) = (&pipeline.pdi.local_f_teid, &pipeline.outer_header_removal, &pipeline.forwarding_parameters) {
					if fow.outer_header_creation.is_none() {
						return Ok(None);
					}
					if fow.transport_level_marking.is_some() {
						return Ok(None);
					}
					let ohc = fow.outer_header_creation.as_ref().unwrap();
					if fow.destination_interface != DestinationInterface::CoreSide && fow.destination_interface != DestinationInterface::AccessSide {
						return Ok(None);
					}
					let a = ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IPv4 || ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IPv6 || ohr.desc == OuterHeaderRemovalDescription::GTP_U_UDP_IP;
					let b = f_teid.choose_id.is_none() && (f_teid.ipv4.is_some() || f_teid.ipv6.is_some()) && f_teid.teid.is_some();
					let ohc_condition = ohc.ipv4.is_some() && ohc.teid.is_some() && ohc.desc.getGTP_U_UDP_IPv4() == 1;
					let r = a & b & ohc_condition;
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
						let mut teid = 0;
						let action_tuple = if drop {
							ActionInstance::DropUl(nocp)
						} else if pipeline.qfi.is_some() {
							if ohc.ipv4.is_some() && ohc.teid.is_some() && ohc.desc.getGTP_U_UDP_IPv4() == 1 && ohc.ipv4.is_some() {
								teid = ohc.teid.unwrap();
								ActionInstance::EncapUl(nocp, ohc.ipv4.unwrap(), pipeline.qfi.unwrap())
							} else {
								return Ok(None);
							}
						} else {
							return Ok(None);
						};
						Ok(Some(
							(
								Self {
									key_teid: f_teid.teid.unwrap(),
									key_qfi: pipeline.pdi.qfi.unwrap(),
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
		if self.teid & 0xff000000u32 != 0 {
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
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N9_simple_ipv4", "hdr.gtpu.teid[23:0]"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: self.key_teid.to_be_bytes()[1..].to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N9_simple_ipv4", "hdr.gtpu_ext_psc.qfi"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: vec![self.key_qfi]
					}))
				}
			]
		}
	}
	fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> TableData {
		TableData {
			action_id: info.get_action_id_by_name("pipe.Ingress.pdr.ul_N9_simple_ipv4", "Ingress.pdr.set_ma_id_and_tunnel_ul_N9_simple_ipv4"),
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
		"pipe.Ingress.pdr.ul_N9_simple_ipv4"
	}
	fn get_p4_table_order(&self) -> i32 {
		5
	}
	fn generate_upf_driver_update_remove(&self, old_ma_id: u32) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        todo!()
    }
	fn generate_upf_driver_update_update(&self, old_ma_id: u32, new_ma_id: u32, global_qer_id: u16) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        todo!()
    }
	fn generate_upf_driver_update_insert(&self, new_ma_id: u32, global_qer_id: u16) -> crate::datapath::tofino::upf_driver_interface::PDRUpdate {
        todo!()
    }
}

impl ReorderMatchActionInstance for UplinkToN9Simple {
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

			teid: self.teid,
			pdr_id: self.pdr_id
		};
		super::MatchInstance::UplinkToN9Complex(new_fow)
	}
}
