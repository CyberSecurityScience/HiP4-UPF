use crate::datapath::tofino::match_action_id::ActionInstance;

use super::MatchInstance;


pub trait ReorderMatchActionInstance {
    fn to_p4_table_order(&self, order: i32) -> MatchInstance;
}
