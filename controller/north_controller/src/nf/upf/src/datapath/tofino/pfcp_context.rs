use std::collections::HashMap;

use linear_map::set::LinearSet;
use log::info;
use models::QosRequirement;

use crate::datapath::{FlattenedQER, FlattenedPacketPipeline, PacketPipeline};

use super::{action_tables::MatchActionPipeline, TofinoBackendError, bfruntime::{bfrt::Update, P4TableInfo}, utils, table_templates::{self, MatchInstance}, match_action_id::ActionInstance};


pub type pdr_id_t = u16;

#[derive(Debug, Clone)]
pub struct TofinoPdrPipeline {
    pub pdr_id: pdr_id_t,
    pub matches: Vec<MatchInstance>,
    pub precedence: i32,
    pub action: ActionInstance,
    pub maid: Option<u32>,
    pub linked_urrs: Vec<u32>,
    pub linked_global_qer_id: Option<u16>,
}

pub struct PfcpContext {
    seid: u64,

    /// Stored To delete PDR_IDs
    pub to_del_pdr_ids: LinearSet<pdr_id_t>,

    /// Pipelines active in ASIC
    pub pipelines: linear_map::LinearMap<pdr_id_t, TofinoPdrPipeline>,
    /// Map from local QER_ID to global_qer_id
    pub qers: linear_map::LinearMap<u32, (u16, FlattenedQER)>,

    // transaction context
    pub pipelines_in_transaction: linear_map::LinearMap<pdr_id_t, TofinoPdrPipeline>,
}

impl PfcpContext {
    pub fn new(seid: u64) -> Self {
        Self {
            seid,
            to_del_pdr_ids: LinearSet::new(),
            pipelines: linear_map::LinearMap::new(),
            qers: linear_map::LinearMap::new(),
            pipelines_in_transaction: linear_map::LinearMap::new(),
        }
    }
}

impl PfcpContext {
    pub fn prepare_update(&mut self) {
        self.pipelines_in_transaction = self.pipelines.clone();
    }
    pub fn merge_updates(&mut self, deleted_pipes: Vec<pdr_id_t>, updated_or_new_pipes: linear_map::LinearMap<pdr_id_t, PacketPipeline>) -> Result<(), TofinoBackendError> {
        for d in deleted_pipes.iter() {
            self.pipelines_in_transaction.remove(d);
        }
        for (pdr_id, pipeline) in updated_or_new_pipes.iter() {
            let expanded_pipelines = utils::flatten_packet_pipeline(pipeline);
            assert!(expanded_pipelines.len() > 0);
            let mut expanded_ma_pipelines = Vec::with_capacity(expanded_pipelines.len());
            for pipeline in expanded_pipelines.iter() {
                expanded_ma_pipelines.push(table_templates::match_pipeline(pipeline)?);
            }
            let action = expanded_ma_pipelines[0].1;
            let all_matches = expanded_ma_pipelines.into_iter().map(|(f, _)| f).collect::<Vec<_>>();
            let old_maid = if let Some(active_pipe) = self.pipelines.get(pdr_id) {
                active_pipe.maid.clone()
            } else {
                None
            };
            let linked_global_qer_id = if let Some(qer) = &pipeline.qer_id {
                if let Some((global_qer_id, _)) = self.qers.get(&qer.rule_id) {
                    Some(*global_qer_id)
                } else {
                    return Err(TofinoBackendError::UnknownQerId);
                }
            } else {
                None
            };
            let tofino_pdr_pipe = TofinoPdrPipeline {
                pdr_id: *pdr_id,
                matches: all_matches,
                precedence: pipeline.precedence.precedence,
                action: action,
                maid: old_maid,
                linked_urrs: pipeline.urr_ids.iter().map(|f| f.rule_id).collect::<Vec<_>>(),
                linked_global_qer_id,
            };
            self.pipelines_in_transaction.insert(*pdr_id, tofino_pdr_pipe);
		}
        Ok(())
    }

    pub fn activate_pipelines(&mut self, touched: linear_map::LinearMap<pdr_id_t, u32>, untouched: Vec<pdr_id_t>) {
        // println!("===activate_pipelines===");
        // println!("touched={:?}", touched);
        // println!("untouched={:?}", untouched);
        self.pipelines = self.pipelines_in_transaction.clone();
        for (pdr_id, maid) in touched.into_iter() {
            self.pipelines.get_mut(&pdr_id).unwrap().maid.replace(maid);
        }
        // println!("self.pipelines");
        // println!("{:#?}", self.pipelines);
    }
}

impl PfcpContext {
    pub fn change_maid(&mut self, changes: Vec<(u32, u32)>) -> Vec<Update> {
        todo!()
    }
}
