use libpfcp::models::{SourceInterface, OuterHeaderRemovalDescription, DestinationInterface};

use crate::datapath::{FlattenedPacketPipeline, tofino::{bfruntime::{P4TableInfo, bfrt::{TableKey, TableEntry, key_field::{Exact, self, Ternary, Range}, KeyField, table_entry, DataField, TableData, data_field}}, match_action_id::ActionInstance}};

use super::{PipelineTemplate, super::TofinoBackendError, UsageReportingKey, reorder::ReorderMatchActionInstance};


#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UplinkToN6Complex {
	pub key_teid: u32,
	pub key_qfi: u8,
	pub key_qfi_mask: u8,
	pub key_dst_ip: std::net::Ipv4Addr,
	pub key_dst_ip_mask: u32,
	pub key_src_ip: std::net::Ipv4Addr,
	pub key_src_ip_mask: u32,
	pub key_ip_proto: u8,
	pub key_ip_proto_mask: u8,
	pub key_tos: u8,
	pub key_tos_mask: u8,
	pub key_src_port_range: (u16, u16),
	pub key_dst_port_range: (u16, u16),
	pub key_priority: i32,

	pub pdr_id: u16,
}

impl PipelineTemplate for UplinkToN6Complex {
	fn hit(pipeline: &FlattenedPacketPipeline, pdr_id: u16) -> Result<Option<(Self, ActionInstance)>, TofinoBackendError> {
		if pipeline.pdi.source_interface == SourceInterface::AccessSide {
			if  pipeline.pdi.dst_ipv6.is_none() &&
				pipeline.pdi.dst_ipv6_mask.is_none() &&
				pipeline.pdi.src_ipv6.is_none() &&
				pipeline.pdi.src_ipv6_mask.is_none() &&
				pipeline.pdi.ipv6_flow_label.is_none()
			//    pipeline.qfi.is_none() &&
			//    pipeline.forwarding_parameters.is_none()
			{
				let mut mark_tos = None;
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
						let (dst_ip, dst_ip_mask) = if let (Some(ip), mask) = (pipeline.pdi.dst_ipv4, pipeline.pdi.dst_ipv4_mask) {
							if let Some(mask) = mask {
								(ip, mask)
							} else {
								(ip, u32::MAX)
							}
						} else {
							(std::net::Ipv4Addr::new(0, 0, 0, 0), 0u32)
						};
						let (src_ip, src_ip_mask) = if let (Some(ip), mask) = (pipeline.pdi.src_ipv4, pipeline.pdi.src_ipv4_mask) {
							if let Some(mask) = mask {
								(ip, mask)
							} else {
								(ip, u32::MAX)
							}
						} else {
							(std::net::Ipv4Addr::new(0, 0, 0, 0), 0u32)
						};
						let (ip_proto, ip_proto_mask) = if let Some(p) = pipeline.pdi.ip_proto {
							(p, u8::MAX)
						} else {
							(0, 0)
						};
						let (key_tos, key_tos_mask) = if let Some((t, tm)) = pipeline.pdi.tos {
							(t, tm)
						} else {
							(0, 0)
						};
						let src_port_range = if let Some((low, high)) = pipeline.pdi.src_port_range {
							(low, high)
						} else {
							(u16::MIN, u16::MAX)
						};
						let dst_port_range = if let Some((low, high)) = pipeline.pdi.dst_port_range {
							(low, high)
						} else {
							(u16::MIN, u16::MAX)
						};
						Ok(Some(
							(
								UplinkToN6Complex {
									key_teid: f_teid.teid.unwrap(),
									key_qfi: pipeline.pdi.qfi.unwrap_or_default(),
									key_qfi_mask: if pipeline.pdi.qfi.is_some() { 0x3F } else { 0 },
									key_dst_ip: dst_ip,
									key_dst_ip_mask: dst_ip_mask,
									key_src_ip: src_ip,
									key_src_ip_mask: src_ip_mask,
									key_ip_proto: ip_proto,
									key_ip_proto_mask: ip_proto_mask,
									key_tos,
									key_tos_mask,
									key_src_port_range: src_port_range,
									key_dst_port_range: dst_port_range,
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
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.gtpu.teid[23:0]"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: self.key_teid.to_be_bytes()[1..].to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.gtpu_ext_psc.qfi"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: vec![self.key_qfi],
						mask: vec![self.key_qfi_mask],
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_ipv4.dstAddr"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: self.key_dst_ip.octets().to_vec(),
						mask: self.key_dst_ip_mask.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_ipv4.srcAddr"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: self.key_src_ip.octets().to_vec(),
						mask: self.key_src_ip_mask.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_ipv4.protocol"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: self.key_ip_proto.to_be_bytes().to_vec(),
						mask: self.key_ip_proto_mask.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_ipv4.diffserv"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: self.key_tos.to_be_bytes().to_vec(),
						mask: self.key_tos_mask.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_tcp_udp.srcPort"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: 0u16.to_be_bytes().to_vec(),
						mask: 0u16.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "hdr.overlay_tcp_udp.dstPort"),
					match_type: Some(key_field::MatchType::Ternary(Ternary {
						value: 0u16.to_be_bytes().to_vec(),
						mask: 0u16.to_be_bytes().to_vec()
					}))
				},
				KeyField {
					field_id: info.get_key_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "$MATCH_PRIORITY"),
					match_type: Some(key_field::MatchType::Exact(Exact {
						value: self.key_priority.to_be_bytes().to_vec()
					}))
				}
			]
		}
	}
	fn generate_table_action_data(&self, info: &P4TableInfo, maid: u32, qer_id: u16) -> TableData {
		TableData {
			action_id: info.get_action_id_by_name("pipe.Ingress.pdr.ul_N6_complex_ipv4", "Ingress.pdr.set_ma_id_ul_N6_complex_ipv4"),
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
		"pipe.Ingress.pdr.ul_N6_complex_ipv4"
	}
	fn get_p4_table_order(&self) -> i32 {
		2
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

impl ReorderMatchActionInstance for UplinkToN6Complex {
	fn to_p4_table_order(&self, order: i32) -> super::MatchInstance {
		assert_eq!(order, 4);
		let new_fow = super::uplink_n9_complex::UplinkToN9Complex {
			key_teid: self.key_teid,
			key_qfi: self.key_qfi,
			key_qfi_mask: self.key_qfi_mask,
			key_dst_ip: self.key_dst_ip,
			key_dst_ip_mask: self.key_dst_ip_mask,
			key_src_ip: self.key_src_ip,
			key_src_ip_mask: self.key_src_ip_mask,
			key_ip_proto: self.key_ip_proto,
			key_ip_proto_mask: self.key_ip_proto_mask,
			key_tos: self.key_tos,
			key_tos_mask: self.key_tos_mask,
			key_src_port_range: self.key_src_port_range,
			key_dst_port_range: self.key_dst_port_range,
			key_priority: self.key_priority,
			teid: 0,
			pdr_id: self.pdr_id
		};
		super::MatchInstance::UplinkToN9Complex(new_fow)
	}
}

