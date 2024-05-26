use std::{collections::{HashMap, HashSet, VecDeque}, iter::FromIterator, sync::Arc, convert::TryInto, thread::JoinHandle};

use libpfcp::{
	messages::{CreateFAR, CreatePDR, CreateQER, UpdateFAR, UpdatePDR, UpdateQER, ForwardingParameters},
	models::{FAR_ID, PDR_ID, QER_ID, ApplyAction, MeasurementMethod, ReportingTriggers, VolumeThreshold, VolumeThresholdFlags, URR_ID},
};
use log::{info, error, warn};
use once_cell::sync::OnceCell;
use tokio::sync::{RwLock, Mutex, mpsc::Sender};

use crate::{datapath::{tofino::{bfruntime::bfrt::*, table_templates::PipelineTemplateDirection, action_tables::MatchActionPipeline}, FlattenedURR}, context::get_async_runtime};

use self::{table_templates::{MatchInstance, match_pipeline}, pfcp_context::PfcpContext, bfruntime::{P4TableInfo, bfrt::key_field::Exact}};

use super::{BackendInterface, BackendReport, PacketPipeline, FlattenedPacketPipeline, FlattenedQER, OperationPerfStats};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
mod bfruntime_models;
mod bfruntime;

mod requests_generated;
mod response_generated;

mod utils;
mod common_tables;
mod routing;
mod pfcp_context;
mod rule_optimization;
mod difference_generator;
mod table_templates;
mod action_tables;
mod match_action_id;
mod usage_reporting;
mod qer;
mod upf_driver_interface;
pub mod qos;

mod domain_watcher;

#[derive(Debug)]
pub enum TofinoBackendError {
	UnsupportedSourceInterface,
	NoForwardingTemplateFound,
	Todo,
	TooManyUpfSelfId,
	TooManyNexthopIPs,
	TeidOutOfRange,
	ConflictingQuotaFAR,
	UnknownQerId,
	InsufficientTableCapacity,
	InsufficientGBRFlowCapacity,
	InsufficientUsageReportingCapacity,
	InsufficientBandwidthCapacity,
	InsufficientActionTableCapacity,
	UnforeseenBfrtError
}
impl std::fmt::Display for TofinoBackendError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f,"{:?}", *self)
	}
}

impl std::error::Error for TofinoBackendError {
	fn description(&self) -> &str {
		"ERROR TODO"
	}
}

pub struct TofinoBackendSingleThreadContext {
	global_qer_context: qer::GlobalQerContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URRTestAuxSettings {
	ema_value: f64,
	countdown_ms: i64,
	allow_neg_countdown: bool,
	enable_volume_estimation: bool,
	enable_auto_countdown: bool,
	auto_countdown_update_freq_ms: i64,
	auto_countdown_offset: i64,
	enable_delayed_countdown: bool,
	enter_slow_pull_mode_est_pull_distance: i32,
	enter_slow_pull_mode_rounds: i32,
	slow_pull_mode_delayed_rounds: i32,
	pull_round_time_ms: i64,
	max_update_delay_ms: u64
}

unsafe impl Send for TofinoBackendSingleThreadContext {}
unsafe impl Sync for TofinoBackendSingleThreadContext {}

pub struct TofinoBackend {
	pub bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>,
	pfcp_contexts: tokio::sync::Mutex<HashMap<u64, Arc<RwLock<PfcpContext>>>>,
	global_contexts: Arc<Mutex<TofinoBackendSingleThreadContext>>,
	global_action_table_context: action_tables::GlobalActionTableContext,
	table_info: P4TableInfo,
	callback_thread: JoinHandle<()>,
	urr_aux: URRTestAuxSettings,
	domain_watcher_android: Option<domain_watcher::DomainWatcherModel>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SendEndMarkerRequest {
	pub src_ip: std::net::IpAddr,
	pub dst_ip: std::net::IpAddr,
	pub dst_teid: u32
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SendBufferedDataRequest {
	pub seid: u64,
	pub pdr_id: u16,
	pub drop: bool
}



impl TofinoBackend {
	async fn next_squence(&self) -> u64 {
		0 // TODO: used to ensure table updates are all in order when batching update requests even in the face of out of order TofinoBackend procedure completions
	}
}

enum CallbackContext {
	CounterValues((bool, u64, Vec<response_generated::upfdriver::response::CounterValueArgs>)),
	ReclaimedMAIDs(Vec<u32>)
}

fn create_callback_handler_task(recv: std::sync::mpsc::Receiver<CallbackContext>, ur_tx: Sender<BackendReport>) -> JoinHandle<()> {
	std::thread::spawn(|| {
		let runtime = tokio::runtime::Builder::new_current_thread().thread_name("Tofino UPF callback handler thread").enable_time().build().unwrap();
		//let runtime = get_async_runtime();
		runtime.block_on(async move {
			while let Ok(content) = recv.recv() {
				match content {
					CallbackContext::CounterValues((is_eg, ts, values)) => {
						usage_reporting::push_counter_values(is_eg, values, &ur_tx, ts).await;
					},
					CallbackContext::ReclaimedMAIDs(maids) => {
						let l = GLOBAL_MAID_RECLAIM_QUEUE.get().unwrap();
						let mut guard = l.write().await;
						guard.extend(maids.iter());
					},
				}
			}
		});
	})
}

static GLOBAL_MAID_RECLAIM_QUEUE: OnceCell<std::sync::Arc<tokio::sync::RwLock<VecDeque<u32>>>> = OnceCell::new();
static GLOBAL_CALLBACK_SENDER: OnceCell<std::sync::mpsc::Sender<CallbackContext>> = OnceCell::new();

fn counter_value_cb(v: &flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<response_generated::upfdriver::response::CounterValue<'_>>>, is_ig: bool, ts: u64) {
	let values = v
		.iter()
		.map(|v| {
			response_generated::upfdriver::response::CounterValueArgs {
				ma_id: v.ma_id(),
				bytes: v.bytes(),
				pkts: v.pkts()
			}
		})
		.collect::<Vec<_>>();
	GLOBAL_CALLBACK_SENDER.get().unwrap().send(CallbackContext::CounterValues((!is_ig, ts, values))).unwrap();
}

fn ma_id_reclaim_cb(v: &flatbuffers::Vector<'_, u32>) {
    let values = v
		.iter()
		.map(|v| {
			v
		})
		.collect::<Vec<_>>();
    GLOBAL_CALLBACK_SENDER.get().unwrap().send(CallbackContext::ReclaimedMAIDs(values)).unwrap();
}

#[async_trait]
impl BackendInterface for TofinoBackend {
	async fn new(settings: super::BackendSettings) -> Box<dyn BackendInterface + Sync + Send>
	where
		Self: Sized,
	{
		info!("Backend: TofinoBackend::new");
		// step 1: connect to gRPC
		let mut client_id = 0;
		let mut bfrt = {
			loop {
				match bfruntime::BFRuntime::new(settings.target_addr.clone(), client_id, 0).await {
					Ok(c) => {
						break c;
					}
					Err(e) => {
						warn!("{:?}", e);
					}
				};
				client_id += 1;
				if client_id > 10 {
					panic!("Failed to create BfRuntime after 10 tries");
				}
			}
		};
		if client_id != 0 {
			warn!("Connected to Tofino swich driver using client_id={}", client_id);
		}
		info!("Connected to switch via BFRT");
		// info!("Resetting all tables");
		// bfrt.reset_all_tables().await;
		// step 2: populate common tables
		info!("Populating Tofino tables");
		let non_p4_tables = common_tables::populate_cpu_pcie_port(&bfrt.tofino_table_info, settings.cpu_pcie_port);
		bfrt.write_update_no_transaction(non_p4_tables).await.unwrap();
		// step 3: populate UPF table
		let upf_ip = match &settings.upf_ip {
			std::net::IpAddr::V4(x) => *x,
			std::net::IpAddr::V6(_) => panic!("IPv6 not supported!"),
		};
		let upf_ip_updates = common_tables::populate_upf_self_ip(&bfrt.table_info, upf_ip);
		bfrt.write_update_no_transaction(upf_ip_updates).await.unwrap();
		info!("Populating routing tables");
		// step 4: populate routing table
		let routing_updates = routing::populate_routing_table(&bfrt.table_info, &settings.routing);
		bfrt.write_update_no_transaction(routing_updates).await.unwrap();

		// info!("Populating DomainWatcher tables");
		// let mut domain_watcher_updates = domain_watcher::populate_tld_table("tlds.txt", &bfrt.table_info);
		// let dw_model_android = domain_watcher::DomainWatcherModel::new("domains.txt");
		// domain_watcher_updates.append(&mut dw_model_android.write_to_dataplane(&bfrt.table_info, 0));
		// bfrt.write_update_no_transaction(domain_watcher_updates).await.unwrap();

		info!("Connecting to upf_driver");
		upf_driver_interface::init("/tmp/upf-cpp.sock".to_owned(), "/tmp/upf-rust.sock".to_owned());
		let (cb_send, cb_recv) = std::sync::mpsc::channel();
		let maid_queue = std::sync::Arc::new(tokio::sync::RwLock::new(VecDeque::new()));
		GLOBAL_MAID_RECLAIM_QUEUE.set(maid_queue).unwrap();
		GLOBAL_CALLBACK_SENDER.set(cb_send).unwrap();
		let ur_tx_cloned = settings.ur_tx.clone();
		let callback_thread = create_callback_handler_task(cb_recv, ur_tx_cloned);
    	upf_driver_interface::set_callbacks(counter_value_cb, ma_id_reclaim_cb);

		let table_info = bfrt.table_info.clone();

		let bfrt_arc = Arc::new(RwLock::new(Some(bfrt)));
		
		let (
			action_table,
			initial_maid_updates,
			sender
		) = action_tables::GlobalActionTableContext::new(
			vec![],
			bfrt_arc.clone(),
			&table_info
		).unwrap();
		let global_ctx = TofinoBackendSingleThreadContext {
			global_qer_context: qer::GlobalQerContext::new()
		};
		{
			let mut bfrt2 = bfrt_arc.write().await;
			let bfrt = bfrt2.as_mut().unwrap();
			bfrt.write_update(initial_maid_updates).await.unwrap();
		}
		info!("Creating usage reporting thread");
		//let (pull_entry_update_tx, usage_seid_del_rx) = tokio::sync::mpsc::channel(3000);
		usage_reporting::create_usage_reporting_threads(settings.ur_tx, settings.urr_aux.clone());

		info!("TofinoBackend created");
		// step 5: return
		Box::new(TofinoBackend {
			bfrt: bfrt_arc,
			pfcp_contexts: tokio::sync::Mutex::new(HashMap::new()),
			global_contexts: Arc::new(Mutex::new(global_ctx)),
			global_action_table_context: action_table,
			table_info,
			callback_thread,
			urr_aux: settings.urr_aux,
			domain_watcher_android: None,//Some(dw_model_android)
		})
	}

	async fn on_to_cpu_packet_received(&self, packet: &[u8]) {

	}

	async fn update_or_add_forwarding(&self, seid: u64, pipelines: Vec<PacketPipeline>) -> Result<OperationPerfStats, Box<dyn std::error::Error + Send + Sync>> {
		if pipelines.len() == 0 {
			return Ok(OperationPerfStats { stats1: 0, stats2: 0, stats3: 0, stats4: 0 });
		}
		//info!("Backend: TofinoBackend::update_or_add_rules [SEID={}]", seid);
		//println!("{:#?}", pipelines);

		let perf_timer_1 = std::time::Instant::now();
		{
			let mut pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			if !pfcp_contexts_guard.contains_key(&seid) {
				pfcp_contexts_guard.insert(seid, Arc::new(RwLock::new(PfcpContext::new(seid))));
			}
		}

		
		// step 1: Turn pipelines to templates, track which ones require SendEM
		// step 2: figure out which pipelines are updates and which are inserts by tracking know PDR_ID within a given session
		// step 3: find PDRs who require buffered data to be sent
		
		let pfcp_ctx_ptr = {
			let pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			pfcp_contexts_guard.get(&seid).unwrap().clone()
		};
		let mut pfcp_ctx = pfcp_ctx_ptr.write().await;


		let mut paired_pipelines = linear_map::LinearMap::with_capacity(pipelines.len());
		for p in pipelines.into_iter() {
			paired_pipelines.insert(p.pdr_id.rule_id, p);
		}

		let to_del = Vec::from_iter(pfcp_ctx.to_del_pdr_ids.iter().cloned());
		
		
		pfcp_ctx.prepare_update();

 	 	// merge with old rules and update rules PFCP context
		pfcp_ctx.merge_updates(to_del, paired_pipelines)?;
		// reorder MA pipeline
 	 	pfcp_ctx.reorder_ma_rules();
		// find differences
		let (to_del, to_mod, to_add, untouched, untouched_pipelines) = pfcp_ctx.find_difference();
		

		// perf changes:
		// 1. use unsafe to make tree faster          -- 100x faster
		// 2. sort to delete MAIDs in piggyback       -- meh
		// 3: limit piggyback deletion to 10 entries  -- ok
		// 4. delete MAID in UPF driver update thread -- ok

		// 1. make libpfcp async
		// 4: do action table update in UPF driver   

		let piggybacked_maid_deletions: Vec<u32> = {
			let l = GLOBAL_MAID_RECLAIM_QUEUE.get().unwrap();
			let mut guard = l.write().await;
			if guard.len() > 10 {
				let mut to_remove = guard.drain(0..10).collect::<Vec<_>>();
				usage_reporting::remove_maids(&to_remove).await;
				to_remove.sort_by(|a, b| b.cmp(a));
				to_remove
			} else {
				vec![]
			}
		};
		
		let (
			touched,
			mut grpc_table_updates,
			mut upf_driver_table_updates,
			pdr_urr_associations,
			new_maids,
			to_del_maids
		) = self.global_action_table_context.update(
			&self.table_info,
			seid,
			to_del,
			to_mod,
			to_add,
			untouched_pipelines,
			piggybacked_maid_deletions
		).await?;
		pfcp_ctx.activate_pipelines(touched, untouched);
		pfcp_ctx.to_del_pdr_ids.clear();
		let perf_timer_1_ret = perf_timer_1.elapsed().as_micros() as u64;


		// let mut updates2 = if touched.len() != 0 {
		// 	let (_, &maid) = touched.iter().next().unwrap();
		// 	let ret = domain_watcher::add_monitored_ue(maid, 0, 0, &self.table_info).await;
		// 	if let Some((_, updates)) = ret {
		// 		updates
		// 	} else {
		// 		vec![]
		// 	}
		// } else {
		// 	vec![]
		// };


		// grpc_table_updates.append(&mut updates2);

		
		let perf_timer_2 = std::time::Instant::now();

		let task1_opt = if grpc_table_updates.len() != 0 {
			Some(
				async move {
					let mut bfrt2 = self.bfrt.write().await;
					let bfrt = bfrt2.as_mut().unwrap();
					bfrt.write_update(grpc_table_updates).await
				}
			)
		} else {
			None
		};
		

		// TODO: if pdr update failed it will leave garbage in usage reporting module
		// update PDR URR association
		let vol_thres_l = usage_reporting::update_pipeline_urr_association(seid, pdr_urr_associations, to_del_maids).await;
		for upf_driver_table_update in upf_driver_table_updates.iter_mut() {
			for vol_thres in vol_thres_l.iter() {
				upf_driver_table_update.set_vol_thres(vol_thres.maid, vol_thres.vol_thres);
			}
		}

		// we assume MAID tree update will always succeed
			
		let perf_timer_2_ret = perf_timer_2.elapsed().as_micros() as u64;
		// after applying MAID tree update, we perform PDR table update
		// if failed we need to roll back MAID allocation
		// info!("upf_driver_table_updates");
		// println!("{:#?}", upf_driver_table_updates);
		let perf_timer_3 = std::time::Instant::now();


		let task2 = upf_driver_interface::send_pdr_update(upf_driver_table_updates, 0);
		let pdr_update_result = if let Some(task1) = task1_opt {
			let (result1, pdr_update_result) = tokio::join!(task1, task2);
			result1.unwrap();
			pdr_update_result
		} else {
			task2.await
		};


		//let pdr_update_result = upf_driver_interface::send_pdr_update(upf_driver_table_updates, 0).await;


		let perf_timer_3_ret = perf_timer_3.elapsed().as_micros() as u64;
		//info!("Backend: TofinoBackend::update_or_add_rules [SEID={}] table update done", seid);
		if let Err(code) = pdr_update_result {
			error!("PDR update failed with code {}, will recycle MAID: {:?}", code, new_maids);
			let recycle_ops = self.global_action_table_context.recycle_maids(new_maids, &self.table_info).await;
			let mut bfrt2 = self.bfrt.write().await;
			let bfrt = bfrt2.as_mut().unwrap();
			bfrt.write_update(recycle_ops).await?; // we assume MAID tree update will always succeed
			return Err(TofinoBackendError::InsufficientTableCapacity.into());
		}

		//info!("Backend: TofinoBackend::update_or_add_rules [SEID={}] done", seid);
		let perf_timer_4_ret = perf_timer_1.elapsed().as_micros() as u64;
		Ok(OperationPerfStats { stats1: perf_timer_1_ret, stats2: perf_timer_2_ret, stats3: perf_timer_3_ret, stats4: perf_timer_4_ret })
	}

	async fn delete_forwarding(&self, seid: u64, pipelines: Vec<PDR_ID>) {
		//info!("Backend: TofinoBackend::delete_rules [SEID={}] {:?}", seid, pipelines);
		{
			let mut pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			if !pfcp_contexts_guard.contains_key(&seid) {
				pfcp_contexts_guard.insert(seid, Arc::new(RwLock::new(PfcpContext::new(seid))));
			}
		}

		let pfcp_ctx_ptr = {
			let pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			pfcp_contexts_guard.get(&seid).unwrap().clone()
		};
		let mut pfcp_ctx = pfcp_ctx_ptr.write().await;

		// delete_forwarding is always followed by update_or_add_forwarding, we just store it in cache and do the actual deleting in update_or_add_forwarding
		pfcp_ctx.to_del_pdr_ids.extend(pipelines.into_iter().map(|f| f.rule_id));
	}

	async fn delete_session(&self, seid: u64) -> (OperationPerfStats, Vec<BackendReport>) {
		//info!("Backend: TofinoBackend::delete_session [SEID={}]", seid);
		
		let perf_timer_1 = std::time::Instant::now();
		let mut perf_timer_3_ret = 0;

		let mut reports = vec![];
		let pfcp_ctx_ptr_opt = {
			let mut pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			pfcp_contexts_guard.remove(&seid)
		};
		if let Some(pfcp_ctx_lock) = pfcp_ctx_ptr_opt {
			let pfcp_ctx = pfcp_ctx_lock.write().await;
			let mut grpc_updates = vec![];

			// step 1: delet reporting
			reports.append(&mut usage_reporting::delete_pfcp_session(seid).await);

			// step 2: get all existing pipelines
			let mut upf_driver_updates = vec![];
			for (_pdr_id, pipeline) in pfcp_ctx.pipelines.iter() {
				for m in pipeline.matches.iter() {
					if let Some(maid) = &pipeline.maid {
						if let Some(update) = m.generate_upf_driver_update_remove(*maid) {
							upf_driver_updates.push(update);
						}
					}
				}
			}

			// step 3: get piggybacked
			let piggybacked_maid_deletions: Vec<u32> = {
				let l = GLOBAL_MAID_RECLAIM_QUEUE.get().unwrap();
				let mut guard = l.write().await;
				if guard.len() > 10 {
					let mut to_remove = guard.drain(0..10).collect::<Vec<_>>();
					usage_reporting::remove_maids(&to_remove).await;
					to_remove.sort_by(|a, b| b.cmp(a));
					to_remove
				} else {
					vec![]
				}
			};

			if piggybacked_maid_deletions.len() != 0 {
				grpc_updates.append(&mut self.global_action_table_context.remove_maids(&piggybacked_maid_deletions, &self.table_info).await);
			}

			// step 2: delete QER_IDs
			let mut global_ctx = self.global_contexts.lock().await;
			let global_qer_ctx = &mut global_ctx.global_qer_context;

			global_qer_ctx.transaction_begin();
			for (_, p) in pfcp_ctx.pipelines.iter() {
				if let Some(global_qer_id) = p.linked_global_qer_id {
					if global_qer_id != 0 && global_qer_id != 1 {
						global_qer_ctx.free_qer_id(&self.table_info, global_qer_id);
					}
				}
			}

			grpc_updates.append(&mut global_qer_ctx.transaction_commit());
			drop(global_ctx);
			// GRPC update first
			let task1_opt = if grpc_updates.len() != 0 {
				Some(
					async move {
						let mut bfrt2 = self.bfrt.write().await;
						let bfrt = bfrt2.as_mut().unwrap();
						bfrt.write_update(grpc_updates).await
					}
				)
			} else {
				None
			};
			// UPF driver update later
			let perf_timer_3 = std::time::Instant::now();
			let task2 = upf_driver_interface::send_pdr_update(upf_driver_updates, 0);
			let pdr_update_result = if let Some(task1) = task1_opt {
				let (result1, pdr_update_result) = tokio::join!(task1, task2);
				result1.unwrap();
				pdr_update_result
			} else {
				task2.await
			};
			pdr_update_result.unwrap();
			perf_timer_3_ret = perf_timer_3.elapsed().as_micros() as u64;
			//info!("pdr update delay {}ms", t0.elapsed().as_secs_f32() * 1000.0f32);
		}
		//info!("Backend: TofinoBackend::delete_session [SEID={}] done", seid);
		let perf_timer_4_ret = perf_timer_1.elapsed().as_micros() as u64;

		(OperationPerfStats { stats1: 0, stats2: 0, stats3: perf_timer_3_ret, stats4: perf_timer_4_ret }, reports)
	}

	async fn update_or_add_qer(&self, seid: u64, pipelines: Vec<FlattenedQER>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		{
			let mut pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			if !pfcp_contexts_guard.contains_key(&seid) {
				pfcp_contexts_guard.insert(seid, Arc::new(RwLock::new(PfcpContext::new(seid))));
			}
		}
		
		let mut global_ctx = self.global_contexts.lock().await;

		let pfcp_ctx_ptr = {
			let pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			pfcp_contexts_guard.get(&seid).unwrap().clone()
		};
		let mut pfcp_ctx = pfcp_ctx_ptr.write().await;

		let pfcp_qer_context = &mut pfcp_ctx.qers;
		let global_qer_ctx = &mut global_ctx.global_qer_context;
		global_qer_ctx.transaction_begin();

		for p in pipelines.iter() {
			if let Some(entry) = pfcp_qer_context.get_mut(&p.qer_id) {
				let global_qer_id = entry.0;
				if let Some(new_id) = global_qer_ctx.update_qer(&self.table_info, global_qer_id, *p)? {
					entry.0 = new_id;
				}
			} else {
				if let Some(global_qer_id) = global_qer_ctx.allocate_qer_id(&self.table_info, *p) {
					pfcp_qer_context.insert(p.qer_id, (global_qer_id, *p));
				} else {
					return Err(TofinoBackendError::InsufficientGBRFlowCapacity.into());
				}
			}
		}

		let updates = global_qer_ctx.transaction_commit();
		drop(global_ctx);
		//info!("[update_or_add_qer] [SEID={}] wrting BFRT", seid);
		{
			let mut bfrt2 = self.bfrt.write().await;
			let bfrt = bfrt2.as_mut().unwrap();
			bfrt.write_update(updates).await?;
		}
		//info!("[update_or_add_qer] [SEID={}] wrting BFRT done", seid);
		
		Ok(())
	}

	async fn delete_qer(&self, seid: u64, pipelines: Vec<QER_ID>) {
		let mut global_ctx = self.global_contexts.lock().await;

		let pfcp_ctx_ptr = {
			let pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			pfcp_contexts_guard.get(&seid).unwrap().clone()
		};
		let mut pfcp_ctx = pfcp_ctx_ptr.write().await;

		let pfcp_qer_context = &mut pfcp_ctx.qers;
		let global_qer_ctx = &mut global_ctx.global_qer_context;
		global_qer_ctx.transaction_begin();

		for p in pipelines.iter() {
			if let Some((global_qer_id, _)) = pfcp_qer_context.remove(&p.rule_id) {
				global_qer_ctx.free_qer_id(&self.table_info, global_qer_id);
			}
		}

		let updates = global_qer_ctx.transaction_commit();
		drop(global_ctx);
		{
			let mut bfrt2 = self.bfrt.write().await;
			let bfrt = bfrt2.as_mut().unwrap();
			bfrt.write_update(updates).await.unwrap();
		}
	}

	async fn update_or_add_urr(&self, seid: u64, pipelines: Vec<FlattenedURR>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// println!("===update_or_add_urr===");
		// println!("{:#?}", pipelines);
		
		{
			let mut pfcp_contexts_guard = self.pfcp_contexts.lock().await;
			if !pfcp_contexts_guard.contains_key(&seid) {
				pfcp_contexts_guard.insert(seid, Arc::new(RwLock::new(PfcpContext::new(seid))));
			}
		}

		usage_reporting::create_pfcp_session(seid).await;


		for p in pipelines.into_iter() {
			usage_reporting::add_or_update_usage_reporting(seid, p, self.urr_aux.clone()).await;
		}

		Ok(())
	}

	async fn delete_urr(&self, seid: u64, pipelines: Vec<URR_ID>) -> Vec<Option<BackendReport>> {
		// println!("===delete_urr===");
		// println!("{:#?}", pipelines);

		let mut ret = Vec::with_capacity(pipelines.len());

		for p in pipelines.into_iter() {
			if let Ok(x) = usage_reporting::delete_usage_reporting(seid, p.rule_id).await {
				ret.push(x);
			}
		}

		ret
	}

	async fn release_all_sessions(&self) {
		info!("Backend: TofinoBackend::release_all_sessions");
	}

	async fn reset_all(&self, settings: super::BackendSettings) {
		info!("Backend: TofinoBackend::reset_all");
	}

	async fn stop(&self) {
		info!("Backend: TofinoBackend::stop");
		let mut bfrt = self.bfrt.write().await;
		let bfrt2 = bfrt.take();
		drop(bfrt2);
	}
}
