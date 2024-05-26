use std::{sync::{Arc}, collections::HashMap};

use atomic_counter::AtomicCounter;
use chrono::Utc;
use lazy_static::lazy_static;
use libpfcp::models::{VolumeMeasurement, URR_ID, UR_SEQN, DurationMeasurement, TimeOfFirstPacket, TimeOfLastPacket};
use linear_map::set::LinearSet;
use log::info;
use tokio::sync::{mpsc::Sender, RwLock};

use crate::datapath::{FlattenedURR, BackendReport, Report};

use super::{URRTestAuxSettings, TofinoBackendError, response_generated::upfdriver::response::CounterValueArgs};

#[derive(Debug)]
pub struct PdrUsageReportingAssociation {
	pub pdr_id: u16,
	pub maid: u32,
	pub is_uplink: bool,

	pub urr_ids: LinearSet<u32>
}

pub struct MaIdVolThresholdPair {
	pub maid: u32,
	pub vol_thres: u64
}

pub async fn add_or_update_usage_reporting(seid: u64, urr: FlattenedURR, aux: URRTestAuxSettings) {
	if let Some(context) = USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.get(&seid) {
		let mut context_guard = context.write().await;
		let ts = crate::context::GLOBAL_TIMER.get().unwrap().get_cur_us();
		if let Some(existing_urr) = context_guard.urr_entries.get_mut(&urr.urr_id) {
			existing_urr.apply_urr(&urr, ts);
		} else {
			let mut new_urr = SingleUsageReportingEntry::new(seid, urr.urr_id);
			new_urr.apply_urr(&urr, ts);
			context_guard.urr_entries.insert(urr.urr_id, new_urr);
		}
	}
}

pub async fn delete_usage_reporting(seid: u64, urr_id: u32) -> Result<Option<BackendReport>, TofinoBackendError> {
	let mut ret = None;
	if let Some(context) = USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.get(&seid) {
		let mut context_guard = context.write().await;
		if let Some(mut urr_entry) = context_guard.urr_entries.remove(&urr_id) {
			let ts = crate::context::GLOBAL_TIMER.get().unwrap().get_cur_us();
			ret = urr_entry.generate_report(ts, true);
		}
		for (_, (_, v)) in context_guard.maid2reporting_entry.iter_mut() {
			v.remove(&urr_id);
		}
	}
	Ok(ret)
}

pub async fn create_pfcp_session(seid: u64) {
	if USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.contains_key(&seid) {
		return;
	}
	let context = UsageReportingPfcpContext::new();
	USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.insert(seid, Arc::new(RwLock::new(context)));
}


pub async fn delete_pfcp_session(seid: u64) -> Vec<BackendReport> {
	if let Some((_, context)) = USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.remove(&seid) {
		let mut context_guard = context.write().await;
		context_guard.generate_report(true)
	} else {
		vec![]
	}
}

pub async fn update_pipeline_urr_association(seid: u64, all_associations: Vec<PdrUsageReportingAssociation>, to_del_maids: Vec<u32>) -> Vec<MaIdVolThresholdPair> {
	let mut all_maids = vec![];
	let mut vol_thres_l = vec![];
	let ctx = USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.get(&seid).map(|f| f.value().clone());
	if let Some(context) = ctx {
		let mut context_guard = context.write().await;
		// step 1: keep a copy of old assoications
		let cloned = context_guard.maid2reporting_entry.clone();
		// step 2: rebuild assoications
		context_guard.maid2reporting_entry.clear();
		for a in all_associations.iter() {
			if let Some((_, urr_ids)) = context_guard.maid2reporting_entry.get_mut(&a.maid) {
				urr_ids.extend(a.urr_ids.iter());
			} else {
				context_guard.maid2reporting_entry.insert(a.maid, (a.is_uplink, a.urr_ids.clone()));
			}
		}
		// step 3: keep to delete MAIDs
		for maid in to_del_maids {
			if let Some(existing_association) = cloned.get(&maid) {
				context_guard.maid2reporting_entry.insert(maid, existing_association.clone());
			}
		}
		// step 4: initialize counters
		let keys = context_guard.maid2reporting_entry.keys().cloned().collect::<Vec<_>>();
		for maid in keys {
			if !context_guard.maid2value_postqos.contains_key(&maid) {
				context_guard.maid2value_postqos.insert(maid, (0, 0));
			}
			if !context_guard.maid2value_preqos.contains_key(&maid) {
				context_guard.maid2value_preqos.insert(maid, (0, 0));
			}
			all_maids.push(maid);
		}
		// step 4: update global MAID mapping
		//let mut map = USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.write().await;
		vol_thres_l.append(&mut context_guard.get_ma_id_vol_thres());
		drop(context_guard);
		for maid in all_maids {
			USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.insert(maid, context.clone());
		}
	}
	vol_thres_l
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TrafficState {
	Ongoing,
	Paused
}


struct SingleUsageReportingEntry {
	pub seid: u64,
	pub urr_id: u32,
	pub urr_sqn_gen: atomic_counter::RelaxedCounter,
	pub rule_applied: bool,

	pub traffic_state: TrafficState,

	pub ul_bytes: u64,
	pub dl_bytes: u64,
	pub ul_pkts: u64,
	pub dl_pkts: u64,

	pub ul_threshold: u64,
	pub dl_threshold: u64,

	pub preqos_ul_bytes: u64,
	pub preqos_dl_bytes: u64,
	pub preqos_ul_pkts: u64,
	pub preqos_dl_pkts: u64,

	pub traffic_usage_thres: u64,
	pub next_reporting_ts_us: u64,
	pub reporting_peroid_us: u64,

	pub last_push_ts_us: u64,
	pub last_traffic_change_ts_us: u64,
	pub last_traffic_state_change_ts_us: u64,
	pub traffic_state_changed: bool,

	pub traffic_inactivity_timer_us: u64,
	pub cum_traffic_usage_time_us: u64,

	pub report_start_of_traffic: bool,
	pub report_stop_of_traffic: bool
}

impl SingleUsageReportingEntry {
	pub fn new(seid: u64, urr_id: u32) -> Self {
		Self {
			seid,
			urr_id,
			urr_sqn_gen: atomic_counter::RelaxedCounter::new(0),
			rule_applied: false,
			traffic_state: TrafficState::Paused,
			ul_bytes: 0,
			dl_bytes: 0,
			ul_pkts: 0,
			dl_pkts: 0,
			ul_threshold: u64::MAX,
			dl_threshold: u64::MAX,
			preqos_ul_bytes: 0,
			preqos_dl_bytes: 0,
			preqos_ul_pkts: 0,
			preqos_dl_pkts: 0,
			traffic_usage_thres: u64::MAX,
			next_reporting_ts_us: u64::MAX,
			reporting_peroid_us: u64::MAX,
			last_push_ts_us: 0,
			last_traffic_change_ts_us: 0,
			last_traffic_state_change_ts_us: 0,
			traffic_state_changed: false,
			traffic_inactivity_timer_us: u64::MAX,
			cum_traffic_usage_time_us: 0,
			report_start_of_traffic: false,
			report_stop_of_traffic: false
		}
	}
	pub fn apply_urr(&mut self, urr: &FlattenedURR, ts: u64) {
		if !self.rule_applied {
			self.last_push_ts_us = ts;
			self.last_traffic_change_ts_us = ts;
			self.last_traffic_state_change_ts_us = ts;
		}
		
		if urr.measurement_method.getVOLUM() != 0 {
			// TODO: PreQoS
			match &urr.volume_threshold {
				Some(vol_thres) if urr.reporting_triggers.getVOLTH() != 0 => {
					self.ul_threshold = vol_thres.uplink_volume.unwrap_or(u64::MAX);
					self.dl_threshold = vol_thres.downlink_volume.unwrap_or(u64::MAX);
				}
				_ => {}
			};
			// TODO: quota
		}
		if urr.measurement_method.getDURAT() != 0 {
			match &urr.time_threshold {
				Some(time_thres) => {
					self.traffic_usage_thres = time_thres.value as u64 * 1000_000;
					if let Some(mi) = &urr.measurement_information {
						if mi.getISTM() != 0 {
							self.traffic_state = TrafficState::Ongoing;
							self.last_traffic_change_ts_us = ts;
						}
					}
					self.traffic_inactivity_timer_us = urr.inactivity_detection_time.as_ref().map(|f| f.value as u64 * 1000_000).unwrap_or(u64::MAX);
				},
				_ => {},
			}
		}
		if urr.reporting_triggers.getPERIO() != 0 {
			if let Some(period) = &urr.measurement_period {
				self.reporting_peroid_us = period.value as u64 * 1000_000;
				self.next_reporting_ts_us = ts + self.reporting_peroid_us;
			}
		}

		self.report_stop_of_traffic = urr.reporting_triggers.getSTOPT() != 0;
		self.report_start_of_traffic = urr.reporting_triggers.getSTART() != 0;

		self.rule_applied = true;
	}
	pub fn generate_report(&mut self, ts: u64, deletion: bool) -> Option<BackendReport> {
		let mut report_generated = false;

		let volume_measurement: Option<VolumeMeasurement> = Some(
			VolumeMeasurement {
				total_volume: Some(self.ul_bytes + self.dl_bytes),
				uplink_volume: Some(self.ul_bytes),
				downlink_volume: Some(self.dl_bytes),
				total_packets: Some(self.ul_pkts + self.dl_pkts),
				uplink_packets: Some(self.ul_pkts),
				downlink_packets: Some(self.dl_pkts),
			}
		);

		let total_traffic_usage_time_us = match self.traffic_state {
			TrafficState::Ongoing => {
				self.cum_traffic_usage_time_us + ts - self.last_traffic_state_change_ts_us
			},
			TrafficState::Paused => {
				self.cum_traffic_usage_time_us
			},
		};
		let duration_measurement: Option<DurationMeasurement> = Some(
			DurationMeasurement {
				seconds: (total_traffic_usage_time_us as f64 / 1000000.0f64).round() as u32,
			}
		);

		let mut usage_report_trigger = libpfcp::models::UsageReportTrigger(0);

		//info!("[SEID={},URR_ID={}] ul_bytes={}", self.seid, self.urr_id, self.ul_bytes);

		if self.ul_bytes >= self.ul_threshold {
			info!("[SEID={}] generting due to UL vol reach {}/{}", self.seid, self.ul_bytes, self.ul_threshold);
			report_generated = true;
			self.ul_bytes = 0;
			self.ul_threshold = u64::MAX; // TODO: this is incorrect, spec says you reapply the threshold
			usage_report_trigger.setVOLTH(1);
		}

		if self.dl_bytes >= self.dl_threshold {
			report_generated = true;
			self.dl_bytes = 0;
			self.dl_threshold = u64::MAX; // TODO: this is incorrect, spec says you reapply the threshold
			usage_report_trigger.setVOLTH(1);
		}

		if total_traffic_usage_time_us >= self.traffic_usage_thres {
			report_generated = true;
			self.cum_traffic_usage_time_us = 0; // reset to 0, 
			self.traffic_usage_thres = u64::MAX;
			if self.traffic_state == TrafficState::Ongoing {
				self.last_traffic_state_change_ts_us = ts;
			}
			usage_report_trigger.setTIMTH(1);
		}

		if ts >= self.next_reporting_ts_us {
			report_generated = true;
			usage_report_trigger.setPERIO(1);
			self.next_reporting_ts_us += self.reporting_peroid_us;
		}

		let mut traffic_start = None;
		let mut traffic_stop = None;

		if self.report_start_of_traffic {
			if self.traffic_state_changed && self.traffic_state == TrafficState::Ongoing {
				info!("generting due to report_start_of_traffic");
				report_generated = true;
				usage_report_trigger.setSTART(1);
				traffic_start = Some(TimeOfFirstPacket::new(Utc::now()));
				self.report_start_of_traffic = false;
			}
		}

		if self.report_stop_of_traffic {
			if self.traffic_state_changed && self.traffic_state == TrafficState::Paused {
				info!("generting due to report_stop_of_traffic, last_traffic_state_change_ts_us={}", self.last_traffic_state_change_ts_us);
				report_generated = true;
				usage_report_trigger.setSTOPT(1);
				traffic_stop = Some(TimeOfLastPacket::new(Utc::now()));
				self.report_stop_of_traffic = false;
			}
		}

		self.traffic_state_changed = false;

		if report_generated || deletion {
			let ur = libpfcp::messages::UsageReport {
				urr_id: URR_ID { rule_id: self.urr_id },
				ur_seqn: UR_SEQN { sqn: self.urr_sqn_gen.inc() as _ },
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
			Some(report)
		} else {
			None
		}
	}

	pub fn traffic_pause(&mut self, diff: u64, ts: u64) {
		if self.traffic_state == TrafficState::Paused {
			return;
		}
		self.last_traffic_state_change_ts_us = ts;
		self.traffic_state = TrafficState::Paused;
		self.cum_traffic_usage_time_us += diff;
		self.traffic_state_changed = true;
	}

	pub fn traffic_resume(&mut self, diff: u64, ts: u64) {
		if self.traffic_state == TrafficState::Ongoing {
			return;
		}
		self.last_traffic_state_change_ts_us = ts;
		self.traffic_state = TrafficState::Ongoing;
		self.traffic_state_changed = true;
	}
}

struct UsageReportingPfcpContext {
	pub urr_entries: linear_map::LinearMap<u32, SingleUsageReportingEntry>,
	pub maid2reporting_entry: linear_map::LinearMap<u32, (bool, LinearSet<u32>)>,
	pub maid2value_postqos: linear_map::LinearMap<u32, (u64, u64)>,
	pub maid2value_preqos: linear_map::LinearMap<u32, (u64, u64)>,
}

impl UsageReportingPfcpContext {
	pub fn new() -> Self {
		Self {
			urr_entries: linear_map::LinearMap::new(),
			maid2reporting_entry: linear_map::LinearMap::new(),
			maid2value_postqos: linear_map::LinearMap::new(),
			maid2value_preqos: linear_map::LinearMap::new(),
		}
	}
	pub fn get_ma_id_vol_thres(&self) -> Vec<MaIdVolThresholdPair> {
		let mut ret = vec![];
		for (ma_id, (is_ul, urr_ids)) in self.maid2reporting_entry.iter() {
			if urr_ids.len() == 0 {
				continue;
			}
			let urr_id = urr_ids.iter().next().unwrap();
			let urr = self.urr_entries.get(urr_id).unwrap();
			if *is_ul && urr.ul_threshold != u64::MAX {
				ret.push(MaIdVolThresholdPair {
					maid: *ma_id,
					vol_thres: urr.ul_threshold,
				})
			}
			if !is_ul && urr.dl_threshold != u64::MAX {
				ret.push(MaIdVolThresholdPair {
					maid: *ma_id,
					vol_thres: urr.dl_threshold,
				})
			}
		}
		ret
	}
	pub async fn update_maid_value_postqos(&mut self, ma_id: u32, bytes: u64, pkts: u64, _: u64, ur_tx: &Sender<BackendReport>) {
		if let Some((old_bytes, old_pkts)) = self.maid2value_postqos.get_mut(&ma_id) {
			let pkt_diff = pkts - *old_pkts;
			if let Some((is_ul, urr_ids)) = self.maid2reporting_entry.get_mut(&ma_id) {
				for urr_id in urr_ids.iter() {
					if let Some(entry) = self.urr_entries.get_mut(urr_id) {
						let ts = crate::context::GLOBAL_TIMER.get().unwrap().get_cur_us();
						if *is_ul { // bad for branch predictor?
							entry.ul_bytes += bytes - *old_bytes;
							entry.ul_pkts += pkt_diff;
						} else {
							entry.dl_bytes += bytes - *old_bytes;
							entry.dl_pkts += pkt_diff;
						}
						entry.last_push_ts_us = ts;
						if pkt_diff != 0 {
							let diff = ts - entry.last_traffic_change_ts_us;
							entry.last_traffic_change_ts_us = ts;
							entry.traffic_resume(diff, ts);
						} else {
							if ts - entry.last_traffic_change_ts_us >= entry.traffic_inactivity_timer_us {
								entry.traffic_pause(ts - entry.last_traffic_state_change_ts_us, ts);
							}
						}
					}
				}
			}
			*old_bytes = bytes;
			*old_pkts = pkts;
		}
		let reports = self.generate_report(false);
		for r in reports {
			ur_tx.send(r).await.unwrap();
		}
	}
	pub fn update_maid_value_preqos(&mut self, ma_id: u32, bytes: u64, pkts: u64, pull_ts_us: u64) {
		if let Some((old_bytes, old_pkts)) = self.maid2value_preqos.get_mut(&ma_id) {
			if let Some((is_ul, urr_ids)) = self.maid2reporting_entry.get_mut(&ma_id) {
				for urr_id in urr_ids.iter() {
					if let Some(entry) = self.urr_entries.get_mut(urr_id) {
						if *is_ul { // bad for branch predictor?
							entry.preqos_ul_bytes += bytes - *old_bytes;
							entry.preqos_ul_pkts += pkts - *old_pkts;
						} else {
							entry.preqos_dl_bytes += bytes - *old_bytes;
							entry.preqos_dl_pkts += pkts - *old_pkts;
						}
					}
				}
			}
			*old_bytes = bytes;
			*old_pkts = pkts;
		}
	}
	pub fn delete_maid(&mut self, ma_id: u32) {
		self.maid2reporting_entry.remove(&ma_id);
		self.maid2value_postqos.remove(&ma_id);
		self.maid2value_preqos.remove(&ma_id);
	}
	pub fn generate_report(&mut self, deletion: bool) -> Vec<BackendReport> {
		let mut ret = vec![];
		for (_, v) in self.urr_entries.iter_mut() {
			let ts = crate::context::GLOBAL_TIMER.get().unwrap().get_cur_us();
			if let Some(report) = v.generate_report(ts, deletion) {
				ret.push(report);
			}
		}
		ret
	}
}



lazy_static! {
	static ref USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP: dashmap::DashMap<u32, Arc<RwLock<UsageReportingPfcpContext>>> = dashmap::DashMap::new();
	static ref USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP: dashmap::DashMap<u64, Arc<RwLock<UsageReportingPfcpContext>>> = dashmap::DashMap::new();
}

pub async fn remove_maids(maids: &Vec<u32>) {
	for maid in maids.iter() {
		if let Some((_, e)) = USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.remove(maid) {
			let mut guard = e.write().await;
			guard.delete_maid(*maid);
		}
	}
}

pub async fn push_counter_values(is_eg: bool, values: Vec<CounterValueArgs>, ur_tx: &Sender<BackendReport>, ts: u64) {
	// info!("============================");
	// info!("{:?}", values);
	for value in values {
		// let guard = USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.read().await;
		// guard.insert(u32::MAX, Arc::new(RwLock::new(UsageReportingPfcpContext::new())));
		// if let Some(a) = USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.get(&u32::MAX) {
		// 	println!("haha");
		// }
		if let Some(v) = USAGE_REPORTING_GLOBAL_CONTEXT_MAID_MAP.get(&value.ma_id) {
			let mut guard = v.write().await;
			if is_eg {
				//println!("update EG");
				guard.update_maid_value_postqos(value.ma_id, value.bytes, value.pkts, ts, ur_tx).await;
			} else {
				//println!("update IG");
				guard.update_maid_value_preqos(value.ma_id, value.bytes, value.pkts, ts);
			}
		}
	}
}

pub fn create_global_usage_reporting_context() {
	let mut d: dashmap::DashMap<u32,u32> = dashmap::DashMap::new();
	for i in d.iter_mut() {

	}
}

fn usage_reporting_thread(ur_tx: Sender<BackendReport>) {	
	let runtime = tokio::runtime::Builder::new_current_thread().thread_name("Tofino UPF usage reporting loop thread").enable_time().build().unwrap();
	let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
	runtime.block_on(async {
		info!("Active usage reporting thread running");
		loop {
			let start = std::time::Instant::now();
			
			let mut ctr = 0usize;
			let mut reported = 0usize;
			let mut n_reports = 0usize;
			for item in USAGE_REPORTING_GLOBAL_CONTEXT_PFCP_MAP.iter() {
				let mut guard = item.value().write().await;
				let reports = guard.generate_report(false);
				if reports.len() != 0 {
					reported += 1;
				}
				for r in reports {
					ur_tx.send(r).await.unwrap();
					n_reports += 1;
				}
				ctr += 1;
			}
	
			let elp = start.elapsed();
			//info!("Loop through {} entries used {}ms, generated {} reports from {} reported sessions", ctr, elp.as_secs_f32() * 1000.0f32, n_reports, reported);
			if elp.as_millis() <= 1_000 {
				tokio::time::sleep(std::time::Duration::from_millis(1_000 - elp.as_millis() as u64)).await;
			}
		}
	});
}

pub fn create_usage_reporting_threads(ur_tx: Sender<BackendReport>, settings: URRTestAuxSettings) {
	std::thread::spawn(move || {
		usage_reporting_thread(ur_tx)
	});
}
