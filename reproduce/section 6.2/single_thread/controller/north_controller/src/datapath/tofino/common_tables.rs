
use crate::datapath::tofino::bfruntime::bfrt::key_field::{Lpm, Exact};

use super::bfruntime::{bfrt::*, P4TableInfo};

pub fn populate_cpu_pcie_port(tofino_table_info: &P4TableInfo, cpu_port: u16) -> Vec<Update> {
	let table_key = TableKey {
		fields: vec![
			KeyField {
				field_id: tofino_table_info.get_key_id_by_name("$pre.port", "$DEV_PORT"),
				match_type: Some(key_field::MatchType::Exact(Exact {
					value: (cpu_port as u32).to_be_bytes().to_vec()
				}))
			}
		]
	};
	let table_data = TableData {
		action_id: 0,
		fields: vec![
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$pre.port", "$COPY_TO_CPU_PORT_ENABLE"),
				value: Some(data_field::Value::BoolVal(true))
			}
		]
	};
	let table_entry = TableEntry {
		table_id: tofino_table_info.get_table_by_name("$pre.port"),
		data: Some(table_data),
		is_default_entry: false,
		table_read_flag: None,
		table_mod_inc_flag: None,
		entry_tgt: None,
		table_flags: None,
		value: Some(table_entry::Value::Key(table_key)),
	};
	let entity = Entity {
		entity: Some(entity::Entity::TableEntry(table_entry))
	};
	let update = Update {
		r#type: update::Type::Insert as _,
		entity: Some(entity)
	};
	vec![update]
}

pub fn populate_upf_self_ip(table_info: &P4TableInfo, upf_ip: std::net::Ipv4Addr) -> Vec<Update> {
	let table_key = TableKey {
		fields: vec![
			// KeyField {
			// 	field_id: table_info.get_key_id_by_name("pipe.Ingress.set_upf_ip_table", "meta.always_one"),
			// 	match_type: Some(key_field::MatchType::Exact(Exact { value: vec![1u8] }))
			// }
		]
	};
	let table_data = TableData {
		action_id: table_info.get_action_id_by_name("pipe.Ingress.set_upf_ip_table", "Ingress.set_upf_ip_table_set_ip"),
		fields: vec![
			DataField {
				field_id: 1,
				value: Some(data_field::Value::Stream(upf_ip.octets().to_vec()))
			}
		]
	};
	let table_entry = TableEntry {
		table_id: table_info.get_table_by_name("pipe.Ingress.set_upf_ip_table"),
		data: Some(table_data),
		is_default_entry: true,
		table_read_flag: None,
		table_mod_inc_flag: None,
		entry_tgt: None,
		table_flags: None,
		value: None,//Some(table_entry::Value::Key(table_key)),
	};
	let entity = Entity {
		entity: Some(entity::Entity::TableEntry(table_entry))
	};
	let update = Update {
		r#type: update::Type::Modify as _,
		entity: Some(entity)
	};
	vec![update]
}

pub fn populate_flow_key_reporting_egree_mirror(tofino_table_info: &P4TableInfo, cpu_port: u16) -> Vec<Update> {
	let mirror_session = 1u16;
	let dst_port = cpu_port as u32;
	let table_key = TableKey {
		fields: vec![
			KeyField {
				field_id: tofino_table_info.get_key_id_by_name("$mirror.cfg", "$sid"),
				match_type: Some(key_field::MatchType::Exact(Exact {
					value: mirror_session.to_be_bytes().to_vec()
				}))
			}
		]
	};
	let table_data = TableData {
		action_id: tofino_table_info.get_action_id_by_name("$mirror.cfg", "$normal"),
		fields: vec![
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$mirror.cfg", "$session_enable"),
				value: Some(data_field::Value::BoolVal(true))
			},
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$mirror.cfg", "$direction"),
				value: Some(data_field::Value::StrVal("EGRESS".into())),
			},
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$mirror.cfg", "$ucast_egress_port"),
				value: Some(data_field::Value::Stream(dst_port.to_be_bytes().to_vec()))
			},
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$mirror.cfg", "$ucast_egress_port_valid"),
				value: Some(data_field::Value::BoolVal(true))
			},
			DataField {
				field_id: tofino_table_info.get_data_id_by_name("$mirror.cfg", "$max_pkt_len"),
				value: Some(data_field::Value::Stream(128u16.to_be_bytes().to_vec()))
			},
		]
	};
	let table_entry = TableEntry {
		table_id: tofino_table_info.get_table_by_name("$mirror.cfg"),
		data: Some(table_data),
		is_default_entry: false,
		table_read_flag: None,
		table_mod_inc_flag: None,
		entry_tgt: None,
		table_flags: None,
		value: Some(table_entry::Value::Key(table_key)),
	};
	let entity = Entity {
		entity: Some(entity::Entity::TableEntry(table_entry))
	};
	let update = Update {
		r#type: update::Type::Insert as _,
		entity: Some(entity)
	};
	vec![update]
}
