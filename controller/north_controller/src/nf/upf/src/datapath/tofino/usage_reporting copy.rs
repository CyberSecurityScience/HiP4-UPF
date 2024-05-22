use async_trait::async_trait;
use rand::rngs::ThreadRng;

use std::cell::RefMut;
use std::collections::{HashSet, HashMap, VecDeque};
use std::convert::TryInto;
use std::f64::consts;
use std::iter::FromIterator;
use std::mem::take;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::Utc;
use libpfcp::models::{URR_ID, VolumeMeasurement, UR_SEQN, DurationMeasurement, TimeOfFirstPacket, TimeOfLastPacket};
use linear_map::LinearMap;
use linear_map::set::LinearSet;
use log::{info, error, warn};
use once_cell::sync::OnceCell;
use tokio::sync::{RwLock, Barrier, mpsc::{self, Receiver, Sender}};
use tokio::task::JoinHandle;

use crate::datapath::Report;
use crate::datapath::tofino::bfruntime::bfrt::{TableKey, KeyField, Entity, entity, Update, update};
use crate::n4::GLOBAL_PFCP_CONTEXT;
use crate::utils::BitrateEstimator;
use crate::{datapath::{FlattenedURR, FlattenedFAR, BackendReport}, context::get_async_runtime};
use lazy_static::lazy_static;

use super::URRTestAuxSettings;
use super::bfruntime::P4TableInfo;
use super::bfruntime::bfrt::key_field::Exact;
use super::bfruntime::bfrt::{DataField, data_field, key_field, table_entry, TableFlags, TargetDevice, TableData};
use super::{TofinoBackendError, table_templates::UsageReportingKey, bfruntime::{bfrt::TableEntry, self}};

pub const COUNTDOWN_TIME_MS: i64 = 40 * 1000; // 60 seconds countdown
pub const TRAFFIC_PAUSE_TIME_MS: u64 = 5 * 1000; // if no packets for 5 seconds then we consider that flow paused

#[derive(Debug)]
pub enum PullingEntryOp {
	Insert((u64, UsagePullingEntryPointer)),
	Delete((u64, UsagePullingEntryPointer)),
	DeleteURR(UsageReportingEntryPointer)
}

pub async fn add_or_update_usage_reporting(seid: u64, urr: FlattenedURR, aux: URRTestAuxSettings, pull_entry_update_tx: &tokio::sync::mpsc::Sender<PullingEntryOp>) {
	// info!("Creating URR [SEID={}, URR_ID={}] Method={:?}", seid, urr.urr_id, urr.measurement_method);
	let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
	let urr_id = urr.urr_id;
	let cur_timestamp_ms = start_timestamp.elapsed().as_millis() as u64;
	let cur_utc_seconds = utc_now_seconds();
	create_pfcp_session(seid, pull_entry_update_tx).await;
	if let Some(urr_ptr) = USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.get_mut(&(seid, urr_id)) {
		urr_ptr.create_or_update(cur_timestamp_ms, cur_utc_seconds, &urr);
	} else {
		let urr_ptr = UsageReportingEntryPointer::new(UsageReportingEntry::new(seid, &urr, cur_timestamp_ms, cur_utc_seconds, aux));
		urr_ptr.create_or_update(cur_timestamp_ms, cur_utc_seconds, &urr);
		if let Some(sess_entry) = USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.get(&seid) {
			sess_entry.add_urr(urr_id, urr_ptr.clone());
		}
		USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.insert((seid, urr_id), urr_ptr);
		USAGE_REPORTING_GLOBAL_CONTEXT.urr_seqn.insert((seid, urr_id), 0);
	}
}

pub async fn delete_usage_reporting(seid: u64, urr_id: u32, pull_entry_update_tx: &tokio::sync::mpsc::Sender<PullingEntryOp>) -> Result<Option<BackendReport>, TofinoBackendError> {
	// info!("Deleting Report [SEID={}, URR_ID={}]", seid, urr_id);
	let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
	let report = if let Some(mut urr_ptr) = USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.get_mut(&(seid, urr_id)) {
		let report = urr_ptr.generate_report(true);
		// {
		// 	let mut guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.write().await;
		// 	let pull_entries = guard.as_mut().unwrap();
		// 	pull_entries.retain(|(_, entry)| entry.get_seid() != seid);
		// }
		USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.remove(&(seid, urr_id));
		USAGE_REPORTING_GLOBAL_CONTEXT.urr_seqn.remove(&(seid, urr_id));
		//urr_ptr.del();
		pull_entry_update_tx.send(PullingEntryOp::DeleteURR(urr_ptr.value().clone())).await.unwrap();
		if let Some(sess_entry) = USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.get(&seid) {
			sess_entry.remove_urr(urr_id);
		}
		report
	} else {
		None
	};
	Ok(report)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PullingEntryEstimation {
	est_time: i32
}
use rand::{thread_rng, Rng};
impl PullingEntryEstimation {
	pub fn MAX() -> Self {
		Self {
			est_time: i32::MAX
		}
	}
	pub fn MIN() -> Self {
		Self {
			est_time: 0
		}
	}
	pub fn apply_jitter(&mut self) {
		let mut rng = thread_rng();
		self.est_time += rng.gen_range(0..100);
	}
}

#[derive(Debug)]
pub struct PdrUsageReportingAssociation {
	pub pdr_id: u16,
	pub maid: u32,
	pub ingress_entries: Vec<TableEntry>,
	pub is_uplink: bool,

	pub urr_ids: LinearSet<u32>
}

pub async fn action_table_update(
	bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>,
	table_info: &P4TableInfo,
	seid: u64,
	all_associations: Vec<PdrUsageReportingAssociation>,
	need_immediate_pulling: bool,
	list_of_to_free_maid_with_pdr_id: LinearMap<u32, u16>,
	maid_del_tx: &Sender<u32>
) -> Vec<Update> {
	// println!("===action_table_update===");
	// println!("need_immediate_pulling={}", need_immediate_pulling);
	// println!("list_of_to_free_maid_with_pdr_id={:?}", list_of_to_free_maid_with_pdr_id);
	let cur_entry = USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.get_mut(&seid).unwrap();
	if need_immediate_pulling {
		// step 1: pull affected entries
		let pull_entries = vec![(PullingEntryEstimation::MIN(), cur_entry.value().clone())];
		// step 1.1: run batch pull
		let mut s1 = 0f32;
		let mut s2 = 0f32;
		let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
		let cur_timestamp_utc_seconds = utc_now_seconds();
		if pull_entries.len() != 0 {
			//info!("forced pulling pull_entries.len()={}",pull_entries.len());
			do_batch_pulling(
				bfrt.clone(),
				table_info,
				&pull_entries,
				cur_timestamp_utc_seconds,
				None,
				start_timestamp,
				pull_entries.len(),
				0,
				None,
				maid_del_tx,
				None,
				&mut s1,
				&mut s2
			).await;
		}
		// step 1.2: run estimation
		let cur_timestamp_ms = start_timestamp.elapsed().as_millis() as u64;
		let cur_timestamp_utc_seconds = utc_now_seconds();
		cur_entry.estimate_milliseconds_to_next_pull(cur_timestamp_ms, cur_timestamp_utc_seconds);
	}
	let mut all_new_maids = all_associations.iter().map(|f| f.maid).collect::<Vec<_>>();
	let all_old_maids = cur_entry.reset_association(&all_associations, list_of_to_free_maid_with_pdr_id, need_immediate_pulling);
	// info!("all_new_maids={:?}", all_new_maids);
	// info!("all_old_maids={:?}", all_old_maids);
	cur_entry.invalid_cache();

	if !need_immediate_pulling {
		all_new_maids.retain(|f| !all_old_maids.contains(f));
	}

	// let mut bfrt_lock2 = bfrt.read().await;
	// let bfrt_lock = bfrt_lock2.as_ref().unwrap();
	// let table_info = &bfrt_lock.table_info;

	let mut table_updates = Vec::with_capacity(10);

	// delete old MAIDS
	if need_immediate_pulling {
		for maid in all_old_maids.iter() {
			let egress_table_entry = TableKey {
				fields: vec![
					KeyField {
						field_id: table_info.get_key_id_by_name("pipe.Egress.accounting.accounting_exact", "ma_id"),
						match_type: Some(key_field::MatchType::Exact(Exact {
							value: maid.to_be_bytes()[1..].to_vec()
						}))
					}
				]
			};
			let table_entry = TableEntry {
				table_id: table_info.get_table_by_name("pipe.Egress.accounting.accounting_exact"),
				data: None,//Some(table_data),
				is_default_entry: false,
				table_read_flag: None,
				table_mod_inc_flag: None,
				entry_tgt: None,
				table_flags: None,
				value: Some(table_entry::Value::Key(egress_table_entry)),
			};
			let entity = Entity {
				entity: Some(entity::Entity::TableEntry(table_entry))
			};
			let update = Update {
				r#type: update::Type::Delete as _,
				entity: Some(entity)
			};
			table_updates.push(update);
		}
		// info!("delete MAID {:?}", all_old_maids);
	}
	// add new MAIDS
	{
		for maid in all_new_maids.iter() {
			let egress_table_entry = TableKey {
				fields: vec![
					KeyField {
						field_id: table_info.get_key_id_by_name("pipe.Egress.accounting.accounting_exact", "ma_id"),
						match_type: Some(key_field::MatchType::Exact(Exact {
							value: maid.to_be_bytes()[1..].to_vec()
						}))
					}
				]
			};
			let table_data = TableData {
				action_id: table_info.get_action_id_by_name("pipe.Egress.accounting.accounting_exact", "Egress.accounting.inc_counter"),
				fields: vec![
					DataField {
						field_id: table_info.get_data_id_by_name("pipe.Egress.accounting.accounting_exact", "$COUNTER_SPEC_PKTS"),
						value: Some(data_field::Value::Stream(0u64.to_be_bytes().to_vec())) // <-- 64bit
					},
					DataField {
						field_id: table_info.get_data_id_by_name("pipe.Egress.accounting.accounting_exact", "$COUNTER_SPEC_BYTES"),
						value: Some(data_field::Value::Stream(0u64.to_be_bytes().to_vec())) // <-- 64bit
					},
				]
			};
			let table_entry = TableEntry {
				table_id: table_info.get_table_by_name("pipe.Egress.accounting.accounting_exact"),
				data: Some(table_data),
				is_default_entry: false,
				table_read_flag: None,
				table_mod_inc_flag: None,
				entry_tgt: None,
				table_flags: None,
				value: Some(table_entry::Value::Key(egress_table_entry)),
			};
			let entity = Entity {
				entity: Some(entity::Entity::TableEntry(table_entry))
			};
			let update = Update {
				r#type: update::Type::Insert as _,
				entity: Some(entity)
			};
			table_updates.push(update);
		}
		// info!("insert MAID {:?}", all_new_maids);
	}
	
	table_updates
}

pub async fn create_pfcp_session(seid: u64, pull_entry_update_tx: &tokio::sync::mpsc::Sender<PullingEntryOp>) {
	if !USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.contains_key(&seid) {
		// info!("Creating SEID={}", seid);
		let pfcp_session_pulling_entry_ptr = UsagePullingEntryPointer::new(UsagePullingEntry::new(seid));
		USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.insert(seid, pfcp_session_pulling_entry_ptr.clone());
		// let mut guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.write().await;
		// let pull_entries = guard.as_mut().unwrap();
		// pull_entries.push((PullingEntryEstimation::MAX(), pfcp_session_pulling_entry_ptr));
		pull_entry_update_tx.send(PullingEntryOp::Insert((seid, pfcp_session_pulling_entry_ptr))).await.unwrap();
	}
}

pub async fn delete_pfcp_session(
	bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>,
	info: &P4TableInfo,
	maid_del_tx: &Sender<u32>,
	seid: u64,
	pull_entry_update_tx: &tokio::sync::mpsc::Sender<PullingEntryOp>
) -> Vec<BackendReport> {
	//info!("Deleting SEID={}", seid);
	let mut ret = vec![];
	let last_pulling = false;
	if let Some((_, mut old_entry)) = USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.remove(&seid) {
		if last_pulling {
			// step 1: pull affected entries
			let pull_entries = vec![(PullingEntryEstimation::MIN(), old_entry.clone())];
			// step 1.1: run batch pull
			let mut s1 = 0f32;
			let mut s2 = 0f32;
			let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
			let cur_timestamp_utc_seconds = utc_now_seconds();
			if pull_entries.len() != 0 {
				//info!("forced pulling pull_entries.len()={}",pull_entries.len());
				do_batch_pulling(
					bfrt,
					info,
					&pull_entries,
					cur_timestamp_utc_seconds,
					None,
					start_timestamp,
					pull_entries.len(),
					0,
					None,
					maid_del_tx,
					None,
					&mut s1,
					&mut s2
				).await;
			}
		};
		// {
		// 	let t = std::time::Instant::now();
		// 	let mut guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries_map.write().await;
		// 	// let new_value = guard.take().unwrap().into_iter().filter(|(_, entry)| entry.get_seid() != seid).collect::<Vec<_>>();
		// 	// guard.replace(new_value);
		// 	guard.as_mut().unwrap().remove(&seid);
		// 	println!("[Remove]delete_pfcp_session remove_pulling_entry={}", t.elapsed().as_micros());
		// }
		//info!("pulling entry deleted");
		pull_entry_update_tx.send(PullingEntryOp::Delete((seid, old_entry.clone()))).await.unwrap();


		////////////////////////////////////
		for urr_id in old_entry.list_urr_ids() {
			//info!("deleting urr_id={}", urr_id);
			let report = if let Some((_, mut urr_ptr)) = USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.remove(&(seid, urr_id)) {
				//info!("report genereing urr_id={}", urr_id);
				let report = urr_ptr.generate_report(true);
				//info!("report generetd urr_id={}", urr_id);
				//USAGE_REPORTING_GLOBAL_CONTEXT.report_entries_map.remove(&(seid, urr_id));
				USAGE_REPORTING_GLOBAL_CONTEXT.urr_seqn.remove(&(seid, urr_id));
				//urr_ptr.del();
				pull_entry_update_tx.send(PullingEntryOp::DeleteURR(urr_ptr)).await.unwrap();
				if let Some(sess_entry) = USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.get(&seid) {
					sess_entry.remove_urr(urr_id);
				}
				report
			} else {
				None
			};
			if let Some(report) = report {
				ret.push(report);
			}
			//info!("deleted urr_id={}", urr_id);
		}
		old_entry.invalid();
		/////////////////////////////////


		//info!("Deleted SEID={}", seid);
	}
	ret
}

/// an entire PFCP session
#[async_trait]
pub trait UsagePullingEntryTrait {
	fn estimate_milliseconds_to_next_pull(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation;
	fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation;
	fn create_or_update(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urrs: &Vec<FlattenedURR>);
	async fn pull(
		&mut self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>, 
		post_qos_num_packets_vec: Vec<(u16, u32, u64)>,
		post_qos_num_bytes_vec: Vec<(u16, u32, u64)>,
		maid_del_tx: &Sender<u32>,
		aux: PullAuxData
	);
	fn generate_report(&mut self, deletion: bool) -> Vec<BackendReport>;
	fn generate_pulling_entry(&mut self, table_info: &P4TableInfo) -> (&Vec<TableEntry>, Vec<(u16, u32)>);
}

/// a single URR
pub trait UsageReportingEntryTrait {
	fn estimate_milliseconds_to_next_pull(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation;
	fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation;
	fn create_or_update(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urr: &FlattenedURR);
	fn pull(
		&mut self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets: LinearMap<u16, u64>,
		post_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets_base_offset: LinearMap<u16, u64>,
		post_qos_num_bytes_base_offset: LinearMap<u16, u64>,
		aux: PullAuxData
	);
	fn generate_report(&mut self, deletion: bool) -> Option<BackendReport>;
	fn get_maid_and_pdr_id_and_ingress_table_entries(&self) -> Vec<(u32, u16, Vec<TableEntry>)>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TrafficState {
	Ongoing,
	Paused
}

#[derive(Debug, Clone)]
pub enum UsageReportingEntryState {
	Normal,
	SlowedCountdown
}

/// a single URR
pub struct UsageReportingEntry {
	seid: u64,
	urr: FlattenedURR,
	pdr2direction_and_last_counters_and_mode: LinearMap<u16, (bool, u64, u64, u64, u64)>,
	pdr2maid_and_table_entry: LinearMap<u16, (u32, Vec<TableEntry>)>,

	preqos_ul_bitrate: BitrateEstimator,
	preqos_dl_bitrate: BitrateEstimator,
	postqos_ul_bitrate: BitrateEstimator,
	postqos_dl_bitrate: BitrateEstimator,
	stable_ul_bitrate: f64,
	stable_dl_bitrate: f64,
	cum_preqos_ul_num_packets: u64,
	cum_preqos_dl_num_packets: u64,
	cum_preqos_ul_num_bytes: u64,
	cum_preqos_dl_num_bytes: u64,
	cum_postqos_ul_num_packets: u64,
	cum_postqos_dl_num_packets: u64,
	cum_postqos_ul_num_bytes: u64,
	cum_postqos_dl_num_bytes: u64,
	postqos_volume_ul_thres: Option<u64>,
	postqos_volume_dl_thres: Option<u64>,

	ul_thres_reach_est_ms: u64,
	dl_thres_reach_est_ms: u64,

	periodical_ms: Option<u64>,
	next_periodical_milestone_ms: Option<u64>,

	time_thres_ms: Option<u64>,
	time_thres_measure_start_ms: Option<u64>,
	time_thres_measure_start_is_ul: Option<bool>,
	time_thres_measure_start_ul_bytes: Option<u64>,
	time_thres_measure_start_dl_bytes: Option<u64>,
	time_measure_start_ms: u64,
	time_consumed_ms: u64,
	time_inactivity_timer: Option<u64>,

	cached_est: PullingEntryEstimation,
	traffic_state: TrafficState,
	last_pull_timestamp_ms: u64,
	last_report_timestamp_ms: u64,
	last_pkt_change_timestamp_ms: u64,

	past_estimations: VecDeque<f64>,
	countdown_ms_offset: i64,

	state: UsageReportingEntryState,
	no_countdown_pull_needed_counter: i32,

	report_start_of_traffic: bool,
	report_stop_of_traffic: bool,

	pkt_changed_in_last_pull: bool,
	traffic_state_changed_in_last_pull: bool,

	aux: URRTestAuxSettings,
	countdown_ms: i64,
	pending_volume_report_generation: bool,

	time_est_done: bool
}
impl UsageReportingEntry {
	pub fn new(seid: u64, urr: &FlattenedURR, cur_timestamp_ms: u64, cur_utc_seconds: u32, aux: URRTestAuxSettings) -> Self {
		let countdown_ms = aux.countdown_ms;
		UsageReportingEntry {
			seid,
			urr: urr.clone(),
			pdr2direction_and_last_counters_and_mode: LinearMap::new(),
			pdr2maid_and_table_entry: LinearMap::new(),
			preqos_ul_bitrate: BitrateEstimator::new(cur_timestamp_ms, 1000, aux.ema_value),
			preqos_dl_bitrate: BitrateEstimator::new(cur_timestamp_ms, 1000, aux.ema_value),
			postqos_ul_bitrate: BitrateEstimator::new(cur_timestamp_ms, 1000, aux.ema_value),
			postqos_dl_bitrate: BitrateEstimator::new(cur_timestamp_ms, 1000, aux.ema_value),
			stable_ul_bitrate: 0.0f64,
			stable_dl_bitrate: 0.0f64,
			cum_preqos_ul_num_packets: 0,
			cum_preqos_dl_num_packets: 0,
			cum_preqos_ul_num_bytes: 0,
			cum_preqos_dl_num_bytes: 0,
			cum_postqos_ul_num_packets: 0,
			cum_postqos_dl_num_packets: 0,
			cum_postqos_ul_num_bytes: 0,
			cum_postqos_dl_num_bytes: 0,
			postqos_volume_ul_thres: None,
			postqos_volume_dl_thres: None,
			ul_thres_reach_est_ms: u64::MAX,
			dl_thres_reach_est_ms: u64::MAX,
			periodical_ms: None,
			next_periodical_milestone_ms: None,
			time_thres_ms: None,
			time_thres_measure_start_ms: None,
			time_thres_measure_start_is_ul: None,
			time_thres_measure_start_ul_bytes: None,
			time_thres_measure_start_dl_bytes: None,
			time_measure_start_ms: cur_timestamp_ms,
			time_consumed_ms: 0,
			time_inactivity_timer: None,
			state: UsageReportingEntryState::Normal,
			no_countdown_pull_needed_counter: 0,
			cached_est: PullingEntryEstimation::MAX(),
			traffic_state: TrafficState::Paused,
			last_pull_timestamp_ms: cur_timestamp_ms,
			last_report_timestamp_ms: cur_timestamp_ms,
			last_pkt_change_timestamp_ms: cur_timestamp_ms,
			past_estimations: VecDeque::with_capacity(6),
			countdown_ms_offset: 0,
			report_start_of_traffic: false,
			report_stop_of_traffic: false,
			pkt_changed_in_last_pull: false,
			traffic_state_changed_in_last_pull: false,
			aux,
			countdown_ms,
			pending_volume_report_generation: false,
			time_est_done: false
		}
	}
	pub fn associate_usage_reporting(&mut self, pdr_id: u16, maid: u32, table_entry: Vec<TableEntry>, uplink: bool, need_reassociate: bool) {
		if need_reassociate {
			// replace old values
			//info!("associate_usage_reporting::need_reassociate");
			self.pdr2direction_and_last_counters_and_mode.insert(pdr_id, (uplink, 0, 0, 0, 0));
			self.pdr2maid_and_table_entry.insert(pdr_id, (maid, table_entry));
		} else {
			if self.pdr2maid_and_table_entry.contains_key(&pdr_id) {
				// simple MAID update
				let (old_maid, _) = self.pdr2maid_and_table_entry.get_mut(&pdr_id).unwrap();
				//info!("No reassociate needed, updating MAID {} to {}", *old_maid, maid);
				*old_maid = maid;
			} else {
				// insert new
				//info!("associate_usage_reporting::insert_new");
				self.pdr2direction_and_last_counters_and_mode.insert(pdr_id, (uplink, 0, 0, 0, 0));
				self.pdr2maid_and_table_entry.insert(pdr_id, (maid, table_entry));
			}
		}
		//println!("[associate_usage_reporting] self.pdr2direction = {:?}", self.pdr2direction);
	}
	pub fn deassociate_usage_reporting(&mut self, pdr_id: u16) {
		self.pdr2direction_and_last_counters_and_mode.remove(&pdr_id);
		self.pdr2maid_and_table_entry.remove(&pdr_id);
	}
	pub fn reset_association(&mut self, need_reassociate: bool) -> LinearSet<u32> {
		let all_maids = LinearSet::from_iter(self.pdr2maid_and_table_entry.iter().map(|(_, (maid, _))| *maid));
		if need_reassociate {
			self.pdr2direction_and_last_counters_and_mode.clear();
			self.pdr2maid_and_table_entry.clear();
		}
		all_maids
	}
	pub fn change_maid(&mut self, old_id: u32, new_id: u32) -> Vec<Update> {
		vec![]
	}
	fn reset(&mut self) {
		self.cum_preqos_ul_num_packets = 0;
		self.cum_preqos_dl_num_packets = 0;
		self.cum_preqos_ul_num_bytes = 0;
		self.cum_preqos_dl_num_bytes = 0;
		self.cum_postqos_ul_num_packets = 0;
		self.cum_postqos_dl_num_packets = 0;
		self.cum_postqos_ul_num_bytes = 0;
		self.cum_postqos_dl_num_bytes = 0;
	}
	fn volume_estimation_postqos(&self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> i32 {
		let ul_est = if let Some(postqos_volume_ul_thres) = self.postqos_volume_ul_thres {
			std::cmp::max(self.ul_thres_reach_est_ms as i64 - cur_timestamp_ms as i64, 0) as i32
		} else {
			i32::MAX
		};
		let dl_est = if let Some(postqos_volume_dl_thres) = self.postqos_volume_dl_thres {
			std::cmp::max(self.dl_thres_reach_est_ms as i64 - cur_timestamp_ms as i64, 0) as i32
		} else {
			i32::MAX
		};
		std::cmp::min(ul_est, dl_est)
	}
	fn time_estimation(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> i32 {
		let val = if self.traffic_state == TrafficState::Ongoing {
			if let (Some(time_thres), Some(mut measure_start)) = (self.time_thres_ms, self.time_thres_measure_start_ms) {
				// let ul_adjustment = if let Some(ul_bytes) = self.time_thres_measure_start_ul_bytes {
				// 	((ul_bytes as f64 / self.stable_ul_bitrate) * 1000.0f64) as u64
				// } else {
				// 	0u64
				// };
				// let dl_adjustment = if let Some(dl_bytes) = self.time_thres_measure_start_ul_bytes {
				// 	((dl_bytes as f64 / self.stable_dl_bitrate) * 1000.0f64) as u64
				// } else {
				// 	0u64
				// };
				// measure_start -= std::cmp::max(ul_adjustment, dl_adjustment);
				let est = measure_start + time_thres;
				//self.time_est_done = true;
				std::cmp::max((est as i64) - (cur_timestamp_ms as i64), 0) as i32
			} else {
				i32::MAX
			}
		} else {
			i32::MAX
		};
		val
	}
	fn countdown(&self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> i32 {
		let last_pull_timestamp_ms = self.last_pull_timestamp_ms as i64;
		let cur_timestamp_ms = cur_timestamp_ms as i64;
		let countdown_ms = match self.state {
			UsageReportingEntryState::Normal => self.countdown_ms,
			UsageReportingEntryState::SlowedCountdown => self.countdown_ms,
		};
		let mut countdown_ms = self.countdown_ms + self.countdown_ms_offset;
		if let Some(period) = self.periodical_ms {
			countdown_ms = period as _;// std::cmp::min(countdown_ms, period as i64);
		}
		if self.traffic_state == TrafficState::Paused && self.time_thres_ms.is_some() {
			countdown_ms = 0;
		}
		let mut val = (countdown_ms - (cur_timestamp_ms - last_pull_timestamp_ms)) as i32;
		if !self.aux.allow_neg_countdown {
			val = std::cmp::max(val, 0);
		}
		val
	}
}
impl UsageReportingEntryTrait for UsageReportingEntry {
	fn estimate_milliseconds_to_next_pull(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation {
		// if !self.pending_volume_report_generation {
		// 	return PullingEntryEstimation::MAX();
		// }
		let mut min_est = PullingEntryEstimation::MAX();
		let mut vol_est_ms = i32::MAX;
		if self.aux.enable_volume_estimation {
			vol_est_ms = self.volume_estimation_postqos(cur_timestamp_ms, cur_utc_seconds);
		}
		let time_est_ms = self.time_estimation(cur_timestamp_ms, cur_utc_seconds);
		let skip_countdown = time_est_ms != i32::MAX && vol_est_ms == i32::MAX && self.traffic_state == TrafficState::Ongoing;
		let non_countdown_min_est = std::cmp::min(vol_est_ms, time_est_ms);
		// if non_countdown_min_est >= self.countdown_ms as i32 * self.aux.enter_slow_pull_mode_est_pull_distance {
		// 	self.no_countdown_pull_needed_counter += 1;
		// } else {
		// 	self.no_countdown_pull_needed_counter = 0;
		// }
		self.state = UsageReportingEntryState::Normal;

		// if self.aux.enable_delayed_countdown {
		// 	// if self.no_countdown_pull_needed_counter >= self.aux.enter_slow_pull_mode_rounds as i32 {
		// 	// 	self.state = UsageReportingEntryState::SlowedCountdown;
		// 	// } else {
		// 	// 	self.state = UsageReportingEntryState::Normal;
		// 	// }
		// 	let max_rate = self.past_bitrates.iter().cloned().fold(f64::NAN, f64::max);
		// 	let ul_est = if let Some(postqos_volume_ul_thres) = self.postqos_volume_ul_thres {
		// 		let remaining_bytes = std::cmp::max((postqos_volume_ul_thres as i64) - (self.cum_postqos_ul_num_bytes as i64), 0);
		// 		let remaining_ms = (remaining_bytes as f64) / max_rate;
		// 		if remaining_ms.is_finite() {
		// 			remaining_ms as i32
		// 		} else {
		// 			i32::MAX
		// 		}
		// 	} else {
		// 		i32::MAX
		// 	};
		// 	if ul_est >= self.countdown_ms as i32 * self.aux.enter_slow_pull_mode_est_pull_distance {
		// 		self.state = UsageReportingEntryState::SlowedCountdown;
		// 	} else {
		// 		self.state = UsageReportingEntryState::Normal;
		// 	}
		// }

		if self.aux.enable_delayed_countdown {

		}

		let countdown_est = if !skip_countdown { self.countdown(cur_timestamp_ms, cur_utc_seconds) } else { i32::MAX };
		// if !self.time_est_done {
		// 	min_est = std::cmp::min(min_est, self.countdown(cur_timestamp_ms, cur_utc_seconds));
		// }
		self.cached_est = PullingEntryEstimation { est_time: std::cmp::min(countdown_est, non_countdown_min_est) };
		self.cached_est
	}

	fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation {
		self.cached_est
	}

	fn create_or_update(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urr: &FlattenedURR) {
		// println!("===create_or_update===");
		// println!("{:#?}", urr);
		// TODO: lock update
		self.reset();
		let mut postqos_volume_ul_thres = None;
		let mut postqos_volume_dl_thres = None;
		let mut start_time_measurement_now = false;
		if urr.measurement_method.getVOLUM() != 0 {
			// TODO: PreQoS
			match &urr.volume_threshold {
				Some(vol_thres) if urr.reporting_triggers.getVOLTH() != 0 => {
					postqos_volume_ul_thres = vol_thres.uplink_volume;
					postqos_volume_dl_thres = vol_thres.downlink_volume;
					self.pending_volume_report_generation = true;
				}
				_ => {}
			};
			// TODO: quota
		}
		if urr.measurement_method.getDURAT() != 0 {
			match &urr.time_threshold {
				Some(time_thres) => {
					let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
					self.time_thres_ms = Some((time_thres.value * 1000) as u64);
					if let Some(mi) = &urr.measurement_information {
						if mi.getISTM() != 0 {
							start_time_measurement_now = true;
							self.traffic_state = TrafficState::Ongoing;
							self.time_thres_measure_start_ms = Some(cur_timestamp_ms);
						}
					}
					self.time_inactivity_timer = urr.inactivity_detection_time.as_ref().map(|f| f.value as u64 * 1000);
					//self.time_thres_reach_est_ms = Some(cur_timestamp_ms + self.time_thres_ms.unwrap() as u64);
					//info!("TIMTH_SETUP,{},time_thres.value={},time_now={},ETA={}ms", self.seid, time_thres.value, time_now, self.time_thres_ms.unwrap());
				},
				_ => {},
			}
		}
		if urr.reporting_triggers.getPERIO() != 0 {
			if let Some(period) = &urr.measurement_period {
				self.periodical_ms = Some(period.value as u64 * 1000);
				self.next_periodical_milestone_ms = Some(cur_timestamp_ms + period.value as u64 * 1000);
			}
		}
		self.time_consumed_ms = 0;
		self.time_measure_start_ms = cur_timestamp_ms;
		if !start_time_measurement_now {
			self.traffic_state = TrafficState::Paused;
		}
		self.report_stop_of_traffic = urr.reporting_triggers.getSTOPT() != 0;
		self.report_start_of_traffic = urr.reporting_triggers.getSTART() != 0;
		self.last_pkt_change_timestamp_ms = cur_timestamp_ms;
		self.last_report_timestamp_ms = cur_timestamp_ms;
		self.last_pull_timestamp_ms = cur_timestamp_ms;
		self.pkt_changed_in_last_pull = false;
		self.traffic_state_changed_in_last_pull = false;
		self.postqos_volume_ul_thres = postqos_volume_ul_thres;
		self.postqos_volume_dl_thres = postqos_volume_dl_thres;
	}

	fn pull(
		&mut self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets: LinearMap<u16, u64>,
		post_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets_base_offset: LinearMap<u16, u64>,
		post_qos_num_bytes_base_offset: LinearMap<u16, u64>,
		aux: PullAuxData
	) {
		let mut preqos_dl_inc_bytes = 0u64;
		let mut preqos_ul_inc_bytes = 0u64;
		let mut preqos_dl_inc_packets = 0u64;
		let mut preqos_ul_inc_packets = 0u64;
		let mut postqos_dl_inc_bytes = 0u64;
		let mut postqos_ul_inc_bytes = 0u64;
		let mut postqos_dl_inc_packets = 0u64;
		let mut postqos_ul_inc_packets = 0u64;
		for (pdr_id, cum_value) in pre_qos_num_packets {
			if let Some((uplink, pre_qos_last_bytes, pre_qos_last_packets, post_qos_last_bytes, post_qos_last_packets)) = self.pdr2direction_and_last_counters_and_mode.get_mut(&pdr_id) {
				let inc = cum_value - *pre_qos_last_packets;
				*pre_qos_last_packets = cum_value;
				if *uplink {
					preqos_ul_inc_packets += inc;
				} else {
					preqos_dl_inc_packets += inc;
				}
			}
		}
		for (pdr_id, cum_value) in pre_qos_num_bytes {
			if let Some((uplink, pre_qos_last_bytes, pre_qos_last_packets, post_qos_last_bytes, post_qos_last_packets)) = self.pdr2direction_and_last_counters_and_mode.get_mut(&pdr_id) {
				let inc = cum_value - *pre_qos_last_bytes;
				*pre_qos_last_bytes = cum_value;
				if *uplink {
					preqos_ul_inc_bytes += inc;
				} else {
					preqos_dl_inc_bytes += inc;
				}
			}
		}
		for (pdr_id, cum_value) in post_qos_num_packets {
			if let Some((uplink, pre_qos_last_bytes, pre_qos_last_packets, post_qos_last_bytes, post_qos_last_packets)) = self.pdr2direction_and_last_counters_and_mode.get_mut(&pdr_id) {
				let inc = {
					if *post_qos_last_packets > cum_value {
						error!("[pdr_id={}] cum_value={},post_qos_last_packets={}", pdr_id, cum_value, *post_qos_last_packets);
					}
					cum_value - *post_qos_last_packets
				};
				*post_qos_last_packets = cum_value - post_qos_num_packets_base_offset.get(&pdr_id).unwrap_or(&0);
				if *uplink {
					postqos_ul_inc_packets += inc;
				} else {
					postqos_dl_inc_packets += inc;
				}
			}
		}
		for (pdr_id, cum_value) in post_qos_num_bytes {
			if let Some((uplink, pre_qos_last_bytes, pre_qos_last_packets, post_qos_last_bytes, post_qos_last_packets)) = self.pdr2direction_and_last_counters_and_mode.get_mut(&pdr_id) {
				let inc = cum_value - *post_qos_last_bytes;
				*post_qos_last_bytes = cum_value - post_qos_num_bytes_base_offset.get(&pdr_id).unwrap_or(&0);
				if *uplink {
					postqos_ul_inc_bytes += inc;
				} else {
					postqos_dl_inc_bytes += inc;
				}
			}
		}
		self.traffic_state_changed_in_last_pull = false;
		if postqos_ul_inc_bytes != 0 || postqos_dl_inc_bytes != 0 || preqos_ul_inc_bytes != 0 || preqos_dl_inc_bytes != 0 {
			self.last_pkt_change_timestamp_ms = cur_timestamp_ms;
			if self.traffic_state == TrafficState::Paused {
				self.traffic_state_changed_in_last_pull = true;
				let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
				self.time_measure_start_ms = cur_timestamp_ms;
				if let Some(time_thres) = self.time_thres_ms {
					// update time estimation
					self.time_thres_measure_start_dl_bytes = Some(postqos_dl_inc_bytes);
					self.time_thres_measure_start_ul_bytes = Some(postqos_ul_inc_bytes);
					self.time_thres_measure_start_ms = Some(cur_timestamp_ms);// + std::cmp::max(time_thres as i64 - self.time_consumed_ms as i64, 0) as u64);
					//println!("TIMTH_START,{},time_measure_start_ms={},time_thres_reach_est_ms={:?},time_now={}", self.seid, self.time_measure_start_ms, self.time_thres_reach_est_ms, time_now);
				}
			}
			self.traffic_state = TrafficState::Ongoing;
			self.pkt_changed_in_last_pull = true;
		} else {
			self.pkt_changed_in_last_pull = false;
		}
		self.preqos_ul_bitrate.pull(cur_timestamp_ms, preqos_ul_inc_bytes);
		self.preqos_dl_bitrate.pull(cur_timestamp_ms, preqos_dl_inc_bytes);
		let br_ul = self.postqos_ul_bitrate.pull(cur_timestamp_ms, postqos_ul_inc_bytes);
		let br_dl = self.postqos_dl_bitrate.pull(cur_timestamp_ms, postqos_dl_inc_bytes);
		if self.stable_ul_bitrate == 0.0f64 {
			self.stable_ul_bitrate = br_ul;
		} else {
			self.stable_ul_bitrate = 0.9f64 * self.stable_ul_bitrate + 0.1f64 * br_ul;
		}
		if self.stable_dl_bitrate == 0.0f64 {
			self.stable_dl_bitrate = br_dl;
		} else {
			self.stable_dl_bitrate = 0.9f64 * self.stable_dl_bitrate + 0.1f64 * br_dl;
		}
		self.cum_preqos_ul_num_packets += preqos_ul_inc_packets;
		self.cum_preqos_dl_num_packets += preqos_dl_inc_packets;
		self.cum_preqos_ul_num_bytes += preqos_ul_inc_bytes;
		self.cum_preqos_dl_num_bytes += preqos_dl_inc_bytes;
		self.cum_postqos_ul_num_packets += postqos_ul_inc_packets;
		self.cum_postqos_dl_num_packets += postqos_dl_inc_packets;
		self.cum_postqos_ul_num_bytes += postqos_ul_inc_bytes;
		self.cum_postqos_dl_num_bytes += postqos_dl_inc_bytes;
		if postqos_ul_inc_bytes != 0 || postqos_dl_inc_bytes != 0 || preqos_ul_inc_bytes != 0 || preqos_dl_inc_bytes != 0 {
			// info!("at cur_timestamp_ms={}", cur_timestamp_ms);
			// info!("[SEID={}] PRE QOS UL INC = {} bytes, CUM = {}, bitrate = {} Bps", self.seid, preqos_ul_inc_bytes, self.cum_preqos_ul_num_bytes, br_ul);
			// info!("[SEID={}] PRE QOS DL INC = {} bytes, CUM = {}, bitrate = {} Bps", self.seid, preqos_dl_inc_bytes, self.cum_preqos_dl_num_bytes, br_dl);
			// info!("[SEID={}] POST QOS UL INC = {} bytes, CUM = {}, bitrate = {} Bps", self.seid, postqos_ul_inc_bytes, self.cum_postqos_ul_num_bytes, br_ul);
			// info!("[SEID={}] POST QOS DL INC = {} bytes, CUM = {}, bitrate = {} Bps", self.seid, postqos_dl_inc_bytes, self.cum_postqos_dl_num_bytes, br_dl);
		}
		let diff = cur_timestamp_ms - self.last_pull_timestamp_ms;
		self.last_pull_timestamp_ms = cur_timestamp_ms;

		// perform volume threshold estimation based on latest counter values
		// TODO: 2^32 ms is 49 days
		let mut remaining_bytes_log = 0;
		let mut remaining_ms_log = 0f64;
		let ul_est = if let Some(postqos_volume_ul_thres) = self.postqos_volume_ul_thres {
			let remaining_bytes = std::cmp::max((postqos_volume_ul_thres as i64) - (self.cum_postqos_ul_num_bytes as i64), 0);
			remaining_bytes_log = remaining_bytes;
			let remaining_ms = (remaining_bytes as f64) / self.postqos_ul_bitrate.bytes_per_ms();
			remaining_ms_log = remaining_ms;
			if remaining_ms.is_finite() {
				remaining_ms as u32
			} else {
				u32::MAX
			}
		} else {
			u32::MAX
		};
		self.past_estimations.push_back(((ul_est as u64) + cur_timestamp_ms) as f64 / 1000.0f64);
		let g = 2.0f64;
		let k = 0.5f64;
		if self.past_estimations.len() > 5 {
			self.past_estimations.pop_front();
			if self.aux.enable_delayed_countdown {
				let stddev = stats::stddev(self.past_estimations.iter().cloned());
				let cd = self.countdown_ms as f64 / 1000.0f64;
				if stddev >= cd / k {
					self.countdown_ms_offset = 0;
				} else {
					self.countdown_ms_offset = ((-k * g * stddev + g * cd) * 1000.0f64) as _;
				}
			}
		}
		let dl_est = if let Some(postqos_volume_dl_thres) = self.postqos_volume_dl_thres {
			let remaining_bytes = std::cmp::max((postqos_volume_dl_thres as i64) - (self.cum_postqos_dl_num_bytes as i64), 0);
			let remaining_ms = (remaining_bytes as f64) / self.postqos_dl_bitrate.bytes_per_ms();
			if remaining_ms.is_finite() {
				remaining_ms as u32
			} else {
				u32::MAX
			}
		} else {
			u32::MAX
		};

		self.ul_thres_reach_est_ms = ul_est as u64 + cur_timestamp_ms;
		// if postqos_ul_inc_bytes != 0 || postqos_dl_inc_bytes != 0 || preqos_ul_inc_bytes != 0 || preqos_dl_inc_bytes != 0 {
		// 	info!("UL THREAS_REACH_EST={}ms from now ({}ms)", ul_est, self.ul_thres_reach_est_ms);
		// }
		self.dl_thres_reach_est_ms = dl_est as u64 + cur_timestamp_ms;

		if self.aux.enable_auto_countdown {
			self.countdown_ms = aux.new_countdown_ms as i64 + self.aux.auto_countdown_offset;
		}

		{
			let guard = USAGE_REPORTING_GLOBAL_CONTEXT.volestlog_sender.lock().unwrap();
			if let Some(ref2) = guard.as_ref() {
				let log = VolEstLog {
					seid: self.seid,
					urr_id: self.urr.urr_id,
					cur_ts_system: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
					cur_ts_ms: cur_timestamp_ms,
					byte_per_ms: self.postqos_ul_bitrate.bytes_per_ms(),
					est_ts: self.ul_thres_reach_est_ms,
					flush: false,
					remaining_bytes: remaining_bytes_log,
					remaining_ms: remaining_ms_log,
					countdown_ms: self.countdown_ms,
					countdown_ms_offsetset: self.countdown_ms_offset,
				};
				ref2.send(UsageLog::VOLEST(log)).unwrap();
			}
		}

		if false {
			// TODO: time_thres_measure_start_ms is incompatible with time_inactivity_timer
			if let Some(inact_time) = self.time_inactivity_timer {
				if self.last_pull_timestamp_ms - self.last_pkt_change_timestamp_ms >= inact_time {
					if self.traffic_state == TrafficState::Ongoing {
						self.traffic_state_changed_in_last_pull = true;
						if let Some(time_thres) = self.time_thres_ms {
							// stop time measurement
							self.time_consumed_ms += cur_timestamp_ms - self.time_measure_start_ms - TRAFFIC_PAUSE_TIME_MS;
							let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
							//println!("TIMTH_PAUSE,{},cur_timestamp_ms={},time_now={},time_consumed_ms={}", self.seid, cur_timestamp_ms, time_now, self.time_consumed_ms);
							if self.time_consumed_ms < time_thres {
								self.time_thres_measure_start_ms = None;
							}
						}
					}
					self.time_measure_start_ms = cur_timestamp_ms;
					self.traffic_state = TrafficState::Paused;
				}
			}
		}
	}

	fn generate_report(&mut self, deletion: bool) -> Option<BackendReport> {
		let mut sqn = USAGE_REPORTING_GLOBAL_CONTEXT.urr_seqn.get_mut(&(self.seid, self.urr.urr_id)).unwrap();
		let mut report_generated = false;

		let volume_measurement: Option<VolumeMeasurement> = Some(
			VolumeMeasurement {
				total_volume: Some(self.cum_postqos_ul_num_bytes + self.cum_postqos_dl_num_bytes),
				uplink_volume: Some(self.cum_postqos_ul_num_bytes),
				downlink_volume: Some(self.cum_postqos_dl_num_bytes),
				total_packets: Some(self.cum_postqos_dl_num_packets + self.cum_postqos_ul_num_packets),
				uplink_packets: Some(self.cum_postqos_ul_num_packets),
				downlink_packets: Some(self.cum_postqos_dl_num_packets),
			}
		);
		let duration_measurement: Option<DurationMeasurement> = Some(
			DurationMeasurement {
				seconds: ((self.last_pull_timestamp_ms - self.time_measure_start_ms + self.time_consumed_ms) as f64 / 1000.0f64).round() as u32,
			}
		);
		let mut usage_report_trigger = libpfcp::models::UsageReportTrigger(0);

		if let Some(thres) = self.postqos_volume_ul_thres.as_mut() {
			if self.cum_postqos_ul_num_bytes >= *thres {
				// 29244:5.2.2.3.1 re-apply all the thresholds
				self.cum_postqos_ul_num_bytes = 0;
				report_generated = true;
				usage_report_trigger.setVOLTH(1);
			}
		}
		if let Some(thres) = self.postqos_volume_dl_thres.as_mut() {
			if self.cum_postqos_dl_num_bytes >= *thres {
				// 29244:5.2.2.3.1 re-apply all the thresholds
				self.cum_postqos_dl_num_bytes = 0;
				report_generated = true;
				usage_report_trigger.setVOLTH(1);
			}
		}
		//info!("RG[SEID={}] self.time_thres_ms={:?},self.time_thres_reach_est_ms={:?}", self.seid,self.time_thres_ms,self.time_thres_reach_est_ms);
		if let (Some(time_thres), Some(measure_start)) = (self.time_thres_ms, self.time_thres_measure_start_ms) {
			let est = time_thres + measure_start;
			//info!("RG[SEID={}] self.last_pull_timestamp_ms={},est={}", self.seid,self.last_pull_timestamp_ms,est);
			if self.last_pull_timestamp_ms >= est - 500 {
				report_generated = true;
				usage_report_trigger.setTIMTH(1);
				self.time_thres_measure_start_ms = Some(self.last_pull_timestamp_ms);// + self.time_thres_ms.unwrap() as u64); // 29244:5.2.2.3.1 re-apply all the thresholds
				let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
				//println!("TIMTH,{},last_pull_timestamp_ms={},time_now={}", self.seid, self.last_pull_timestamp_ms, time_now);
			}
		}
		if let (Some(period), Some(period_milestone)) = (self.periodical_ms, self.next_periodical_milestone_ms.as_mut()) {
			if self.last_pull_timestamp_ms >= *period_milestone - 500 {
				*period_milestone += period;
				report_generated = true;
				usage_report_trigger.setPERIO(1);
			}
		}
		let mut traffic_start = None;
		let mut traffic_stop = None;
		if self.report_start_of_traffic {
			if self.traffic_state_changed_in_last_pull && self.traffic_state == TrafficState::Ongoing {
				//info!("generting due to report_start_of_traffic");
				report_generated = true;
				usage_report_trigger.setSTART(1);
				traffic_start = Some(TimeOfFirstPacket::new(Utc::now()));
				self.report_start_of_traffic = false;
			}
		}
		if self.report_stop_of_traffic {
			if self.traffic_state == TrafficState::Paused && self.traffic_state_changed_in_last_pull {
				//info!("[SEID={}], last_pull_timestamp_s={}, last_pkt_change_timestamp_s={}", self.seid, self.last_pull_timestamp_ms / 1000, self.last_pkt_change_timestamp_ms / 1000);
				report_generated = true;
				usage_report_trigger.setSTOPT(1);
				traffic_stop = Some(TimeOfLastPacket::new(Utc::now()));
				self.report_stop_of_traffic = false;
			}
		}
		if report_generated || deletion {
			let ur = libpfcp::messages::UsageReport {
				urr_id: URR_ID { rule_id: self.urr.urr_id },
				ur_seqn: UR_SEQN { sqn: *sqn.value() },
				usage_report_trigger,
				start_time: None,
				end_time: None,
				volume_measurement: volume_measurement,
				duration_measurement: duration_measurement,
				ue_ip_address: None,
				network_instance: None,
				time_of_first_packet: traffic_start,
				time_of_last_packet: traffic_stop,
				usage_information: None,
				predefined_rules_name: None,
			};
			//info!("Report generated for [SEID={}]", self.seid);
			let report = BackendReport {
				report: Report::UsageReports(vec![ur]),
				seid: self.seid,
			};
			*sqn.value_mut() += 1;
			//self.pending_volume_report_generation = false; // TODO: repalce with actual report generation
			Some(report)
		} else {
			None
		}
	}

	fn get_maid_and_pdr_id_and_ingress_table_entries(&self) -> Vec<(u32, u16, Vec<TableEntry>)> {
		let mut ret = Vec::with_capacity(self.pdr2maid_and_table_entry.len());
		for (pdr_id, (maid, table_entries)) in self.pdr2maid_and_table_entry.iter() {
			ret.push((*maid, *pdr_id, table_entries.clone()));
		}
		ret
	}
}

pub async fn pull_cum_packets_per_session() -> HashMap<u64, (u64, u64)> {
	let mut ret = HashMap::new();
	for item in USAGE_REPORTING_GLOBAL_CONTEXT.pfcp_session_map.iter() {
		let (ul, dl) = item.post_qos_packets();
		ret.insert(*item.key(), (ul, dl));
	}
	ret
}

#[derive(Debug, Clone)]
pub struct UsageReportingEntryPointer {
	ptr: *mut UsageReportingEntry
}
impl UsageReportingEntryPointer {
	/// return true if the same
	pub fn compare(&self, urr: &FlattenedURR) -> bool {
		true
	}
	pub fn new(entry: UsageReportingEntry) -> Self {
		let copied = Box::new(entry);
		Self {
			ptr: Box::into_raw(copied)
		}
	}
	pub fn del(&mut self) {
		let x = unsafe { Box::from_raw(self.ptr) };
		drop(x);
	}
	pub fn post_qos_packets(&self) -> (u64, u64) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		(self_ref.cum_postqos_ul_num_packets, self_ref.cum_postqos_dl_num_packets)
	}
}
unsafe impl Send for UsageReportingEntryPointer {}
unsafe impl Sync for UsageReportingEntryPointer {}

impl Drop for UsageReportingEntryPointer {
	fn drop(&mut self) {
		
	}
}

impl UsageReportingEntryPointer {
	pub fn estimate_milliseconds_to_next_pull(&self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation {
		unsafe { self.ptr.as_mut() }.unwrap().estimate_milliseconds_to_next_pull(cur_timestamp_ms, cur_utc_seconds)
	}
	pub fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation {
		unsafe { self.ptr.as_mut() }.unwrap().get_estimated_milliseconds_to_next_pull()
	}
	pub fn create_or_update(&self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urr: &FlattenedURR) {
		unsafe { self.ptr.as_mut() }.unwrap().create_or_update(cur_timestamp_ms, cur_utc_seconds, urr)
	}
	pub fn pull(
		&mut self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets: LinearMap<u16, u64>,
		post_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets_base_offset: LinearMap<u16, u64>,
		post_qos_num_bytes_base_offset: LinearMap<u16, u64>,
		aux: PullAuxData
	) {
		unsafe { self.ptr.as_mut() }.unwrap().pull(cur_timestamp_ms, cur_utc_seconds, pre_qos_num_packets, pre_qos_num_bytes, post_qos_num_packets, post_qos_num_bytes, post_qos_num_packets_base_offset, post_qos_num_bytes_base_offset, aux)
	}
	pub fn generate_report(&self, deletion: bool) -> Option<BackendReport> {
		unsafe { self.ptr.as_mut() }.unwrap().generate_report(deletion)
	}
	pub fn change_maid(&mut self, old_id: u32, new_id: u32) -> Vec<Update> {
		unsafe { self.ptr.as_mut() }.unwrap().change_maid(old_id, new_id)
	}
	pub fn get_maid_and_pdr_id_and_ingress_table_entries(&self) -> Vec<(u32, u16, Vec<TableEntry>)> {
		unsafe { self.ptr.as_mut() }.unwrap().get_maid_and_pdr_id_and_ingress_table_entries()
	}
	pub fn associate_usage_reporting(&mut self, pdr_id: u16, maid: u32, table_entry: Vec<TableEntry>, uplink: bool, need_reassociate: bool) {
		unsafe { self.ptr.as_mut() }.unwrap().associate_usage_reporting(pdr_id, maid, table_entry, uplink, need_reassociate)
	}
	pub fn deassociate_usage_reporting(&mut self, pdr_id: u16) {
		unsafe { self.ptr.as_mut() }.unwrap().deassociate_usage_reporting(pdr_id)
	}
	pub fn reset_association(&mut self, need_reassociate: bool) -> LinearSet<u32> {
		unsafe { self.ptr.as_mut() }.unwrap().reset_association(need_reassociate)
	}
}

#[derive(Debug, Clone)]
pub struct PullAuxData {
	pub new_countdown_ms: u64
}
impl PullAuxData {
	pub fn new() -> Self {
		Self {
			new_countdown_ms: 0
		}
	}
}

/// represent a PFCP session
#[derive(Debug, Clone)]
pub struct UsagePullingEntry {
	pub valid: bool,

	seid: u64,
	cached_pulling_entries: Vec<TableEntry>,
	cached_pulling_entry_pdr_id_and_maids: Vec<(u16, u32)>,
	list_of_to_free_maid_with_pdr_id: LinearMap<u32, u16>,
	cache_invalided: bool,
	pre_qos_num_packets: LinearMap<u16, u64>,
	pre_qos_num_bytes: LinearMap<u16, u64>,
	post_qos_num_packets: LinearMap<u16, u64>,
	post_qos_num_bytes: LinearMap<u16, u64>,
	post_qos_num_packets_base_offset: LinearMap<u16, u64>,
	post_qos_num_bytes_base_offset: LinearMap<u16, u64>,
	pull_aux: Option<PullAuxData>,
	urr_id2urr: LinearMap<u32, UsageReportingEntryPointer>,

	pull_est: PullingEntryEstimation,
	pull_valid: bool,
	pull_cur_timestamp_ms: u64,
	pull_cur_utc_seconds: u32,
}

impl UsagePullingEntry {
	pub fn is_valid(&self) -> bool {
		self.valid
	}
	pub fn new(seid: u64) -> Self {
		Self {
			valid: true,
			seid,
			cached_pulling_entries: vec![],
			cached_pulling_entry_pdr_id_and_maids: vec![],
			list_of_to_free_maid_with_pdr_id: LinearMap::new(),
			cache_invalided: true,
			pre_qos_num_packets: LinearMap::new(),
			pre_qos_num_bytes: LinearMap::new(),
			post_qos_num_packets: LinearMap::new(),
			post_qos_num_bytes: LinearMap::new(),
			post_qos_num_packets_base_offset: LinearMap::new(),
			post_qos_num_bytes_base_offset: LinearMap::new(),
			pull_aux: None,
			urr_id2urr: LinearMap::new(),
			pull_est: PullingEntryEstimation::MAX(),
			pull_valid: false,
			pull_cur_timestamp_ms: 0,
			pull_cur_utc_seconds: 0,
		}
	}
	fn reset_pull(&mut self) {
		self.pull_valid = false;
		self.pre_qos_num_packets.clear();
		self.pre_qos_num_bytes.clear();
		self.post_qos_num_packets.clear();
		self.post_qos_num_bytes.clear();
		self.post_qos_num_packets_base_offset.clear();
		self.post_qos_num_bytes_base_offset.clear();
		self.pull_aux.take();
	}
	pub fn add_urr(&mut self, urr_id: u32, urr_ptr: UsageReportingEntryPointer) {
		self.urr_id2urr.insert(urr_id, urr_ptr);
	}
	pub fn list_urr_ids(&self) -> Vec<u32> {
		Vec::from_iter(self.urr_id2urr.keys().cloned())
	}
	pub fn remove_urr(&mut self, urr_id: u32) {
		self.urr_id2urr.remove(&urr_id);
	}
	pub fn reset_association(&mut self, all_associations: &Vec<PdrUsageReportingAssociation>, list_of_to_free_maid_with_pdr_id: LinearMap<u32, u16>, need_reassociate: bool) -> LinearSet<u32> {
		let mut allmaids = LinearSet::new();
		for (_, it) in self.urr_id2urr.iter_mut() {
			allmaids.extend(it.reset_association(need_reassociate).iter());
		}
		for (urr_id, it) in self.urr_id2urr.iter_mut() {
			for ass in all_associations.iter() {
				if ass.urr_ids.contains(urr_id) {
					it.associate_usage_reporting(ass.pdr_id, ass.maid, ass.ingress_entries.clone(), ass.is_uplink, need_reassociate);
				}
			}
		}
		self.list_of_to_free_maid_with_pdr_id.extend(list_of_to_free_maid_with_pdr_id);
		// will invalid cache by caller
		allmaids
	}
	pub fn get_seid(&self) -> u64 {
		self.seid
	}
	pub fn invalid_cache(&mut self) {
		self.cache_invalided = true;
	}
	pub fn change_maid(&mut self, old_id: u32, new_id: u32) -> Vec<Update> {
		let mut ret = vec![];
		for (_, it) in self.urr_id2urr.iter_mut() {
			ret.append(&mut it.change_maid(old_id, new_id));
		}
		ret
	}
}

#[async_trait]
impl UsagePullingEntryTrait for UsagePullingEntry {
	fn estimate_milliseconds_to_next_pull(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation {
		let mut min_est = PullingEntryEstimation::MAX();
		for (_, v) in self.urr_id2urr.iter_mut() {
			if self.pull_valid {
				let aux = self.pull_aux.as_ref().unwrap().clone();
				v.pull(
					self.pull_cur_timestamp_ms,
					self.pull_cur_utc_seconds,
					self.pre_qos_num_packets.clone(),
					self.pre_qos_num_bytes.clone(),
					self.post_qos_num_packets.clone(),
					self.post_qos_num_bytes.clone(),
					self.post_qos_num_packets_base_offset.clone(),
					self.post_qos_num_bytes_base_offset.clone(),
					aux
				);
			}
			min_est = std::cmp::min(min_est, v.estimate_milliseconds_to_next_pull(cur_timestamp_ms, cur_utc_seconds));
		}
		self.reset_pull();
		self.pull_est = min_est;
		//self.pull_est.apply_jitter();
		min_est
	}

	fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation {
		self.pull_est
	}


	fn create_or_update(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urrs: &Vec<FlattenedURR>) {
		unreachable!();
		// for urr in urrs.iter() {
		//     if let Some(exitsing) = self.urr_id2urr.get_mut(&urr.urr_id) {
		//         if !exitsing.compare(urr) {
		//             exitsing.create_or_update(cur_timestamp_ms, cur_utc_seconds, urr);
		//         }
		//     } else {
		//         let new_ptr = UsageReportingEntryPointer::new(UsageReportingEntry::new(urr));
		//         self.urr_id2urr.insert(urr.urr_id, new_ptr);
		//     }
		// }
	}

	async fn pull(&mut self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets_vec: Vec<(u16, u32, u64)>,
		post_qos_num_bytes_vec: Vec<(u16, u32, u64)>,
		maid_del_tx: &Sender<u32>,
		aux: PullAuxData
	) {
		self.reset_pull();
		self.pull_valid = true;
		self.pull_aux.replace(aux);
		self.pre_qos_num_packets = pre_qos_num_packets;
		self.pre_qos_num_bytes = pre_qos_num_bytes;
		for (pdr_id, maid, value) in post_qos_num_packets_vec.iter() {
			*self.post_qos_num_packets.entry(*pdr_id).or_insert(0) += value;
		}
		for (pdr_id, maid, value) in post_qos_num_bytes_vec.iter() {
			*self.post_qos_num_bytes.entry(*pdr_id).or_insert(0) += value;
		}
		self.pull_cur_timestamp_ms = cur_timestamp_ms;
		self.pull_cur_utc_seconds = cur_utc_seconds;

		let maids = self.list_of_to_free_maid_with_pdr_id.keys().collect::<Vec<_>>();
		if maids.len() != 0 {
			//info!("[SEID={}] pulled, notifying deletion of MAIDS {:?}", self.seid, maids);
			for (pdr_id, maid1, value) in post_qos_num_packets_vec.iter() {
				for maid2 in maids.iter() {
					if *maid1 == **maid2 {
						*self.post_qos_num_packets_base_offset.entry(*pdr_id).or_insert(0) += value;
					}
				}
			}
			for (pdr_id, maid1, value) in post_qos_num_bytes_vec.iter() {
				for maid2 in maids.iter() {
					if *maid1 == **maid2 {
						*self.post_qos_num_bytes_base_offset.entry(*pdr_id).or_insert(0) += value;
					}
				}
			}
			//info!("non zero base offset for post_qos_num_packets_vec={:?}", post_qos_num_packets_vec);
			//info!("non zero base offset for post_qos_num_bytes_vec={:?}", post_qos_num_bytes_vec);
			for id in self.list_of_to_free_maid_with_pdr_id.keys() {
				maid_del_tx.send(*id).await.unwrap();
			}
			self.list_of_to_free_maid_with_pdr_id.clear();
			self.invalid_cache();
		}
	}

	fn generate_report(&mut self, deletion: bool) -> Vec<BackendReport> {
		let mut ret = Vec::with_capacity(self.urr_id2urr.len());
		for v in self.urr_id2urr.values() {
			if let Some(report) = v.generate_report(deletion) {
				ret.push(report);
			}
		}
		ret
	}

	fn generate_pulling_entry(&mut self, table_info: &P4TableInfo) -> (&Vec<TableEntry>, Vec<(u16, u32)>) {
		if self.cache_invalided {
			self.cache_invalided = false;
			self.cached_pulling_entries.clear();
			self.cached_pulling_entry_pdr_id_and_maids.clear();
			let mut maid2ingress_table_entry = LinearMap::new();
			for v in self.urr_id2urr.values() {
				for (maid, pdr_id, tabke_keys) in v.get_maid_and_pdr_id_and_ingress_table_entries() {
					maid2ingress_table_entry.insert(maid, (pdr_id, tabke_keys));
				}
			}
			let table_flags = TableFlags {
				from_hw: true,
				key_only: false,
				mod_del: false,
				reset_ttl: false,
				reset_stats: false
			};
			let device = TargetDevice {
				device_id: 0,
				pipe_id: 0xffffu32,
				direction: 0xffu32,
				prsr_id: 0xffu32
			};
			self.cached_pulling_entries.reserve(maid2ingress_table_entry.len() * 2);
			self.cached_pulling_entry_pdr_id_and_maids.reserve(maid2ingress_table_entry.len() * 2);
			for (maid, (pdr_id, mut table_entries)) in maid2ingress_table_entry.into_iter() {
				let mut pull_entries = vec![
					{
						let egress_table_entry = TableKey {
							fields: vec![
								KeyField {
									field_id: table_info.get_key_id_by_name("pipe.Egress.accounting.accounting_exact", "ma_id"),
									match_type: Some(key_field::MatchType::Exact(Exact {
										value: maid.to_be_bytes()[1..].to_vec()
									}))
								}
							]
						};
						let table_entry = TableEntry {
							table_id: table_info.get_table_by_name("pipe.Egress.accounting.accounting_exact"),
							data: None,//Some(table_data),
							is_default_entry: false,
							table_read_flag: None,
							table_mod_inc_flag: None,
							entry_tgt: None,
							table_flags: Some(table_flags.clone()),
							value: Some(table_entry::Value::Key(egress_table_entry)),
						};
						table_entry // for egress
					}
				];
				for e in table_entries.iter_mut() {
					e.table_flags = Some(table_flags.clone());
				}
				for i in 0..pull_entries.len() {
					self.cached_pulling_entry_pdr_id_and_maids.push((pdr_id, maid));
				}
				let table_entries_len = table_entries.len();
				pull_entries.append(&mut table_entries); // for ingress
				for i in 0..table_entries_len {
					self.cached_pulling_entry_pdr_id_and_maids.push((pdr_id, u32::MAX));
				}
				self.cached_pulling_entries.append(&mut pull_entries);
			}
			// insert MAIDs used before Action Instance update
			for (maid, pdr_id) in self.list_of_to_free_maid_with_pdr_id.iter() {
				let pull_entry = {
					let egress_table_entry = TableKey {
						fields: vec![
							KeyField {
								field_id: table_info.get_key_id_by_name("pipe.Egress.accounting.accounting_exact", "ma_id"),
								match_type: Some(key_field::MatchType::Exact(Exact {
									value: maid.to_be_bytes()[1..].to_vec()
								}))
							}
						]
					};
					let table_entry = TableEntry {
						table_id: table_info.get_table_by_name("pipe.Egress.accounting.accounting_exact"),
						data: None,//Some(table_data),
						is_default_entry: false,
						table_read_flag: None,
						table_mod_inc_flag: None,
						entry_tgt: None,
						table_flags: Some(table_flags.clone()),
						value: Some(table_entry::Value::Key(egress_table_entry)),
					};
					table_entry // for egress
				};
				self.cached_pulling_entries.push(pull_entry);
				self.cached_pulling_entry_pdr_id_and_maids.push((*pdr_id, *maid));
			}
		}
		(&self.cached_pulling_entries, self.cached_pulling_entry_pdr_id_and_maids.clone())
	}
}

#[derive(Clone, Debug)]
pub struct UsagePullingEntryPointer {
	pub ptr: *mut UsagePullingEntry,
}
unsafe impl Send for UsagePullingEntryPointer {}
unsafe impl Sync for UsagePullingEntryPointer {}

impl Drop for UsagePullingEntryPointer {
	fn drop(&mut self) {
		
	}
}

impl UsagePullingEntryPointer {
	pub fn new(entry: UsagePullingEntry) -> Self {
		let copied = Box::new(entry);
		Self {
			ptr: Box::into_raw(copied)
		}
	}
	pub fn invalid(&self) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.valid = false;
	}
	pub fn is_valid(&self) -> bool {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.is_valid()
	}
	pub fn del(&self) {
		let x = unsafe { Box::from_raw(self.ptr) };
		drop(x);
	}
	pub fn estimate_milliseconds_to_next_pull(&self, cur_timestamp_ms: u64, cur_utc_seconds: u32) -> PullingEntryEstimation {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.estimate_milliseconds_to_next_pull(cur_timestamp_ms, cur_utc_seconds)
	}
	pub fn create_or_update(&mut self, cur_timestamp_ms: u64, cur_utc_seconds: u32, urrs: &Vec<FlattenedURR>) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.create_or_update(cur_timestamp_ms, cur_utc_seconds, urrs)
	}
	pub fn get_estimated_milliseconds_to_next_pull(&self) -> PullingEntryEstimation {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.get_estimated_milliseconds_to_next_pull()
	}
	async fn pull(
		&self,
		cur_timestamp_ms: u64,
		cur_utc_seconds: u32,
		pre_qos_num_packets: LinearMap<u16, u64>,
		pre_qos_num_bytes: LinearMap<u16, u64>,
		post_qos_num_packets_vec: Vec<(u16, u32, u64)>,
		post_qos_num_bytes_vec: Vec<(u16, u32, u64)>,
		maid_del_tx: &Sender<u32>,
		aux: PullAuxData
	) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.pull(cur_timestamp_ms, cur_utc_seconds, pre_qos_num_packets, pre_qos_num_bytes, post_qos_num_packets_vec, post_qos_num_bytes_vec, maid_del_tx, aux).await
	}
	pub fn generate_report(&self, deletion: bool) -> Vec<BackendReport> {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.generate_report(deletion)
	}
	pub fn post_qos_packets(&self) -> (u64, u64) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		let mut ul = 0;
		let mut dl = 0;
		for (k, v) in self_ref.urr_id2urr.iter() {
			let (cul, cdl) = v.post_qos_packets();
			ul += cul;
			dl += cdl;
		}
		(ul, dl)
	}
	pub fn generate_pulling_entry(&self, table_info: &P4TableInfo) -> (&Vec<TableEntry>, Vec<(u16, u32)>) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.generate_pulling_entry(table_info)
	}
	pub fn get_seid(&self) -> u64 {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.get_seid()
	}
	pub fn invalid_cache(&self) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.invalid_cache()
	}
	pub fn add_urr(&self, urr_id: u32, urr_ptr: UsageReportingEntryPointer) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.add_urr(urr_id, urr_ptr)
	}
	pub fn list_urr_ids(&self) -> Vec<u32> {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.list_urr_ids()
	}
	pub fn remove_urr(&self, urr_id: u32) {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.remove_urr(urr_id)
	}
	pub fn reset_association(&self, all_associations: &Vec<PdrUsageReportingAssociation>, list_of_to_free_maid_with_pdr_id: LinearMap<u32, u16>, need_reassociate: bool) -> LinearSet<u32> {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.reset_association(all_associations, list_of_to_free_maid_with_pdr_id, need_reassociate)
	}
	pub fn change_maid(&self, old_id: u32, new_id: u32) -> Vec<Update> {
		let self_ref = unsafe { self.ptr.as_mut() }.unwrap();
		self_ref.change_maid(old_id, new_id)
	}
}

pub struct UsageReportingGloablContext {
	pub urr_seqn: dashmap::DashMap<(u64, u32), u32>,
	pub pfcp_session_map: dashmap::DashMap<u64, UsagePullingEntryPointer>,
	pub report_entries_map: dashmap::DashMap<(u64, u32), UsageReportingEntryPointer>,
	pull_entries: RwLock<Option<Vec<(PullingEntryEstimation, UsagePullingEntryPointer)>>>,
	pub pull_aux: RwLock<PullAuxData>,
	volestlog_sender: Mutex<Option<std::sync::mpsc::Sender<UsageLog>>>
}
impl UsageReportingGloablContext {
	pub fn new() -> Self {
		Self {
			urr_seqn: dashmap::DashMap::new(),
			pfcp_session_map: dashmap::DashMap::new(),
			report_entries_map: dashmap::DashMap::new(),
			pull_entries: RwLock::new(Some(vec![])),
			pull_aux: RwLock::new(PullAuxData::new()),
			volestlog_sender: Mutex::new(None)
		}
	}
}

lazy_static! {
	pub static ref USAGE_REPORTING_GLOBAL_CONTEXT: UsageReportingGloablContext = UsageReportingGloablContext::new();
	static ref USGAE_TABLE_IDS: OnceCell<TableIds> = OnceCell::new();
}

#[derive(Debug, Clone, Copy)]
enum PullingEstimationMessage {
	EnqueueRound((usize, usize, u64, u32)),
	EnqueueBatch((usize, bool)),
	NotifyEstimationComplete
}

const EMA_ALPHA: f32 = 0.1;
const ROUND_TIME_MS: f32 = 200f32;

fn utc_now_seconds() -> u32 {
	let now = chrono::Utc::now();
	use chrono::{TimeZone};
	let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
	let diff = (now - start).num_seconds() as u32;
	diff
}

fn estimation_thread(
	mut pull2est_rx: Receiver<PullingEstimationMessage>,
	est2pull_tx: Sender<PullingEstimationMessage>,
	ur_tx: Sender<BackendReport>
) {
	let runtime = tokio::runtime::Builder::new_current_thread().thread_name("Tofino UPF estimation_thread").build().unwrap();
	runtime.block_on(async {
		loop {
			//info!("[estimation_thread] waiting for round request");
			let round_msg = pull2est_rx.recv().await.unwrap();
			let (mut total_entries, not_pulled_entries, est_timestamp_ms, est_timestamp_utc_seconds) = match round_msg {
				PullingEstimationMessage::EnqueueRound(a) => a,
				_ => unreachable!()
			};
			//info!("[estimation_thread] done waiting for round request, total_entries = {}, not_pulled_entries = {}", total_entries, not_pulled_entries);
			if not_pulled_entries != 0 {
				let pull_entries_guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.read().await;
				let pull_entries = pull_entries_guard.as_ref().unwrap();
				// estimate remaining entries
				let offset = total_entries - not_pulled_entries;
				for i in offset..pull_entries.len() {
					let (_, entry) = pull_entries.get(i).unwrap();
					if entry.is_valid() {
						entry.estimate_milliseconds_to_next_pull(est_timestamp_ms, est_timestamp_utc_seconds);
					}
				}
				total_entries -= not_pulled_entries;
			}
			let mut offset = 0;
			while total_entries > 0 {
				let batch_msg = pull2est_rx.recv().await.unwrap();
				//info!("[estimation_thread] waiting for batch request, remain={}", total_entries);
				let (cur_batch_size, success) = match batch_msg {
					PullingEstimationMessage::EnqueueBatch(a) => a,
					_ => unreachable!()
				};
				if success {
					//info!("done waiting for batch request cur_batch_size = {}", cur_batch_size);
					let pull_entries_guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.read().await;
					let pull_entries = pull_entries_guard.as_ref().unwrap();
					for i in offset..(offset + cur_batch_size) {
						let (_, entry) = pull_entries.get(i).unwrap();
						if entry.is_valid() {
							//info!("[EST] entry.estimate_milliseconds_to_next_pull()");
							entry.estimate_milliseconds_to_next_pull(est_timestamp_ms, est_timestamp_utc_seconds);
							//info!("[EST] done entry.estimate_milliseconds_to_next_pull()");
							let reports = entry.generate_report(false);
							for report in reports.into_iter() {
								ur_tx.send(report).await.unwrap();
							}
						}
					}
				}
				//info!("[EST] all batches done");
				offset += cur_batch_size;
				//info!("[EST] before total_entries={},cur_batch_size={}",total_entries,cur_batch_size);
				total_entries -= cur_batch_size;
				//info!("[EST] total_entries={}",total_entries);
			}
			//info!("[estimation_thread] notifying round finished");
			est2pull_tx.send(PullingEstimationMessage::NotifyEstimationComplete).await.unwrap();
		}
	});
}

fn extract_u64_from_datafield(d: &DataField) -> u64 {
	let val = d.value.as_ref().unwrap();
	match val {
		data_field::Value::Stream(s) => {
			let val = match s.len() {
				8 => {
					let v = u64::from_be_bytes(s.as_slice().try_into().unwrap());
					v
				}
				4 => {
					let v = u32::from_be_bytes(s.as_slice().try_into().unwrap());
					v as u64
				}
				_ => unreachable!()
			};
			val
		},
		_ => unreachable!()
	}
}


fn extract_u32_from_vec(s: &Vec<u8>) -> u32 {
	let val = match s.len() {
		8 => {
			let v = u64::from_be_bytes(s.as_slice().try_into().unwrap());
			v as u32
		}
		4 => {
			let v = u32::from_be_bytes(s.as_slice().try_into().unwrap());
			v 
		}
		3 => {
			let mut tmp = [0u8; 4];
			tmp[1..].copy_from_slice(s.as_slice());
			let v = u32::from_be_bytes(tmp);
			v 
		}
		_ => unreachable!()
	};
	val
}

#[derive(Debug, Clone, Copy)]
struct TableIds {
	post_qos_packets_id: u32,
	post_qos_bytes_id: u32,
	postqos_id: u32,
}

async fn do_batch_pulling(
	bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>,
	info: &P4TableInfo,
	pull_entries: &Vec<(PullingEntryEstimation, UsagePullingEntryPointer)>,
	cur_timestamp_utc_seconds: u32,
	pull_cur_timestamp_ms: Option<u64>,
	start_timestamp: std::time::Instant,
	pulling_batch_size: usize,
	batch_idx: usize,
	pull2est_tx: Option<&Sender<PullingEstimationMessage>>,
	maid_del_tx: &Sender<u32>,
	mut sync_wait_handle: Option<JoinHandle<()>>,
	pull_batch_time: &mut f32,
	pull_batch_time_bfrt: &mut f32,
) {
	let st = std::time::Instant::now();
	let ids = USGAE_TABLE_IDS.get().unwrap();
	let global_pull_aux = { USAGE_REPORTING_GLOBAL_CONTEXT.pull_aux.read().await }.clone();
	
	if batch_idx * pulling_batch_size >= pull_entries.len() {
		return;
	}
	let mut pulling_entry_lengths = Vec::with_capacity(pulling_batch_size * 2);
	let mut pulling_entries = Vec::with_capacity(pulling_batch_size * 2);
	let mut pulling_entries_pdr_id_and_maids = Vec::with_capacity(pulling_batch_size * 2);
	// step 3.1: create pulling updates
	let ref cur_batch = pull_entries[batch_idx * pulling_batch_size..std::cmp::min(pull_entries.len(), (batch_idx + 1) * pulling_batch_size)];
	if cur_batch.len() == 0 {
		return;
	}
	let mut filtered_cur_batch = Vec::with_capacity(pulling_batch_size * 2);
	cur_batch
		.iter()
		.filter(|f| f.1.is_valid())
		.for_each(|(_, entry)| {
			let (entries, mut pdr_ids) = entry.generate_pulling_entry(info);
			pulling_entry_lengths.push(entries.len());
			pulling_entries.append(&mut entries.clone());
			pulling_entries_pdr_id_and_maids.append(&mut pdr_ids);
			filtered_cur_batch.push(entry.clone());
		});
	if pulling_entries.len() == 0 {
		return;
	}
	if let Some(sync_wait_handle) = sync_wait_handle.take() {
		sync_wait_handle.await;
	}
	//println!("cur_batch[-1]={}",cur_batch.last().unwrap().0.est_time);
	assert_eq!(pulling_entries_pdr_id_and_maids.len(), pulling_entries.len());
	let mut bfrt_lock2 = bfrt.write().await;
	let bfrt_lock = bfrt_lock2.as_mut().unwrap();
	// step 3.2: do pulling
	let pull_cur_timestamp_ms = if let Some(pull_cur_timestamp_ms) = pull_cur_timestamp_ms {
		pull_cur_timestamp_ms
	} else {
		start_timestamp.elapsed().as_millis() as u64
	};
	let st_bfrt = std::time::Instant::now();
	let resp_result = bfrt_lock.read_table_entries(pulling_entries.clone()).await;
	*pull_batch_time_bfrt = (1f32 - EMA_ALPHA) * *pull_batch_time_bfrt + EMA_ALPHA * st.elapsed().as_secs_f32();
	if resp_result.is_err() {
		warn!("Pulling error={:?}", resp_result);
		if let Some(pull2est_tx) = pull2est_tx {
			pull2est_tx.send(PullingEstimationMessage::EnqueueBatch((cur_batch.len(), false))).await.unwrap();
		}
		return; // just skip this batch
	}
	let mut resp = resp_result.unwrap();
	let resp = resp.pop().unwrap().entities;
	let mut offset = 0usize;
	for (pull_idx, entry) in filtered_cur_batch
		.iter()
		.enumerate() {
			let cur_pulling_entry_length = pulling_entry_lengths[pull_idx];
			let mut pre_qos_packets_cum_map: LinearMap<u16, u64> = LinearMap::with_capacity(cur_pulling_entry_length);
			let mut pre_qos_bytes_cum_map: LinearMap<u16, u64> = LinearMap::with_capacity(cur_pulling_entry_length);
			let mut post_qos_packets_cum_vec: Vec<(u16, u32, u64)> = Vec::with_capacity(cur_pulling_entry_length);
			let mut post_qos_bytes_cum_vec: Vec<(u16, u32, u64)> = Vec::with_capacity(cur_pulling_entry_length);
			for ei in offset..(offset + cur_pulling_entry_length) {
				let rel_idx = ei - offset;
				let ref e = resp[ei];
				let (pdr_id, maid) = pulling_entries_pdr_id_and_maids[ei];
				let resp_entity = e.entity.as_ref().unwrap();
				let mut pre_qos_packets_cum = 0;
				let mut pre_qos_bytes_cum = 0;
				let mut post_qos_packets_cum = 0;
				let mut post_qos_bytes_cum = 0;
				let mut preqos = false;
				match resp_entity {
					bfruntime::bfrt::entity::Entity::TableEntry(t) => {
						if t.table_id == ids.postqos_id {
							for d in t.data.as_ref().unwrap().fields.iter() {
								if d.field_id == ids.post_qos_packets_id {
									post_qos_packets_cum = extract_u64_from_datafield(d);
								} else if d.field_id == ids.post_qos_bytes_id {
									post_qos_bytes_cum = extract_u64_from_datafield(d);
								}
							}
						} else {
							for d in t.data.as_ref().unwrap().fields.iter() {
								if d.field_id == 65553 {
									pre_qos_bytes_cum = extract_u64_from_datafield(d);
								} else if d.field_id == 65554 {
									pre_qos_packets_cum = extract_u64_from_datafield(d);
								}
							}
							preqos = true;
						}
					},
					_ => unreachable!()
				};
				if preqos {
					*pre_qos_packets_cum_map.entry(pdr_id).or_insert(0) += pre_qos_packets_cum;
					*pre_qos_bytes_cum_map.entry(pdr_id).or_insert(0) += pre_qos_bytes_cum;
				} else {
					post_qos_packets_cum_vec.push((pdr_id, maid, post_qos_packets_cum));
					post_qos_bytes_cum_vec.push((pdr_id, maid, post_qos_bytes_cum));
				}
			}
			entry.pull(
				pull_cur_timestamp_ms,
				cur_timestamp_utc_seconds,
				pre_qos_packets_cum_map,
				pre_qos_bytes_cum_map,
				post_qos_packets_cum_vec,
				post_qos_bytes_cum_vec,
				maid_del_tx,
				global_pull_aux.clone()
			).await;
			offset += cur_pulling_entry_length;
		};
	if let Some(pull2est_tx) = pull2est_tx {
		pull2est_tx.send(PullingEstimationMessage::EnqueueBatch((cur_batch.len(), true))).await.unwrap();
	}
	*pull_batch_time = (1f32 - EMA_ALPHA) * *pull_batch_time + EMA_ALPHA * st.elapsed().as_secs_f32();
}

pub struct LRStore {
    storage: HashMap<usize, f32>,
    slope: Option<f32>,
    intercept: Option<f32>,
	ema_parameter: f32,
}

impl LRStore {
    pub fn new(ema_parameter: f32) -> Self {
        Self {
            storage: HashMap::new(),
            slope: None,
            intercept: None,
            ema_parameter,
        }
    }

    pub fn store_value(&mut self, key: usize, value: f32) {
        if let Some(existing_value) = self.storage.get_mut(&key) {
            *existing_value = self.ema_parameter * value + (1.0 - self.ema_parameter) * *existing_value;
        } else {
            self.storage.insert(key, value);
        }

        if self.storage.len() > 2 {
            self.perform_linear_regression();
        }
    }

    pub fn retrieve_value(&self, key: usize) -> Option<&f32> {
        self.storage.get(&key)
    }

    pub fn get_slope(&self) -> Option<f32> {
        self.slope
    }

    pub fn get_intercept(&self) -> Option<f32> {
        self.intercept
    }

    pub fn predict_x_given_y(&self, y: f32) -> Option<f32> {
        match (self.slope, self.intercept) {
            (Some(slope), Some(intercept)) => Some((y - intercept) / slope),
            _ => None,
        }
    }

    pub fn predict_y_given_x(&self, x: usize) -> Option<f32> {
        match (self.slope, self.intercept) {
            (Some(slope), Some(intercept)) => Some(slope * x as f32 + intercept),
            _ => None,
        }
    }

    fn perform_linear_regression(&mut self) {
        let n = self.storage.len();
        let sum_x: f32 = self.storage.keys().sum::<usize>() as f32;
        let sum_y: f32 = self.storage.values().sum::<f32>();
        let sum_x_squared: f32 = self.storage.keys().map(|x| x.pow(2)).sum::<usize>() as f32;
        let sum_xy: f32 = self
            .storage
            .iter()
            .map(|(x, y)| (*x as f32) * (*y))
            .sum::<f32>();

        let slope = (n as f32 * sum_xy - sum_x * sum_y) / (n as f32 * sum_x_squared - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n as f32;

        self.slope = Some(slope);
        self.intercept = Some(intercept);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MinibatchSchedulerState {
	InsufficientUE,
	Learning,
	Inference
}

struct MinibatchScheduler {
	store: LRStore,
	pfcp_sessions_threshold: usize,
	state: MinibatchSchedulerState,
	learning_phase_counter: usize,
	rng: ThreadRng,
	next_minibatch_size: usize,
	consective_target_miss_count: usize,
	consective_target_achieved_count: usize
}

impl MinibatchScheduler {
	pub fn new(ema_parameter: f32, pfcp_sessions_threshold: usize, learning_phase_counter: usize) -> Self {
		Self {
			store: LRStore::new(ema_parameter),
			pfcp_sessions_threshold,
			state: MinibatchSchedulerState::InsufficientUE,
			learning_phase_counter,
			rng: rand::thread_rng(),
			next_minibatch_size: 1,
			consective_target_miss_count: 0,
			consective_target_achieved_count: 0
		}
	}

	pub fn get_minibatch_size(&mut self, target_time: f32, total_pfcp_sessions: usize) -> usize {
		match self.state {
			MinibatchSchedulerState::InsufficientUE => {
				if total_pfcp_sessions >= self.pfcp_sessions_threshold {
					self.state = MinibatchSchedulerState::Learning;
				}
				self.next_minibatch_size = total_pfcp_sessions;
				total_pfcp_sessions
			},
			MinibatchSchedulerState::Learning => {
				self.learning_phase_counter -= 1;
				if self.learning_phase_counter == 0 {
					self.state = MinibatchSchedulerState::Inference;
					self.next_minibatch_size = self.store.predict_x_given_y(target_time).unwrap() as usize;
					self.next_minibatch_size
				} else {
					let x = self.rng.gen_range(16..self.pfcp_sessions_threshold);
					x
				}
			},
			MinibatchSchedulerState::Inference => {
				self.next_minibatch_size
			},
		}
	}

	pub fn put_pull_time(&mut self, minibatch_size: usize, time: f32, target_time: f32, target_time_tol: f32) {
		self.store.store_value(minibatch_size, time);
		if self.state == MinibatchSchedulerState::Inference {
			self.next_minibatch_size = self.store.predict_x_given_y(target_time).unwrap() as usize;
			if time > target_time + target_time_tol {
				self.consective_target_miss_count += 1;
				self.consective_target_achieved_count = 0;
				self.next_minibatch_size -= self.consective_target_miss_count;
			} else {
				self.consective_target_miss_count = 0;
				self.consective_target_achieved_count += 1;
				self.next_minibatch_size += self.consective_target_achieved_count;
			}
			// explore
			let r = self.rng.gen_range(0..100);
			if r <= 2 { // 2% chance of random exploration
				let offset = self.rng.gen_range(-10..10isize);
				self.consective_target_miss_count = 0;
				self.consective_target_achieved_count = 0;
				self.next_minibatch_size = (self.next_minibatch_size as isize + offset) as usize;
			}
		}
	}

	pub fn is_estimator_ready(&self) -> bool {
		self.state == MinibatchSchedulerState::Inference
	}

	pub fn estimate_pulltime_from_batch_size(&self, bs: usize) -> Option<f32> {
		self.store.predict_y_given_x(bs)
	}
}

fn usage_pulling_thread(
	bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>,
	mut est2pull_rx: Receiver<PullingEstimationMessage>,
	pull2est_tx: Sender<PullingEstimationMessage>,
	maid_del_tx: Sender<u32>,
	log_tx: Option<std::sync::mpsc::Sender<UsageLog>>,
	mut seid_del_rx: tokio::sync::mpsc::Receiver<PullingEntryOp>,
	settings: URRTestAuxSettings
) {
	let runtime = tokio::runtime::Builder::new_current_thread().enable_time().thread_name("Tofino UPF usage_pulling_thread").build().unwrap();
	let target_pull_delay = settings.max_update_delay_ms as f32 / 1000.0f32;
	let target_pull_delay_tol = 0.001f32;
	let mut pulling_batch_size = 512usize;

	let mut pull_batch_time = 0.050f32; // 50ms
	let mut pull_batch_time_bfrt = 0.050f32; // 50ms
	let mut wait_for_all_estimation_time = 0.010f32; //10ms
	let mut sort_time = 0.02f32; // 20ms
	let mut sync_time = 0.05f32; // 50ms
	let overhead_time = 0.004f32; // 4ms
	let mut pull_round_time = settings.pull_round_time_ms as f32 / 1000.0f32;

	let mut no_batch_allocated_counter = 0u32;
	let mut ctr = 0usize;

	let mut minibatch_size2pull_time: LinearMap<usize, f32> = LinearMap::new();
	let mut minibatch_size_guess_lb = 64;
	let mut minibatch_size_guess_ub = 100;
	let mut sleep_time_initial_guess = 0.01f32;

	let mut minibatch_scheduler = MinibatchScheduler::new(0.95f32, 512, 1000);
	let mut total_pulling_entries_for_scheduler = 0;

	{
		use std::fs::OpenOptions;
		use std::io::prelude::*;
		let mut file = OpenOptions::new()
			.create(true)
			.append(true)
			.open("pull_log.log")
			.unwrap();
		if let Err(e) = writeln!(file, "usage_reporting_start") {
			eprintln!("Couldn't write to file: {}", e);
		}
	}

	runtime.block_on(async {
		assert!(core_affinity::set_for_current(core_affinity::CoreId {id: 3}));

		{
			use thread_priority::*;
			assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
		}

		let post_qos_packets_id;
		let post_qos_bytes_id;
		let postqos_id;
		{
			let bfrt_lock2 = bfrt.read().await;
			let bfrt_lock = bfrt_lock2.as_ref().unwrap();
			let ref table_info = bfrt_lock.table_info;
			post_qos_packets_id = table_info.get_data_id_by_name("pipe.Egress.accounting.accounting_exact", "$COUNTER_SPEC_PKTS");
			post_qos_bytes_id = table_info.get_data_id_by_name("pipe.Egress.accounting.accounting_exact", "$COUNTER_SPEC_BYTES");
			postqos_id = table_info.get_table_by_name("pipe.Egress.accounting.accounting_exact");
			let table_ids = TableIds {
				post_qos_packets_id,
				post_qos_bytes_id,
				postqos_id,
			};
			USGAE_TABLE_IDS.set(table_ids).unwrap();
		}
		let info = {
			let bfrt_lock2 = bfrt.read().await;
			bfrt_lock2.as_ref().unwrap().table_info.clone()
		};
		let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
		let mut num_entries_pulled = 0;
		let mut countdown_update_num_entries_pulled = 0;
		let mut countdown_update_pull_per_sec_ema = 2200.0f64;
		let mut next_milestone = (start_timestamp.elapsed().as_millis() as u64) + 10 * 1000;
		let mut countdown_update_start_ts = start_timestamp.elapsed().as_millis() as u64;
		let mut countdown_update_next_milestone = countdown_update_start_ts + settings.auto_countdown_update_freq_ms as u64;
		loop {
			ctr += 1;
			
			let mut num_batches = 1;

			let round_start_time = std::time::Instant::now();
			// step 1: sync table
			if false {
				let st = std::time::Instant::now();
				let mut bfrt_lock2 = bfrt.write().await;
				let bfrt_lock = bfrt_lock2.as_mut().unwrap();
				bfrt_lock.sync_table(
					vec![
						"pipe.Ingress.pdr.dl_N6_simple_ipv4",
						"pipe.Ingress.pdr.dl_N6_complex_ipv4",
						"pipe.Ingress.pdr.dl_N9_simple_ipv4",
						"pipe.Ingress.pdr.ul_N6_simple_ipv4",
						"pipe.Ingress.pdr.ul_N6_complex_ipv4",
						"pipe.Ingress.pdr.ul_N9_simple_ipv4",
						"pipe.Ingress.pdr.ul_N9_complex_ipv4",
						"pipe.Egress.accounting.accounting_exact"
					]
				).await;
				sync_time = (1f32 - EMA_ALPHA) * sync_time + EMA_ALPHA * st.elapsed().as_secs_f32();
			}

			let cur_timestamp_ms = start_timestamp.elapsed().as_millis() as u64;
			let cur_timestamp_utc_seconds = utc_now_seconds();
			let cur_system_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

			// step 2: estimate how many we can pull
			let minibatch_size = {
				// largest value that ensure pulling a minibatch last shorter than target_pull_delay
				(minibatch_size_guess_lb + minibatch_size_guess_ub) >> 1
			};
			//let minibatch_size = 60;
			// let mut minibatch_size = minibatch_scheduler.get_minibatch_size(target_pull_delay, total_pulling_entries_for_scheduler);
			// minibatch_size = std::cmp::max(minibatch_size, 1);
			let num_batch_this_epoch = {
				std::cmp::max((pulling_batch_size - 1) / minibatch_size, 1)
			};

			
			let mut sleep_time_per_minibatch = {
				if let Some(t) = minibatch_size2pull_time.get(&minibatch_size) {
					let mut free_time = pull_round_time - sync_time - overhead_time - wait_for_all_estimation_time - (num_batch_this_epoch as f32) * *t;
					if free_time <= 0.0f32 {
						free_time = 0.001f32 * (num_batch_this_epoch as f32);
					}
					free_time / (num_batch_this_epoch as f32)
				} else {
					sleep_time_initial_guess
				}
			};
			if sleep_time_per_minibatch < 0.002f32 {
				sleep_time_per_minibatch = 0.002f32;
			}
			// let sleep_time_per_minibatch = {
			// 	if minibatch_scheduler.is_estimator_ready() {
			// 		minibatch_scheduler.estimate_pulltime_from_batch_size(minibatch_size).unwrap()
			// 	} else {
			// 		sleep_time_initial_guess
			// 	}
			// };


			// stop using that estimator
			let num_batch_this_epoch = 1;
			let sleep_time_per_minibatch = 0f32;
			let minibatch_size = 512;

			if ctr % 50 == 0 {
				info!("minibatch_size={},num_batch_this_epoch={},sleep_time_per_minibatch={}ms,pull_batch_time={}ms,pull_batch_time_bfrt={}ms", 
				minibatch_size, num_batch_this_epoch, sleep_time_per_minibatch * 1000.0f32, pull_batch_time * 1000.0f32, pull_batch_time_bfrt * 1000.0f32);
			}

			let mut cur_epoch_pull_entries_len = 0;
			let mut estimation_notified = false;
			for batch_idx in 0..num_batch_this_epoch {
				// step 3: hold entries lock
				{
					let pull_entries_guard = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.read().await;
					let pull_entries = pull_entries_guard.as_ref().unwrap();
					total_pulling_entries_for_scheduler = pull_entries.len();
					if !estimation_notified {
						// step 3: notify urgency estimate for not pulled entries
						let est_timestamp_ms = cur_timestamp_ms + num_batch_this_epoch as u64 * (((pull_batch_time + sleep_time_per_minibatch) * 1000.0f32) as u64) + (sync_time * 1000.0f32) as u64;
						let est_timestamp_utc_seconds = cur_timestamp_utc_seconds;
						cur_epoch_pull_entries_len = pull_entries.len();
						let num_of_entries_not_pulled_this_round = std::cmp::max(pull_entries.len() as isize - num_batch_this_epoch as isize * (minibatch_size as isize), 0) as usize;
						if let Err(e) = pull2est_tx.send(PullingEstimationMessage::EnqueueRound((cur_epoch_pull_entries_len, num_of_entries_not_pulled_this_round, est_timestamp_ms, est_timestamp_utc_seconds))).await {
							error!("{:?}", e);
						}
						estimation_notified = true;
					}
					// step 4: pull
					do_batch_pulling(
						bfrt.clone(),
						&info,
						&pull_entries,
						cur_timestamp_utc_seconds,
						None,//Some(round_start_time),
						start_timestamp,
						minibatch_size,
						batch_idx,
						Some(&pull2est_tx),
						&maid_del_tx,
						None,
						&mut pull_batch_time,
						&mut pull_batch_time_bfrt
					).await;
				}
				num_entries_pulled += num_batches * minibatch_size;
				countdown_update_num_entries_pulled += num_batches * minibatch_size;
				// step 5: sleep

				//tokio::time::sleep(Duration::from_secs_f32(sleep_time_per_minibatch)).await;
			}
			// step 6: wait for all estimation
			let st = std::time::Instant::now();
			est2pull_rx.recv().await.unwrap();
			wait_for_all_estimation_time = (1f32 - EMA_ALPHA) * wait_for_all_estimation_time + EMA_ALPHA * st.elapsed().as_secs_f32();
			// step 7: update stats
			// minibatch_scheduler.put_pull_time(minibatch_size, pull_batch_time, target_pull_delay, target_pull_delay_tol);
			if false {
				if let Some(t) = minibatch_size2pull_time.get_mut(&minibatch_size) {
					*t = (1f32 - EMA_ALPHA) * *t + EMA_ALPHA * pull_batch_time;
				} else {
					minibatch_size2pull_time.insert(minibatch_size, pull_batch_time);
				}
				if pull_batch_time >= target_pull_delay + target_pull_delay_tol {
					// reduce minibatch size
					if minibatch_size_guess_lb > 10 {
						minibatch_size_guess_lb -= 1;
					}
					if minibatch_size_guess_ub > 10 {
						minibatch_size_guess_ub -= 1;
					}
				} else if pull_batch_time <= target_pull_delay - target_pull_delay_tol {
					// increase minibatch size
					if minibatch_size_guess_lb < 256 {
						minibatch_size_guess_lb += 1;
					}
					if minibatch_size_guess_ub < 256 {
						minibatch_size_guess_ub += 1;
					}
				} else {
					minibatch_size_guess_lb = minibatch_size - 1;
					minibatch_size_guess_ub = minibatch_size + 1;
				}
			}

			// step 8: log
			// if let Some(log_tx) = log_tx.as_ref() {
			// 	let pull_entries_log = pull_entries.iter().map(|(est, entry)| (est.clone(), entry.get_seid())).collect::<Vec<_>>();
			// 	let log_entry = UsageLog::PULL(PullLog {
			// 		cur_ts_system: cur_system_timestamp,
			// 		cur_ts_ms: cur_timestamp_ms,
			// 		entries: pull_entries_log,
			// 	});
			// 	log_tx.send(log_entry).unwrap();
			// }
				
			// step 9: sort all entries and delete pointers
			let st = std::time::Instant::now();
			{
				let mut to_del_set = LinearSet::new();
				let mut to_add_list = LinearMap::new();
				let mut ops: Vec<PullingEntryOp> = vec![];
				while let Ok(op) = seid_del_rx.try_recv() {
					ops.push(op);
				}
				// TODO: we assume within a round for a given SEID, insert followed by delete will not happen
				for op in ops.iter() {
					match op {
						PullingEntryOp::Insert(x) => {
							to_add_list.insert(x.0, x.clone());
						},
						PullingEntryOp::Delete((to_del_seid, e)) => {
							to_del_set.insert(to_del_seid);
							to_add_list.remove(to_del_seid);
						},
        				PullingEntryOp::DeleteURR(_) => {},
					}
				}

				let mut pull_entries_guard2 = USAGE_REPORTING_GLOBAL_CONTEXT.pull_entries.write().await;

				let pull_entries = pull_entries_guard2.as_mut().unwrap();
				if to_del_set.len() != 0 {
					pull_entries.retain(|x| !to_del_set.contains(&x.1.get_seid()));
				}

				for (_, item) in to_add_list {
					pull_entries.push((PullingEntryEstimation::MAX(), item.1.clone()));
				}

				pull_entries.iter_mut().for_each(|(key, entry)| {
					*key = entry.get_estimated_milliseconds_to_next_pull();
				});
				pull_entries.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

				for op in ops {
					match op {
						PullingEntryOp::Insert(x) => {},
						PullingEntryOp::Delete((to_del_seid, e)) => {
							e.del();
						},
        				PullingEntryOp::DeleteURR(mut e) => e.del(),
					}
				}
			}
			sort_time = (1f32 - EMA_ALPHA) * sort_time + EMA_ALPHA * st.elapsed().as_secs_f32();
			// step 10: more logging
			if (start_timestamp.elapsed().as_millis() as u64) >= next_milestone && false {
				use std::fs::OpenOptions;
				use std::io::prelude::*;
				let mut file = OpenOptions::new()
					.create(true)
					.append(true)
					.open("pull_log.log")
					.unwrap();
				let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
				if let Err(e) = writeln!(file, "{}: num_entries_pulled={},sort_time={},wait_for_all_estimation_time={},pull_batch_time={}", time_now, num_entries_pulled, sort_time, wait_for_all_estimation_time, pull_batch_time) {
					eprintln!("Couldn't write to file: {}", e);
				}
				next_milestone = (start_timestamp.elapsed().as_millis() as u64) + 10 * 1000;
				num_entries_pulled = 0;
			}
			if (start_timestamp.elapsed().as_millis() as u64) >= countdown_update_next_milestone {
				let elp = start_timestamp.elapsed().as_millis() as u64 - countdown_update_start_ts;
				let pull_per_sec = 1000.0f64 * countdown_update_num_entries_pulled as f64 / (elp as f64);
				countdown_update_pull_per_sec_ema = 0.9f64 * countdown_update_pull_per_sec_ema + 0.1f64 * pull_per_sec;
				let new_countdown_time_ms = (1000.0f64 * cur_epoch_pull_entries_len as f64 / countdown_update_pull_per_sec_ema) as u64;
				let mut aux = USAGE_REPORTING_GLOBAL_CONTEXT.pull_aux.write().await;
				aux.new_countdown_ms = new_countdown_time_ms;
				countdown_update_start_ts = start_timestamp.elapsed().as_millis() as u64;
				countdown_update_next_milestone = countdown_update_start_ts + settings.auto_countdown_update_freq_ms as u64;
				countdown_update_num_entries_pulled = 0;
				if false {
					use std::fs::OpenOptions;
					use std::io::prelude::*;
					let mut file = OpenOptions::new()
						.create(true)
						.append(true)
						.open("countdown_val.log")
						.unwrap();
					let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
					if let Err(e) = writeln!(file, "{}: new_countdown_time_ms={},pull_per_sec={},ent_pull={}", time_now, new_countdown_time_ms, pull_per_sec, countdown_update_pull_per_sec_ema) {
						eprintln!("Couldn't write to file: {}", e);
					}
				}
			}

			// no more estimator
			let pull_round_time_ms = settings.pull_round_time_ms as u64;
			let round_time_elp = round_start_time.elapsed().as_millis() as u64;
			if round_time_elp < pull_round_time_ms {
				tokio::time::sleep(Duration::from_millis(pull_round_time_ms - round_time_elp)).await;
			}
		}
	});
}

#[derive(Debug)]
struct VolEstLog {
	pub seid: u64,
	pub urr_id: u32,
	pub cur_ts_system: u64,
	pub cur_ts_ms: u64,
	pub byte_per_ms: f64,
	pub remaining_bytes: i64,
	pub remaining_ms: f64,
	pub countdown_ms: i64,
	pub countdown_ms_offsetset: i64,
	pub est_ts: u64,
	pub flush: bool
}

#[derive(Debug)]
struct PullLog {
	pub cur_ts_system: u64,
	pub cur_ts_ms: u64,
	pub entries: Vec<(PullingEntryEstimation, u64)>,
}

#[derive(Debug)]
enum UsageLog {
	VOLEST(VolEstLog),
	PULL(PullLog)
}

fn write_to_vol_est_log(filename: &str, logs: Option<&Vec<VolEstLog>>, sep: Option<bool>) {
	use std::fs::OpenOptions;
	use std::io::prelude::*;
	let mut file = OpenOptions::new()
		.create(true)
		.append(true)
		.open(filename)
		.unwrap();
	if let Some(_) = sep {
		file.write(&[116u8]).unwrap();
	}
	if let Some(logs) = logs {
		for log in logs {
			file.write(&[114u8]).unwrap();
			file.write(&log.seid.to_be_bytes()).unwrap();
			file.write(&log.urr_id.to_be_bytes()).unwrap();
			file.write(&log.cur_ts_system.to_be_bytes()).unwrap();
			file.write(&log.cur_ts_ms.to_be_bytes()).unwrap();
			file.write(&log.byte_per_ms.to_be_bytes()).unwrap();
			file.write(&log.remaining_bytes.to_be_bytes()).unwrap();
			file.write(&log.remaining_ms.to_be_bytes()).unwrap();
			file.write(&log.countdown_ms.to_be_bytes()).unwrap();
			file.write(&log.countdown_ms_offsetset.to_be_bytes()).unwrap();
			file.write(&log.est_ts.to_be_bytes()).unwrap();
		}
	}
	file.flush().unwrap();
}

fn write_to_pull_log(filename: &str, logs: Option<&Vec<PullLog>>, sep: Option<bool>) {
	use std::fs::OpenOptions;
	use std::io::prelude::*;
	let mut file = OpenOptions::new()
		.create(true)
		.append(true)
		.open(filename)
		.unwrap();
	if let Some(_) = sep {
		file.write(&[116u8]).unwrap();
	}
	if let Some(logs) = logs {
		for log in logs {
			file.write(&[114u8]).unwrap();
			file.write(&log.entries.len().to_be_bytes()).unwrap();
			file.write(&log.cur_ts_system.to_be_bytes()).unwrap();
			file.write(&log.cur_ts_ms.to_be_bytes()).unwrap();
			let mut bytes = Vec::with_capacity((4 + 8) * log.entries.len());
			for (est, seid) in log.entries.iter() {
				bytes.extend_from_slice(&est.est_time.to_be_bytes());
				bytes.extend_from_slice(&seid.to_be_bytes());
			}
			file.write(&bytes).unwrap();
			println!("write_to_pull_log len={}", log.entries.len());
		}
	}
	file.flush().unwrap();
}

fn vol_est_log_recevier_thread(filename_volest: String, filename_pull: String, mut rx: std::sync::mpsc::Receiver<UsageLog>) {
	let mut logs_volest = Vec::with_capacity(10000);
	let mut logs_sort = Vec::with_capacity(10000);
	while let Ok(log) = rx.recv() {
		match log {
			UsageLog::VOLEST(log_volest) => {
				let flush = log_volest.flush;
				logs_volest.push(log_volest);
				if logs_volest.len() >= 9990 || flush {
					write_to_vol_est_log(&filename_volest, Some(&logs_volest), None);
					logs_volest.clear();
				}
			},
			UsageLog::PULL(log_sort) => {
				logs_sort.push(log_sort);
				if logs_sort.len() >= 50 {
					write_to_pull_log(&filename_pull, Some(&logs_sort), None);
					logs_sort.clear();
				}
			}
		}
		
	}
}

pub fn create_usage_reporting_threads(bfrt: Arc<RwLock<Option<bfruntime::BFRuntime>>>, ur_tx: Sender<BackendReport>, maid_del_tx: Sender<u32>, seid_del_rx: tokio::sync::mpsc::Receiver<PullingEntryOp>, enable_volume_est_log: bool, est_log_filename: &str, settings: URRTestAuxSettings) {
	let log_tx = if enable_volume_est_log {
		write_to_vol_est_log(est_log_filename, None, Some(true));
		let (tx, rx) = std::sync::mpsc::channel::<UsageLog>();
		let name = est_log_filename.to_string();
		let name2 = est_log_filename.to_string() + ".pull";
		std::thread::spawn(move || {
			vol_est_log_recevier_thread(
				name,
				name2,
				rx
			)
		});
		USAGE_REPORTING_GLOBAL_CONTEXT.volestlog_sender.lock().unwrap().replace(tx.clone());
		Some(tx)
	} else { None };
	let (pull2est_tx, pull2est_rx) = mpsc::channel::<PullingEstimationMessage>(32);
	let (est2pull_tx, est2pull_rx) = mpsc::channel::<PullingEstimationMessage>(32);
	std::thread::spawn(move || {
		estimation_thread(
			pull2est_rx,
			est2pull_tx,
			ur_tx
		)
	});
	std::thread::spawn(move || {
		usage_pulling_thread(
			bfrt,
			est2pull_rx,
			pull2est_tx,
			maid_del_tx,
			log_tx,
			seid_del_rx,
			settings
		)
	});
}
