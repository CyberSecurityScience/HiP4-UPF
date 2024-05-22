use log::warn;

use super::{pfcp_context::PfcpContext, action_tables::MatchActionPipeline};


impl PfcpContext {
	pub fn reorder_ma_rules(&mut self) {
		let mut expanded_count = 0usize;
		for (_, pipeline) in self.pipelines_in_transaction.iter() {
			expanded_count += pipeline.matches.len();
		}
		let mut all_matches = Vec::with_capacity(expanded_count);
		for (_, pipeline) in self.pipelines_in_transaction.iter_mut() {
			for m in pipeline.matches.iter_mut() {
				all_matches.push(m);
			}
		}
		all_matches.retain(|f| !f.is_empty());
		// sort priority from high to low
		all_matches.sort_by(|a, b| b.priority().cmp(&a.priority()));
		let mut out_of_order = false;
		for i in 1..all_matches.len() {
			if all_matches[i - 1].get_p4_table_order() > all_matches[i].get_p4_table_order() {
				out_of_order = true;
			}
		}
		if out_of_order {
			warn!("Out of order tables used");
			let table_order = 4;
			for it in all_matches.iter_mut() {
				**it = it.to_p4_table_order(table_order);
			}
		}
	}
}
