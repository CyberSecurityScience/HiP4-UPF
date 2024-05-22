use std::{collections::{HashSet, HashMap}, sync::Arc, iter::FromIterator, thread::JoinHandle};

use itertools::Itertools;
use linear_map::{LinearMap, set::LinearSet};
use log::info;
use models::QosRequirement;
use tokio::sync::{RwLock, mpsc::{Receiver, Sender}};

use crate::datapath::tofino::{bfruntime::bfrt::{TableEntry, table_entry, Entity, Update, entity, update, key_field::Exact}, upf_driver_interface};

use super::{table_templates::{MatchInstance, PipelineTemplateDirection}, match_action_id::{ActionInstance, MaidTree, ActionTableOperation, ActionInstanceCount}, bfruntime::{P4TableInfo, bfrt::{TableKey, KeyField, key_field::{self, Ternary}, TableData, DataField, data_field}, BFRuntime}, pfcp_context::{pdr_id_t, TofinoPdrPipeline}, difference_generator::{ToDelPipeline, ToModPipeline, ToAddPipeline}, upf_driver_interface::PDRUpdate, usage_reporting::PdrUsageReportingAssociation};

#[derive(Debug)]
pub enum ActionTableError {
    CannotBreakRangeAnymore,
    TreeTooDeep,
    InsufficientActionTableCapacity,
    InsufficientMAID,
}

impl std::fmt::Display for ActionTableError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f,"{:?}", *self)
	}
}

impl std::error::Error for ActionTableError {
	fn description(&self) -> &str {
		"ERROR TODO"
	}
}

#[derive(Debug, Clone)]
pub struct MatchActionPipeline {
    pub priority: i32,
    pub pdr_id: u16,
    pub match_instance: MatchInstance,
    pub action_instance: ActionInstance,
    pub urr_ids: Vec<u32>,
    pub linked_qer_id: Option<u16>,
    pub match_action_id: Option<u32>,
}

impl MatchActionPipeline {
    pub fn is_active(&self) -> bool {
        self.match_action_id.is_some()
    }
}

pub struct GlobalActionTableContext {
    maid_tree: Arc<RwLock<MaidTree>>
}

impl GlobalActionTableContext {
    pub async fn remove_maids(&self, ids: &Vec<u32>, info: &P4TableInfo) -> Vec<Update> {
        // info!("freeing MAIDS: {:?}", ids);
        let mut tree = self.maid_tree.write().await;
        tree.action_tables_transaction_begin();
        for id in ids {
            tree.free_maid(*id as _);
        }
        GlobalActionTableContext::transaction_commit(&mut tree, info, vec![]).unwrap()
    }

    pub fn new(initial_action_tuples: Vec<ActionInstanceCount>, bfrt: Arc<RwLock<Option<BFRuntime>>>, info: &P4TableInfo) -> Result<(Self, Vec<Update>, Sender<u32>), ActionTableError> {
        let (mut maid_tree, ops) = MaidTree::new(initial_action_tuples, 24, 2048, 2048, 7168)?;
        GlobalActionTableContext::transaction_begin(&mut maid_tree);
        let table_updates = GlobalActionTableContext::transaction_commit(&mut maid_tree, info, ops)?;
        let tree = Arc::new(RwLock::new(maid_tree));
        let tree2 = tree.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(4000);
        let s = Self {
            maid_tree: tree
        };
        Ok((s, table_updates, sender))
    }
    pub fn transaction_begin(maid_tree: &mut MaidTree) {
        maid_tree.action_tables_transaction_begin();
    }
    pub fn transaction_commit(maid_tree: &mut MaidTree, info: &P4TableInfo, mut additional_ops: Vec<ActionTableOperation>) -> Result<Vec<Update>, ActionTableError> {
        let mut ops = maid_tree.action_tables_transaction_commit()?;
        ops.append(&mut additional_ops);
        let mut updates = Vec::with_capacity(ops.len());
        for op in ops.into_iter() {
            let (entry, table_op) = match op {
                ActionTableOperation::Modify(_) => unreachable!(),
                ActionTableOperation::Insert(entry) => {
                    (entry, update::Type::Insert)
                },
                ActionTableOperation::Delete(entry) => {
                    (entry, update::Type::Delete)
                },
            };
            let (key, data) = match entry.action_tuple {
                ActionInstance::Nop => continue,
                ActionInstance::Decap(nocp) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.ul_mark_tos_table_forward_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(vec![0u8])) // mark DSCP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![0u8])) // DSCP
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::DecapMarkDSCP(nocp, dscp) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.ul_mark_tos_table_forward_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(vec![1u8])) // mark DSCP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![dscp])) // DSCP
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::EncapDl(nocp, ip, qfi) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.dl_to_N3N9_table_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(ip.octets().to_vec())) // IP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![qfi])) // QFI
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Buffer
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::EncapUl(nocp, ip, qfi) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.ul_to_N3N9_table_table_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(0u32.to_be_bytes().to_vec())) // IP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![qfi])) // QFI
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::Buffer(nocp) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.dl_to_N3N9_table_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(0u32.to_be_bytes().to_vec())) // IP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![0u8])) // QFI
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![1u8])) // Buffer
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::DropDl(nocp) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.dl_to_N3N9_table_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(0u32.to_be_bytes().to_vec())) // IP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![0u8])) // QFI
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![0u8])) // Buffer
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![1u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
                ActionInstance::DropUl(nocp) => {
                    let table_key = TableKey {
                        fields: vec![
                            KeyField {
                                field_id: info.get_key_id_by_name(entry.action_tuple.which_table().p4_table_name(), "meta.ma_id"),
                                match_type: Some(key_field::MatchType::Ternary(Ternary {
                                    value: (entry.value as u32).to_be_bytes()[1..].to_vec(),
                                    mask: (entry.mask as u32).to_be_bytes()[1..].to_vec()
                                }))
                            }
                        ]
                    };
                    let table_data = TableData {
                        action_id: info.get_action_id_by_name(entry.action_tuple.which_table().p4_table_name(), "Ingress.ul_mark_tos_table_forward_v4"),
                        fields: vec![
                            DataField {
                                field_id: 1,
                                value: Some(data_field::Value::Stream(vec![0u8])) // mark DSCP
                            },
                            DataField {
                                field_id: 2,
                                value: Some(data_field::Value::Stream(vec![0u8])) // DSCP
                            },
                            DataField {
                                field_id: 3,
                                value: Some(data_field::Value::Stream(vec![(nocp as u8)])) // NoCP
                            },
                            DataField {
                                field_id: 4,
                                value: Some(data_field::Value::Stream(vec![1u8])) // Drop
                            }
                        ]
                    };
                    (table_key, table_data)
                },
            };
            let table_entry = TableEntry {
                table_id: info.get_table_by_name(entry.action_tuple.which_table().p4_table_name()),
                data: Some(data),
                is_default_entry: false,
                table_read_flag: None,
                table_mod_inc_flag: None,
                entry_tgt: None,
                table_flags: None,
                value: Some(table_entry::Value::Key(key)),
            };
            let entity = Entity {
                entity: Some(entity::Entity::TableEntry(table_entry))
            };
            let update = Update {
                r#type: table_op as _,
                entity: Some(entity)
            };
            updates.push(update);
        }
        Ok(updates)
    }
    pub async fn update<'a>(
        &'a self,
        info: &P4TableInfo,
        seid: u64,
        to_del: Vec<ToDelPipeline<'a>>,
        mut to_mod: Vec<ToModPipeline<'a>>,
        mut to_add: Vec<ToAddPipeline<'a>>,
        untouched_pipelines: Vec<TofinoPdrPipeline>,
        piggybacked_maid_deletions: Vec<u32>
    ) -> Result<(LinearMap<u16, u32>, Vec<Update>, Vec<PDRUpdate>, Vec<PdrUsageReportingAssociation>, Vec<u32>, Vec<u32>), ActionTableError> {
        // println!("=======Rule update for [SEID={}]=======", seid);
        // println!("To Delete");
        // println!("{:?}", to_del);
        // println!("To Modify");
        // println!("{:?}", to_mod);
        // println!("To Insert");
        // println!("{:?}", to_add);

        let mut modified_pdr_id_to_maid_map = LinearMap::with_capacity(to_add.len() + to_mod.len());
        let mut new_maids = vec![];
        let mut to_del_maids = vec![];

        // step 1: update tree
        let mut grpc_table_updates = {
            // hold MAID tree lock as short as possible so other threads can update the tree
            let mut maid_tree = self.maid_tree.write().await;
            Self::transaction_begin(&mut maid_tree);
            for maid in piggybacked_maid_deletions.iter() {
                maid_tree.free_maid(*maid as _);
            }
            for item in to_mod.iter_mut() {
                if let Some(action_update) = &item.update_action {
                    let new_id = maid_tree.allocate_maid(action_update.new_action)? as u32;
                    new_maids.push(new_id);
                    modified_pdr_id_to_maid_map.insert(item.pdr_id, new_id);
                    item.allocated_maid = Some(new_id);
                } else if item.update_matches.is_some() {
                    // action not update but match updated, still need new MAID for follow up remove-then-insert operation
                    let new_id = maid_tree.allocate_maid(*item.old_action)? as u32;
                    new_maids.push(new_id);
                    modified_pdr_id_to_maid_map.insert(item.pdr_id, new_id);
                    item.allocated_maid = Some(new_id);
                }
            }
            for item in to_add.iter_mut() {
                let new_id = maid_tree.allocate_maid(*item.action)? as u32;
                new_maids.push(new_id);
                item.allocated_maid = Some(new_id);
                modified_pdr_id_to_maid_map.insert(item.pdr_id, new_id);
            }
            Self::transaction_commit(&mut maid_tree, info, vec![])?
        };

        // step 1.1 generate usage reporting assocaitions
        let mut all_associations = Vec::with_capacity(10);
        for pipe in untouched_pipelines.into_iter() {
            if pipe.maid.is_some() {
                all_associations.push(PdrUsageReportingAssociation {
                    pdr_id: pipe.pdr_id,
                    maid: pipe.maid.unwrap(),
                    is_uplink: pipe.matches[0].get_direction() == PipelineTemplateDirection::Uplink,
                    urr_ids: LinearSet::from_iter(pipe.linked_urrs.iter().cloned()),
                });
            }
        }
        for item in to_mod.iter() {
            let maid = item.allocated_maid.unwrap_or(item.maid);
            all_associations.push(PdrUsageReportingAssociation {
                pdr_id: item.pdr_id,
                maid: maid,
                is_uplink: item.matches[0].get_direction() == PipelineTemplateDirection::Uplink,
                urr_ids: LinearSet::from_iter(item.urr_ids.iter().cloned()),
            });
        }
        for item in to_add.iter() {
            let maid = item.allocated_maid.unwrap();
            all_associations.push(PdrUsageReportingAssociation {
                pdr_id: item.pdr_id,
                maid: maid,
                is_uplink: item.matches[0].get_direction() == PipelineTemplateDirection::Uplink,
                urr_ids: LinearSet::from_iter(item.urr_ids.iter().cloned()),
            });
        }

        let mut upf_driver_table_updates = vec![];
        //  update usage reporting AFTER PDR update succeed
        // step 2: update match table
        for item in to_del.into_iter() {
            for m in item.matches {
                if let Some(update) = m.generate_upf_driver_update_remove(item.maid) {
                    upf_driver_table_updates.push(update);
                }
                to_del_maids.push(item.maid);
            }
        }

        for item in to_mod.into_iter() {
            // if update only affects QER_ID, we use Update, otherwise we delete and insert
            let global_qer_id = if let Some(update_qer) = &item.update_qer {
                update_qer.new_global_qer_id
            } else {
                item.global_qer_id
            };
            if (item.update_qer.is_some() && item.update_matches.is_none() && item.update_action.is_none()) {
                for m in item.old_matches {
                    if let Some(update) = m.generate_upf_driver_update_update(item.maid, item.maid, global_qer_id) {
                        upf_driver_table_updates.push(update);
                    }
                }
            } else {
                let new_maid = item.allocated_maid.unwrap_or(item.maid);
                if let Some(match_update) = item.update_matches {
                    // if we perform match update
                    // we delete old then insert new
                    // Action is already updated in step 1
                    for m in match_update.old_matches {
                        if let Some(update) = m.generate_upf_driver_update_remove(item.maid) {
                            upf_driver_table_updates.push(update);
                        }
                        to_del_maids.push(item.maid);
                    }
                    for m in match_update.new_matches {
                        // TODO: require new PerMaidURR
                        if let Some(update) = m.generate_upf_driver_update_insert(new_maid, global_qer_id) {
                            upf_driver_table_updates.push(update);
                        }
                    }
                } else if let Some(new_action) = item.update_action {
                    // we only update action, we update MAID
                    for m in item.old_matches {
                        // TODO: require new PerMaidURR
                        if let Some(update) = m.generate_upf_driver_update_update(item.maid, new_maid, global_qer_id) {
                            upf_driver_table_updates.push(update);
                        }
                    }
                }
                // URR is updated by the usage reporting module, not here
            }
            
        }
        for item in to_add.into_iter() {
            let maid = item.allocated_maid.unwrap();
            for m in item.matches {
                // TODO: require new PerMaidURR
                if let Some(update) = m.generate_upf_driver_update_insert(maid, item.global_qer_id) {
                    upf_driver_table_updates.push(update);
                }
            }
        }
        //info!("upf_driver_table_updates: {:#?}", upf_driver_table_updates);
        Ok((modified_pdr_id_to_maid_map, grpc_table_updates, upf_driver_table_updates, all_associations, new_maids, to_del_maids))
    }
    pub async fn recycle_maids(&self, maids: Vec<u32>, info: &P4TableInfo) -> Vec<Update> {
        let mut maid_tree = self.maid_tree.write().await;
        Self::transaction_begin(&mut maid_tree);
        for maid in maids {
            maid_tree.free_maid(maid as _);
        }
        Self::transaction_commit(&mut maid_tree, info, vec![]).unwrap()
    }
}
