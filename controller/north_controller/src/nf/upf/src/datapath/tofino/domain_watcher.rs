
use libpfcp::IDAllocator;
use log::{warn, info};
use tokio::sync::{RwLock, Barrier, mpsc::{self, Receiver, Sender}};
use std::{sync::{Arc, Mutex}, collections::HashMap};
use lazy_static::lazy_static;

use super::bfruntime::{self, bfrt::{Update, TableKey, KeyField, TableEntry, key_field::{self, Exact}, table_entry, entity::{self}, update, Entity, TableData, DataField, data_field}, P4TableInfo};

const NUM_SCORES: usize = 1;

pub struct SingleDomain {
    pub domain: String,
    pub hash: u32,
    pub scores: Vec<i16>
}

pub struct DomainWatcherModel {
    domain_scores: Vec<SingleDomain>,
    threshold: i16
}

fn extract_to_label_vec(dst: &mut Vec<(usize, Vec<u8>)>, domain: &[u8], start_len: usize) {
    if domain.len() > start_len + start_len - 1 {
        warn!("skipping subdomain {}", String::from_utf8(domain.to_vec()).unwrap());
        return;
    }
    let mut domain = domain;
    let mut segment_idx = 0;
    let mut try_length = start_len;
    while try_length != 0 && domain.len() != 0 {
        if domain.len() >= try_length {
            let left_part = &domain[..try_length];
            domain = &domain[try_length..];
            let (l, seg) = dst.get_mut(segment_idx).unwrap();
            assert_eq!(*l, try_length);
            *seg = left_part.to_owned();
        }
        try_length /= 2;
        segment_idx += 1;
    }
    assert_eq!(domain.len(), 0);
}

fn populate_tld_table_impl(tld_list: Vec<String>, table_info: &P4TableInfo) -> Vec<Update> {
    // populate TLD table
    let mut updates = vec![];
    let mut n_added_tlds = 0usize;
    for tld in tld_list {
        // allocate space
        let mut labels = Vec::with_capacity(2);
        for label_id in 0..2usize {
            let mut label_segments = Vec::with_capacity(5);
            for length in [16, 8, 4, 2, 1] {
                label_segments.push((length, vec![0u8; length]));
            }
            labels.push(label_segments);
        }
        
        // TODO:
        let splits = tld.split(".").collect::<Vec<_>>();
        if splits.len() == 2 {
            // for i in 0..splits.len() {
            //     extract_to_label_vec(labels.get_mut(i).unwrap(), splits[i].as_bytes(), 16);
            // }
            extract_to_label_vec(labels.get_mut(0).unwrap(), splits[1].as_bytes(), 16);
            extract_to_label_vec(labels.get_mut(1).unwrap(), splits[0].as_bytes(), 16);
        } else {
            warn!("skipping TLD {}", tld);
        }
        
        let mut table_entry = TableKey {
            fields: vec![]
        };
        for (i, label) in labels.iter().enumerate() {
            for (length, value) in label {
                let key_field = KeyField {
                    field_id: table_info.get_key_id_by_name("pipe.Egress.dns_domain_parts_2", &format!("hdr.label{}_{}.l", i + 1, length)),
                    match_type: Some(key_field::MatchType::Exact(Exact {
                        value: value.clone()
                    }))
                };
                table_entry.fields.push(key_field);
            }
        }
        let table_data = TableData {
            action_id: table_info.get_action_id_by_name("pipe.Egress.dns_domain_parts_2", "Egress.dns_split_labels_action_2"),
            fields: vec![]
        };
        let table_entry = TableEntry {
            table_id: table_info.get_table_by_name("pipe.Egress.dns_domain_parts_2"),
            data: Some(table_data),
            is_default_entry: false,
            table_read_flag: None,
            table_mod_inc_flag: None,
            entry_tgt: None,
            table_flags: None,
            value: Some(table_entry::Value::Key(table_entry)),
        };
        let entity = Entity {
            entity: Some(entity::Entity::TableEntry(table_entry))
        };
        let update = Update {
            r#type: update::Type::Insert as _,
            entity: Some(entity)
        };
        updates.push(update);
        n_added_tlds += 1;
    }
    info!("Added {} TLDs", n_added_tlds);
    updates
}

use std::fs::read_to_string;

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename) 
        .unwrap()  // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}

pub fn populate_tld_table(tld_list_filename: &str, table_info: &P4TableInfo) -> Vec<Update> {
    // populate TLD table
    let tlds = read_lines(tld_list_filename);
    populate_tld_table_impl(tlds, table_info)
}

impl DomainWatcherModel {
    pub fn new(scores_filename: &str) -> DomainWatcherModel {
        let mut input = read_lines(scores_filename);
        assert!(input.len() > 1);
        let threshold: i16 = input[0].parse().unwrap();
        input.remove(0);
        let mut domain_scores = vec![];
        for item in input {
            let parts = item.split("\t").collect::<Vec<_>>();
            let domain = parts[0].to_owned();
            let hash = u32::from_str_radix(parts[1], 16).unwrap();
            let scores = parts[2].split(" ").collect::<Vec<_>>();
            assert_eq!(scores.len(), NUM_SCORES);
            let scores = scores
                .iter()
                .map(|s| s.parse().unwrap())
                .collect::<Vec<i16>>();
            domain_scores.push(SingleDomain { domain, hash, scores });
        }
        info!("DomainWatcher model created for {} domains with threshold {}", domain_scores.len(), threshold);
        Self {
            domain_scores,
            threshold
        }
    }

    fn to_entities(
        &self,
        table_info: &P4TableInfo,
        model_id: u8
    ) -> Vec<Entity> {
        let mut table_entities = Vec::with_capacity(self.domain_scores.len());
        for single_domain in &self.domain_scores {
            if single_domain.domain == "<PAD>" {
                continue;
            } else if single_domain.domain == "<UNKNOWN>" {
                let table_key = TableKey {
                    fields: vec![
                        KeyField {
                            field_id: table_info.get_key_id_by_name("pipe.Egress.unknown_domain_score", "model_id"),
                            match_type: Some(key_field::MatchType::Exact(Exact {
                                value: vec![model_id]
                            }))
                        }
                    ]
                };
                let table_data = TableData {
                    action_id: table_info.get_action_id_by_name("pipe.Egress.unknown_domain_score", "Egress.set_domain_scores_UNKNOWN"),
                    fields: vec![
                        DataField {
                            field_id: 1,
                            value: Some(data_field::Value::Stream(single_domain.scores[0].to_be_bytes().to_vec())) // <-- 16bit
                        },
                    ]
                };
                let table_entry = TableEntry {
                    table_id: table_info.get_table_by_name("pipe.Egress.unknown_domain_score"),
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
                table_entities.push(entity);
            } else {
                let table_key = TableKey {
                    fields: vec![
                        KeyField {
                            field_id: table_info.get_key_id_by_name("pipe.Egress.domain_hash2score", "model_id"),
                            match_type: Some(key_field::MatchType::Exact(Exact {
                                value: vec![model_id]
                            }))
                        },
                        KeyField {
                            field_id: table_info.get_key_id_by_name("pipe.Egress.domain_hash2score", "domain_hash"),
                            match_type: Some(key_field::MatchType::Exact(Exact {
                                value: single_domain.hash.to_be_bytes().to_vec()
                            }))
                        }
                    ]
                };
                let table_data = TableData {
                    action_id: table_info.get_action_id_by_name("pipe.Egress.domain_hash2score", "Egress.set_domain_scores"),
                    fields: vec![
                        DataField {
                            field_id: 1,
                            value: Some(data_field::Value::Stream(single_domain.scores[0].to_be_bytes().to_vec())) // <-- 16bit
                        },
                    ]
                };
                let table_entry = TableEntry {
                    table_id: table_info.get_table_by_name("pipe.Egress.domain_hash2score"),
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
                table_entities.push(entity);
            }
        }
        {
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Egress.score_threshold", "$REGISTER_INDEX"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: (model_id as u32).to_be_bytes().to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: 0,
                fields: vec![
                    DataField {
                        field_id: table_info.get_data_id_by_name("pipe.Egress.score_threshold", "Egress.score_threshold.f1"),
                        value: Some(data_field::Value::Stream(self.threshold.to_be_bytes().to_vec())) // <-- 16bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Egress.score_threshold"),
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
            table_entities.push(entity);
        }
        table_entities
    }

    pub fn remove_from_dataplane(
        &self,
        table_info: &P4TableInfo,
        model_id: u8
    ) -> Vec<Update> {
        let mut updates = vec![];
        for e in self.to_entities(table_info, model_id) {
            let update = Update {
				r#type: update::Type::Delete as _,
				entity: Some(e)
			};
            updates.push(update);
        }
        updates
    }
    pub fn write_to_dataplane(
        &self,
        table_info: &P4TableInfo,
        model_id: u8
    ) -> Vec<Update> {
        let mut updates = vec![];
        for e in self.to_entities(table_info, model_id) {
            let update = Update {
				r#type: update::Type::Insert as _,
				entity: Some(e)
			};
            updates.push(update);
        }
        updates
    }
}

struct MonitoredPDUSession {
    pub model_id: u8,
    pub detection_id: u16,
    pub created_at_ms: u64,
    pub last_detected_ms: u64,
    pub infected_counts: u64,
}

struct DomainWatcherSessions {
    sessions: HashMap<u32, MonitoredPDUSession>,
    id_gen: IDAllocator<u16>,
}

impl DomainWatcherSessions {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            id_gen: IDAllocator::new_with_counter(1)
        }
    }
}

struct DomainWatcherState {
    pub sessions: Arc<RwLock<DomainWatcherSessions>>,
}

impl DomainWatcherState {
    pub fn new() -> DomainWatcherState {
        Self {
            sessions: Arc::new(RwLock::new(DomainWatcherSessions::new()))
        }
    }
}

lazy_static! {
	static ref DOMAIN_WATCHER_STATES: DomainWatcherState = DomainWatcherState::new();
}


pub async fn add_monitored_ue(ma_id: u32, model_id: u8, cur_ts: u64, table_info: &P4TableInfo) -> Option<(u16, Vec<Update>)> {
    // auto assign detection_id
    let mut state_lock = DOMAIN_WATCHER_STATES.sessions.write().await;
    if !state_lock.sessions.contains_key(&ma_id) {
        let detection_id = state_lock.id_gen.allocate();
        if detection_id.is_err() {
            return None;
        }
        let detection_id = detection_id.unwrap();
        let session_state = MonitoredPDUSession {
            model_id,
            detection_id: detection_id,
            created_at_ms: cur_ts,
            last_detected_ms: 0,
            infected_counts: 0
        };
        state_lock.sessions.insert(ma_id, session_state);
        let mut updates = vec![];
        // step 1: add mapping table
        {
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Egress.set_domainwatcher_id", "meta.bridge.ma_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: ma_id.to_be_bytes()[1..].to_vec()
                        }))
                    }
                ]
            };
            let table_data = TableData {
                action_id: table_info.get_action_id_by_name("pipe.Egress.set_domainwatcher_id", "Egress.set_domain_watcher_detection_id"),
                fields: vec![
                    DataField {
                        field_id: 1,
                        value: Some(data_field::Value::Stream(detection_id.to_be_bytes().to_vec())) // <-- 16bit
                    },
                    DataField {
                        field_id: 2,
                        value: Some(data_field::Value::Stream(model_id.to_be_bytes().to_vec())) // <-- 2bit
                    },
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Egress.set_domainwatcher_id"),
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
            updates.push(update);
        }
        info!("adding monitored UE [MAID={}] with detection_id={} with model={}", ma_id, detection_id, model_id);
        Some((detection_id, updates))
    } else {
        None
    }
}

pub async fn del_monitored_ue(pdr_id: u32, table_info: &P4TableInfo) -> Option<(u16, Vec<Update>)> {
    let mut state_lock = DOMAIN_WATCHER_STATES.sessions.write().await;
    if let Some(old_session) = state_lock.sessions.remove(&pdr_id) {
        state_lock.id_gen.free(old_session.detection_id);
        let mut updates = vec![];
        // step 1: remove from mapping table
        {
            let table_key = TableKey {
                fields: vec![
                    KeyField {
                        field_id: table_info.get_key_id_by_name("pipe.Egress.set_domainwatcher_id", "meta.bridge.ma_id"),
                        match_type: Some(key_field::MatchType::Exact(Exact {
                            value: pdr_id.to_be_bytes()[1..].to_vec()
                        }))
                    }
                ]
            };
            let table_entry = TableEntry {
                table_id: table_info.get_table_by_name("pipe.Egress.set_domainwatcher_id"),
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
			let update = Update {
				r#type: update::Type::Delete as _,
				entity: Some(entity)
			};
            updates.push(update);
        }
        // step 2: set Register at detection_id to 0
        {
            for i in 0..4u16 {
                let table_key = TableKey {
                    fields: vec![
                        KeyField {
                            field_id: table_info.get_key_id_by_name(&format!("pipe.Egress.domain_scores_{}", i), "$REGISTER_INDEX"),
                            match_type: Some(key_field::MatchType::Exact(Exact {
                                value: (old_session.detection_id as u32).to_be_bytes().to_vec()
                            }))
                        }
                    ]
                };
                let table_data = TableData {
                    action_id: 0,
                    fields: vec![
                        DataField {
                            field_id: table_info.get_data_id_by_name(&format!("pipe.Egress.domain_scores_{}", i), &format!("Egress.domain_scores_{}.f1", i)),
                            value: Some(data_field::Value::Stream(0u16.to_be_bytes().to_vec())) // <-- 16bit
                        },
                    ]
                };
                let table_entry = TableEntry {
                    table_id: table_info.get_table_by_name(&format!("pipe.Egress.domain_scores_{}", i)),
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
                    r#type: update::Type::Modify as _,
                    entity: Some(entity)
                };
                updates.push(update);
            }
        }
        Some((old_session.detection_id, updates))
    } else {
        None
    }
}

// pub fn generate_detection_trigger_packet(pdr_id: u32) -> Vec<u8> {
    
// }

// fn detection_thread(cpu_if_name: String, period_ms: u64) {

// }
