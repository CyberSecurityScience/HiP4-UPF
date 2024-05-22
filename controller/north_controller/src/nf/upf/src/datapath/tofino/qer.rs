use std::collections::HashMap;

use libpfcp::IDAllocator;

use crate::datapath::FlattenedQER;

use super::{bfruntime::{bfrt::{Update, key_field::{self, Exact}, TableKey, KeyField, DataField, TableData, data_field, table_entry, entity, update, TableEntry, Entity}, P4TableInfo}, TofinoBackendError};

impl FlattenedQER {
    /// DL is even, UL is odd
    pub fn to_table_values_dl(&self) -> (u64, u64, u64, u64) {
        let (gbr_kbps, gbr_burst) = if let Some(v) = self.guaranteed_bitrate {
            (v.dl_mbr, v.dl_mbr / 10) // 100ms of burst
        } else {
            (0, 0)
        };
        let (mbr_kbps, mbr_burst) = if let Some(v) = self.maximum_bitrate {
            (v.dl_mbr, v.dl_mbr / 10)
        } else {
            (u64::MAX, u64::MAX)
        };
        (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst)
    }
    pub fn to_table_values_ul(&self) -> (u64, u64, u64, u64) {
        let (gbr_kbps, gbr_burst) = if let Some(v) = self.guaranteed_bitrate {
            (v.ul_mbr, v.ul_mbr / 10) // 100ms of burst
        } else {
            (0, 0)
        };
        let (mbr_kbps, mbr_burst) = if let Some(v) = self.maximum_bitrate {
            (v.ul_mbr, v.ul_mbr / 10)
        } else {
            (u64::MAX, u64::MAX)
        };
        (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QosRequirement {
    /// 1kbps = 1000bit per sec
    pub gbr_kbps: Option<u64>,
    /// 1kbps = 1000bit per sec
    pub mbr_kbps: Option<u64>
}

impl QosRequirement {
    pub fn to_table_values(&self) -> (u64, u64, u64, u64) {
        let (gbr_kbps, gbr_burst) = if let Some(v) = self.gbr_kbps {
            (v, v / 10)
        } else {
            (0, 0)
        };
        let (mbr_kbps, mbr_burst) = if let Some(v) = self.mbr_kbps {
            (v, v / 10)
        } else {
            (u64::MAX, u64::MAX)
        };
        (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst)
    }
}

pub struct GlobalQerContext {
    id_alloc: IDAllocator<u16>,
    qers: HashMap<u16, FlattenedQER>,
    updates: Vec<Update>
}

impl GlobalQerContext {
    pub fn new() -> Self {
        Self {
            id_alloc: IDAllocator::new_with_counter(2),
            qers: HashMap::new(),
            updates: vec![]
        }
    }
    pub fn transaction_begin(&mut self) {
        self.updates.clear();
    }
    pub fn transaction_commit(&mut self) -> Vec<Update> {
        self.updates.clone()
    }
    pub fn allocate_qer_id(&mut self, table_info: &P4TableInfo, qos: FlattenedQER) -> Option<u16> {
        if !qos.is_meter_session() {
            return Some(0);
        }
        let global_qer_id_dl = self.id_alloc.allocate();
        let global_qer_id_ul = self.id_alloc.allocate();
        if global_qer_id_dl.is_ok() && global_qer_id_ul.is_ok() {
            let global_qer_id = global_qer_id_dl.unwrap();
            self.updates.append(&mut Self::add_qer(table_info, qos, global_qer_id));
            self.qers.insert(global_qer_id, qos);
            Some(global_qer_id)
        } else {
            None
        }
    }
    pub fn update_qer(&mut self, table_info: &P4TableInfo, global_qer_id: u16, qos: FlattenedQER) -> Result<Option<u16>, TofinoBackendError> {
        if global_qer_id == 0 && qos.is_meter_session() {
            // from no meter to meter
            if let Some(id) = self.allocate_qer_id(table_info, qos) {
                return Ok(Some(id));
            } else {
                return Err(TofinoBackendError::InsufficientGBRFlowCapacity);
            }
        }
        if let Some(entry) = self.qers.get_mut(&global_qer_id) {
            // TODO: from meter to no meter
            if qos != *entry {
                *entry = qos;
                self.updates.append(&mut Self::mod_qer(table_info, qos, global_qer_id));
            }
        }
        Ok(None)
    }
    pub fn free_qer_id(&mut self, table_info: &P4TableInfo, global_qer_id: u16) {
        if global_qer_id == 0 {
            return;
        }
        if let Some(entry) = self.qers.remove(&global_qer_id) {
            self.updates.append(&mut Self::del_qer(table_info, global_qer_id));
            self.id_alloc.free(global_qer_id);
            self.id_alloc.free(global_qer_id + 1);
        }
    }
}

impl GlobalQerContext {
    fn add_qer(table_info: &P4TableInfo, qos: FlattenedQER, global_qer_id: u16) -> Vec<Update> {
        let update_dl = {
            let (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst) = qos.to_table_values_dl();
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: global_qer_id.to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: table_info.get_action_id_by_name("pipe.Ingress.bitrate_enforce_table", "Ingress.set_meter_color"),
                fields: vec![
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CIR_KBPS"),
                        value: Some(data_field::Value::Stream(gbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PIR_KBPS"),
                        value: Some(data_field::Value::Stream(mbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CBS_KBITS"),
                        value: Some(data_field::Value::Stream(gbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PBS_KBITS"),
                        value: Some(data_field::Value::Stream(mbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
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
            Update {
                r#type: update::Type::Insert as _,
                entity: Some(entity)
            }
        };
        let update_ul = {
            let (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst) = qos.to_table_values_ul();
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: (global_qer_id + 1).to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: table_info.get_action_id_by_name("pipe.Ingress.bitrate_enforce_table", "Ingress.set_meter_color"),
                fields: vec![
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CIR_KBPS"),
                        value: Some(data_field::Value::Stream(gbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PIR_KBPS"),
                        value: Some(data_field::Value::Stream(mbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CBS_KBITS"),
                        value: Some(data_field::Value::Stream(gbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PBS_KBITS"),
                        value: Some(data_field::Value::Stream(mbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
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
            Update {
                r#type: update::Type::Insert as _,
                entity: Some(entity)
            }
        };
        vec![update_dl, update_ul]
    }
    fn mod_qer(table_info: &P4TableInfo, qos: FlattenedQER, global_qer_id: u16) -> Vec<Update> {
        let update_dl = {
            let (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst) = qos.to_table_values_dl();
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: global_qer_id.to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: table_info.get_action_id_by_name("pipe.Ingress.bitrate_enforce_table", "Ingress.set_meter_color"),
                fields: vec![
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CIR_KBPS"),
                        value: Some(data_field::Value::Stream(gbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PIR_KBPS"),
                        value: Some(data_field::Value::Stream(mbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CBS_KBITS"),
                        value: Some(data_field::Value::Stream(gbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PBS_KBITS"),
                        value: Some(data_field::Value::Stream(mbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
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
            Update {
                r#type: update::Type::Modify as _,
                entity: Some(entity)
            }
        };
        let update_ul = {
            let (gbr_kbps, gbr_burst, mbr_kbps, mbr_burst) = qos.to_table_values_ul();
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: (global_qer_id + 1).to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: table_info.get_action_id_by_name("pipe.Ingress.bitrate_enforce_table", "Ingress.set_meter_color"),
                fields: vec![
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CIR_KBPS"),
                        value: Some(data_field::Value::Stream(gbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PIR_KBPS"),
                        value: Some(data_field::Value::Stream(mbr_kbps.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_CBS_KBITS"),
                        value: Some(data_field::Value::Stream(gbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Ingress.bitrate_enforce_table", "$METER_SPEC_PBS_KBITS"),
                        value: Some(data_field::Value::Stream(mbr_burst.to_be_bytes().to_vec())) // <-- 64bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
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
            Update {
                r#type: update::Type::Modify as _,
                entity: Some(entity)
            }
        };
        vec![update_dl, update_ul]
    }
    fn del_qer(table_info: &P4TableInfo, global_qer_id: u16) -> Vec<Update> {
        let update_dl = {
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: global_qer_id.to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
                data: None,
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
            Update {
                r#type: update::Type::Delete as _,
                entity: Some(entity)
            }
        };
        let update_ul = {
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Ingress.bitrate_enforce_table", "meta.qer_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: (global_qer_id + 1).to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Ingress.bitrate_enforce_table"),
                data: None,
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
            Update {
                r#type: update::Type::Delete as _,
                entity: Some(entity)
            }
        };
        vec![update_dl, update_ul]
    }
}
