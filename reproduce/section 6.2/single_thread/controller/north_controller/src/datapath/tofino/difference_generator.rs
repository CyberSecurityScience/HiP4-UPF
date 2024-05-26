use std::iter::FromIterator;

use linear_map::set::LinearSet;

use crate::utils::linear_set_difference;

use super::{pfcp_context::{PfcpContext, pdr_id_t, TofinoPdrPipeline}, action_tables::{MatchActionPipeline}, table_templates::MatchInstance, match_action_id::ActionInstance};

#[derive(Debug)]
pub struct ToDelPipeline<'a> {
    pub pdr_id: pdr_id_t,
    pub matches: &'a Vec<MatchInstance>,
    pub action: &'a ActionInstance,
    pub maid: u32
}

#[derive(Debug)]
pub struct ToModPipelineUpdateMatches<'a> {
    pub old_matches: &'a Vec<MatchInstance>,
    pub new_matches: &'a Vec<MatchInstance>,
}
#[derive(Debug)]
pub struct ToModPipelineUpdateAction {
    pub old_action: ActionInstance,
    pub new_action: ActionInstance,
}

#[derive(Debug)]
pub struct ToModPipelineUpdateQer {
    pub old_global_qer_id: u16,
    pub new_global_qer_id: u16
}

#[derive(Debug)]
pub struct ToModPipelineUpdateUrr<'a> {
    pub old_urr_ids: &'a Vec<u32>,
    pub new_urr_ids: &'a Vec<u32>
}

#[derive(Debug)]
pub struct ToModPipeline<'a> {
    pub pdr_id: pdr_id_t,
    pub maid: u32,
    pub global_qer_id: u16,
    pub urr_ids: &'a Vec<u32>,
    pub matches: &'a Vec<MatchInstance>,
    pub old_matches: &'a Vec<MatchInstance>,
    pub update_matches: Option<ToModPipelineUpdateMatches<'a>>,
    pub update_action: Option<ToModPipelineUpdateAction>,
    pub update_qer: Option<ToModPipelineUpdateQer>,
    pub update_urr: Option<ToModPipelineUpdateUrr<'a>>,
    pub allocated_maid: Option<u32>
}

#[derive(Debug)]
pub struct ToAddPipeline<'a> {
    pub pdr_id: pdr_id_t,
    pub matches: &'a Vec<MatchInstance>,
    pub action: &'a ActionInstance,
    pub global_qer_id: u16,
    pub urr_ids: &'a Vec<u32>,
    pub allocated_maid: Option<u32>
}


impl PfcpContext {
    pub fn find_difference<'a>(&'a mut self) -> (Vec<ToDelPipeline<'a>>, Vec<ToModPipeline<'a>>, Vec<ToAddPipeline<'a>>, Vec<pdr_id_t>, Vec<TofinoPdrPipeline>) {
        let old_pdr_ids = LinearSet::from_iter(self.pipelines.keys().cloned());
        let new_pdr_ids = LinearSet::from_iter(self.pipelines_in_transaction.keys().cloned());
        let (added, unchanged, removed) = linear_set_difference(&old_pdr_ids, &new_pdr_ids);
        // println!("===find_difference===");
        // println!("=self.pipelines=");
        // println!("{:#?}", self.pipelines);
        // println!("=self.pipelines_in_transaction=");
        // println!("{:#?}", self.pipelines_in_transaction);

        let mut to_del = Vec::with_capacity(removed.len());
        for pdr_id in removed.into_iter() {
            let p1 = self.pipelines.get(&pdr_id);
            let pipe = p1.as_ref().unwrap();
            to_del.push(
                ToDelPipeline {
                    pdr_id: pdr_id,
                    matches: &pipe.matches,
                    action: &pipe.action,
                    maid: *pipe.maid.as_ref().unwrap(),
                }
            );
        }

        let mut to_add = Vec::with_capacity(added.len());
        for pdr_id in added.into_iter() {
            let p1 = self.pipelines_in_transaction.get(&pdr_id);
            let pipe = p1.as_ref().unwrap();
            if pipe.action != ActionInstance::Nop {
                to_add.push(
                    ToAddPipeline {
                        pdr_id: pdr_id,
                        matches: &pipe.matches,
                        action: &pipe.action,
                        global_qer_id: pipe.linked_global_qer_id.unwrap_or(0),
                        urr_ids: &pipe.linked_urrs,
                        allocated_maid: None
                    }
                );
            }
        }

        let mut to_mod = Vec::with_capacity(unchanged.len() + 1);
        let mut untouched = Vec::with_capacity(unchanged.len() + 1);
        let mut untouched_pipelines = Vec::with_capacity(unchanged.len() + 1);
        for pdr_id in unchanged.into_iter() {
            let old_pipe = self.pipelines.get(&pdr_id).unwrap();
            let new_pipe = self.pipelines_in_transaction.get(&pdr_id).unwrap();
            if old_pipe.action == ActionInstance::Nop && new_pipe.action == ActionInstance::Nop {
                untouched.push(old_pipe.pdr_id);
                untouched_pipelines.push(old_pipe.clone());
                continue;
            }
            if new_pipe.action == ActionInstance::Nop {
                if new_pipe.maid.is_some() {
                    // from a non-Nop pipeline to a Nop pipeline is equivalent to deleting it
                    to_del.push(
                        ToDelPipeline {
                            pdr_id: pdr_id,
                            matches: &new_pipe.matches,
                            action: &new_pipe.action,
                            maid: *new_pipe.maid.as_ref().unwrap(),
                        }
                    );
                }
                continue;
            }
            if old_pipe.action == ActionInstance::Nop || old_pipe.maid.is_none() {
                // from a Nop pipeline to a non-Nop pipeline is equivalent to adding it
                to_add.push(
                    ToAddPipeline {
                        pdr_id: pdr_id,
                        matches: &new_pipe.matches,
                        action: &new_pipe.action,
                        global_qer_id: new_pipe.linked_global_qer_id.unwrap_or(0),
                        urr_ids: &new_pipe.linked_urrs,
                        allocated_maid: None
                    }
                );
                continue;
            }
            let old_matches = LinearSet::from_iter(old_pipe.matches.iter());
            let new_matches = LinearSet::from_iter(new_pipe.matches.iter());
            let update_matches = if old_matches != new_matches {
                Some(ToModPipelineUpdateMatches { old_matches: &old_pipe.matches, new_matches: &new_pipe.matches  })
            } else {
                None
            };
            let update_action = if old_pipe.action != new_pipe.action {
                Some(ToModPipelineUpdateAction { old_action: old_pipe.action, new_action: new_pipe.action })
            } else {
                None
            };
            let old_urr_ids = LinearSet::from_iter(old_pipe.linked_urrs.iter());
            let new_urr_ids = LinearSet::from_iter(new_pipe.linked_urrs.iter());
            let update_urr_ids = if old_urr_ids != new_urr_ids {
                Some(ToModPipelineUpdateUrr { old_urr_ids: &old_pipe.linked_urrs, new_urr_ids: &new_pipe.linked_urrs  })
            } else {
                None
            };
            let update_qer = if old_pipe.linked_global_qer_id != new_pipe.linked_global_qer_id {
                Some(ToModPipelineUpdateQer { old_global_qer_id: old_pipe.linked_global_qer_id.unwrap_or(0), new_global_qer_id: new_pipe.linked_global_qer_id.unwrap_or(0) })
            } else {
                None
            };
            if update_matches.is_some() || update_action.is_some() || update_urr_ids.is_some() || update_qer.is_some() {
                to_mod.push(
                    ToModPipeline {
                        pdr_id,
                        global_qer_id: new_pipe.linked_global_qer_id.unwrap_or(0),
                        urr_ids: &new_pipe.linked_urrs,
                        matches: &new_pipe.matches,
                        old_matches: &old_pipe.matches,
                        maid: new_pipe.maid.unwrap(),
                        update_matches,
                        update_action,
                        update_qer,
                        update_urr: update_urr_ids,
                        allocated_maid: None
                    }
                );
            } else {
                untouched.push(old_pipe.pdr_id);
                untouched_pipelines.push(old_pipe.clone());
            }
        }
        (to_del, to_mod, to_add, untouched, untouched_pipelines)
    }
}
