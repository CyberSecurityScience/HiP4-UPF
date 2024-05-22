use std::{thread::JoinHandle, time::Duration};

use itertools::Itertools;
use libpfcp::{messages::{PFCPSessionReportRequest, DownlinkDataReport}, PFCPModel, models::ReportType};
use linear_map::set::LinearSet;
use log::{warn, info, error};
use tokio::sync::mpsc::Receiver;

use crate::datapath::{FlattenedPacketPipeline, PacketPipeline, FlattenedPDI, Report, BackendReport};

pub fn flatten_packet_pipeline(pipeline: &PacketPipeline) -> Vec<FlattenedPacketPipeline> {
	let pdi = &pipeline.pdi;
	let mut ret = LinearSet::new();
	let mut ip_proto = None;
	let tos;
	let ipv6_flow_label;
	let mut src_port_range;
	let mut dst_port_range;
	let mut src_ipv4;
	let mut src_ipv4_mask;
	let mut src_ipv6;
	let mut src_ipv6_mask;
	let mut dst_ipv4;
	let mut dst_ipv4_mask;
	let mut dst_ipv6;
	let mut dst_ipv6_mask;
	src_port_range = vec![];
	dst_port_range = vec![];
	src_ipv4 = None;
	src_ipv4_mask = None;
	src_ipv6 = None;
	src_ipv6_mask = None;
	dst_ipv4 = None;
	dst_ipv4_mask = None;
	dst_ipv6 = None;
	dst_ipv6_mask = None;
	if let Some(sdf_filter) = &pdi.sdf_filter {
		tos = sdf_filter.get_tos();
		ipv6_flow_label = sdf_filter.flow_label.clone();
		if let Some(flow_desc) = &sdf_filter.flow_desc {
			let flow_desc_ret = libpfcp::helpers::DiameterIPFilterRule::from_string(flow_desc);
			match flow_desc_ret {
				Ok(flow_desc) => {
					ip_proto = flow_desc.ip_proto;
					src_port_range = flow_desc.src_port_range;
					dst_port_range = flow_desc.dst_port_range;
					if let Some(src_ip) = flow_desc.src_ip {
						match src_ip {
							cidr::IpCidr::V4(cidr) => {
								src_ipv4 = Some(cidr.first_address());
								src_ipv4_mask = Some(u32::from_be_bytes(cidr.mask().octets()));
							}
							cidr::IpCidr::V6(cidr) => {
								src_ipv6 = Some(cidr.first_address());
								src_ipv6_mask = Some(u128::from_be_bytes(cidr.mask().octets()));
							}
						}
					}
					if let Some(dst_ip) = flow_desc.dst_ip {
						match dst_ip {
							cidr::IpCidr::V4(cidr) => {
								dst_ipv4 = Some(cidr.first_address());
								dst_ipv4_mask = Some(u32::from_be_bytes(cidr.mask().octets()));
							}
							cidr::IpCidr::V6(cidr) => {
								dst_ipv6 = Some(cidr.first_address());
								dst_ipv6_mask = Some(u128::from_be_bytes(cidr.mask().octets()));
							}
						}
					}
				}
				Err(e) => {
					warn!("Ignoring Flow-Description AVP error: {:?}", e);
				}
			}
		}
	} else {
		tos = None;
		ipv6_flow_label = None;
	}
	for ueip_i in 0..(1 + pdi.ue_ip_address.len()) {
		for qfi_i in 0..(1 + pdi.qfi.len()) {
			for src_port_range_i in 0..(1 + src_port_range.len()) {
				for dst_port_range_i in 0..(1 + dst_port_range.len()) {
					let mut cur = FlattenedPDI {
						source_interface: pdi.source_interface,
						local_f_teid: pdi.local_f_teid.clone(),
						qfi: None,
						ip_proto: ip_proto,
						tos: tos,
						ipv6_flow_label: ipv6_flow_label,
						src_ipv4: src_ipv4,
						src_ipv4_mask: src_ipv4_mask,
						src_ipv6: src_ipv6,
						src_ipv6_mask: src_ipv6_mask,
						dst_ipv4: dst_ipv4,
						dst_ipv4_mask: dst_ipv4_mask,
						dst_ipv6: dst_ipv6,
						dst_ipv6_mask: dst_ipv6_mask,
						src_port_range: None,
						dst_port_range: None,
					};
					if qfi_i > 0 {
						cur.qfi = Some(pdi.qfi[qfi_i - 1].0);
					}
					if src_port_range_i > 0 {
						cur.src_port_range = Some(src_port_range[src_port_range_i - 1]);
					}
					if dst_port_range_i > 0 {
						cur.dst_port_range = Some(dst_port_range[dst_port_range_i - 1]);
					}
					if ueip_i > 0 {
						match pdi.source_interface {
							libpfcp::models::SourceInterface::AccessSide => {
								let ueip = &pdi.ue_ip_address[ueip_i - 1];
								if ueip.flags.getSD() == 0 {
									if let Some(v4) = ueip.ipv4 {
										cur.src_ipv4 = Some(v4);
										cur.src_ipv4_mask = Some(u32::MAX);
									}
									if let Some(v6) = ueip.ipv6 {
										cur.src_ipv6 = Some(v6);
										cur.src_ipv6_mask = Some(u128::MAX);
									}
								}
							},
							libpfcp::models::SourceInterface::CoreSide => {
								let ueip = &pdi.ue_ip_address[ueip_i - 1];
								if ueip.flags.getSD() == 1 {
									if let Some(v4) = ueip.ipv4 {
										cur.dst_ipv4 = Some(v4);
										cur.dst_ipv4_mask = Some(u32::MAX);
									}
									if let Some(v6) = ueip.ipv6 {
										cur.dst_ipv6 = Some(v6);
										cur.dst_ipv6_mask = Some(u128::MAX);
									}
								}
							},
							o => warn!("Ignoring unsupported SourceInterface {:?}", o),
						}
					}
					if qfi_i > 0 || src_port_range_i > 0 || dst_port_range_i > 0 || ueip_i > 0 {
						ret.insert(cur);
					}
				}
			}
		}
	}
	if pdi.ue_ip_address.len() == 0 && pdi.qfi.len() == 0 && src_port_range.len() == 0 && dst_port_range.len() == 0 {
		let cur = FlattenedPDI {
			source_interface: pdi.source_interface,
			local_f_teid: pdi.local_f_teid.clone(),
			qfi: None,
			ip_proto: ip_proto,
			tos: tos,
			ipv6_flow_label: ipv6_flow_label,
			src_ipv4: src_ipv4,
			src_ipv4_mask: src_ipv4_mask,
			src_ipv6: src_ipv6,
			src_ipv6_mask: src_ipv6_mask,
			dst_ipv4: dst_ipv4,
			dst_ipv4_mask: dst_ipv4_mask,
			dst_ipv6: dst_ipv6,
			dst_ipv6_mask: dst_ipv6_mask,
			src_port_range: None,
			dst_port_range: None,
		};
		ret.insert(cur);
	}
					
	let mut ans = vec![];
	let mut first = true;
	for pdi in ret {
		let pfcpsm_req_flags = if first {
			pipeline.pfcpsm_req_flags.clone()
		} else {
			None
		};
		if first {
			first = false;
		}
		ans.push(
			FlattenedPacketPipeline {
				seid: pipeline.seid,
				pdr_id: pipeline.pdr_id.rule_id,
				precedence: pipeline.precedence.precedence,
				pdi: pdi,
				outer_header_removal: pipeline.outer_header_removal.clone(),
				qer_id: pipeline.qer_id.as_ref().map(|f| f.rule_id),
				gate_status: pipeline.gate_status.clone(),
				// maximum_bitrate: pipeline.maximum_bitrate.clone(),
				// guaranteed_bitrate: pipeline.guaranteed_bitrate.clone(),
				qfi: pipeline.qfi.as_ref().map(|f| f.0),
				apply_action: pipeline.apply_action.clone(),
				forwarding_parameters: pipeline.forwarding_parameters.clone(),
				pfcpsm_req_flags,
				urr_ids: pipeline.urr_ids.iter().map(|f| f.rule_id).collect::<Vec<_>>(),
				// urrs: pipeline.urrs.clone(),
				suggested_buffering_packets_count: pipeline.suggested_buffering_packets_count.as_ref().map(|f| f.value),
			}
		);
	}
	ans
}
