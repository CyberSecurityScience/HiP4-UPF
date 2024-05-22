
use linear_map::LinearMap;

use crate::{datapath::tofino::bfruntime::bfrt::key_field::{Lpm, Exact}, routing::RoutingTable};

use super::bfruntime::{bfrt::*, P4TableInfo};

pub fn populate_routing_table(info: &P4TableInfo, table: &RoutingTable) -> Vec<Update> {
	let mut entries = Vec::with_capacity(table.routing.len());
	for e in table.routing.iter() {
		let tunnel = match e.interface {
			crate::routing::DeviceInterfaceType::N6 => false,
			crate::routing::DeviceInterfaceType::N3 => true,
			crate::routing::DeviceInterfaceType::N9 => true,
		};
		let table_entry;
		let table_id = match (tunnel, e.cidr) {
			(true, cidr::IpCidr::V4(c)) => {
				let table_key = TableKey {
					fields: vec![
						KeyField {
							field_id: info.get_key_id_by_name("pipe.Ingress.ipv4_routing_underlay.ipv4_lpm", "dst_ip"),
							match_type: Some(key_field::MatchType::Lpm(Lpm {
								value: c.first_address().octets().to_vec(),
								prefix_len: c.network_length() as _
							}))
						}
					]
				};
				let table_data = TableData {
					action_id: info.get_action_id_by_name("pipe.Ingress.ipv4_routing_underlay.ipv4_lpm", "Ingress.ipv4_routing_underlay.send"),
					fields: vec![
						DataField {
							field_id: 1,
							value: Some(data_field::Value::Stream(e.device_port.to_be_bytes().to_vec())) // <-- 9bit PortId_t
						},
						DataField {
							field_id: 2,
							value: Some(data_field::Value::Stream(e.src_mac.octets().to_vec())) // <-- 48bit MacAddr_t
						},
						DataField {
							field_id: 3,
							value: Some(data_field::Value::Stream(e.dst_mac.octets().to_vec())) // <-- 48bit MacAddr_t
						}
					]
				};
				table_entry = TableEntry {
					table_id: info.get_table_by_name("pipe.Ingress.ipv4_routing_underlay.ipv4_lpm"),
					data: Some(table_data),
					is_default_entry: false,
					table_read_flag: None,
					table_mod_inc_flag: None,
					entry_tgt: None,
					table_flags: None,
					value: Some(table_entry::Value::Key(table_key)),
				};
			},
			(false, cidr::IpCidr::V4(c)) => {
				let table_key = TableKey {
					fields: vec![
						KeyField {
							field_id: info.get_key_id_by_name("pipe.Ingress.ipv4_routing_overlay.ipv4_lpm", "dst_ip"),
							match_type: Some(key_field::MatchType::Lpm(Lpm {
								value: c.first_address().octets().to_vec(),
								prefix_len: c.network_length() as _
							}))
						}
					]
				};
				let table_data = TableData {
					action_id: info.get_action_id_by_name("pipe.Ingress.ipv4_routing_overlay.ipv4_lpm", "Ingress.ipv4_routing_overlay.send"),
					fields: vec![
						DataField {
							field_id: 1,
							value: Some(data_field::Value::Stream(e.device_port.to_be_bytes().to_vec())) // <-- 9bit PortId_t
						},
						DataField {
							field_id: 2,
							value: Some(data_field::Value::Stream(e.src_mac.octets().to_vec())) // <-- 48bit MacAddr_t
						},
						DataField {
							field_id: 3,
							value: Some(data_field::Value::Stream(e.dst_mac.octets().to_vec())) // <-- 48bit MacAddr_t
						}
					]
				};
				table_entry = TableEntry {
					table_id: info.get_table_by_name("pipe.Ingress.ipv4_routing_overlay.ipv4_lpm"),
					data: Some(table_data),
					is_default_entry: false,
					table_read_flag: None,
					table_mod_inc_flag: None,
					entry_tgt: None,
					table_flags: None,
					value: Some(table_entry::Value::Key(table_key)),
				};
			},
            (true, cidr::IpCidr::V6(_)) => panic!("IPv6 not supported!"),
            (false, cidr::IpCidr::V6(_)) => panic!("IPv6 not supported!"),
		};
		entries.push(table_entry);
	}
	entries
		.into_iter()
		.map(|e| {
			let entity = Entity {
				entity: Some(entity::Entity::TableEntry(e))
			};
			Update {
				r#type: update::Type::Insert as _,
				entity: Some(entity)
			}
		})
		.collect::<Vec<_>>()
}
