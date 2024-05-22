mod node_handlers;
mod session_handlers;

use lazy_static::lazy_static;
use libpfcp::{
	messages::{CreateFAR, CreatePDR, CreateQER, CreateURR},
	models::{NodeID, FAR_ID, F_SEID, PDR_ID, QER_ID, PFCPHeaderFlags, PFCPHeader},
	PFCPSessionRulesUP, PFCPSessionState, ResponseMatchingTuple, PFCPRequestFuture, PFCPRequestFutureSharedState,
};
use log::info;
use once_cell::sync::OnceCell;
use std::{
	collections::HashMap,
	net::{IpAddr, Ipv4Addr, SocketAddr},
	sync::{Arc}, time::Duration,
};
use tokio::{runtime::Handle, sync, sync::RwLock};
use log::error;

use crate::{
	context::{self, UPF_PARAMETERS},
	datapath::{BackendInterface, PacketPipeline, BackendReport, OperationPerfStats},
};

pub struct PFCPSessionUP {
	pub local_f_seid: F_SEID,
	pub remote_ip: std::net::IpAddr,
	pub remote_seid: u64,

	/// Sequence number for all messages, shares with parent PFCPNodeManager/UPF's sequence counter
	pub seq: Arc<sync::Mutex<u32>>,
	/// Make sure only one PFCP request is on going
	pub request_lock: sync::Mutex<i32>,
	/// State
	pub state: PFCPSessionState,

	pub active_rules: PFCPSessionRulesUP,
	pub active_pipelines: Vec<PacketPipeline>,
}

impl PFCPSessionUP {
	fn send_impl(&self, msg: &Vec<u8>, matching_tuple: &ResponseMatchingTuple, dst_port_override: Option<u16>) -> PFCPRequestFuture {
		let shared_state = Arc::new(std::sync::RwLock::new(PFCPRequestFutureSharedState {
			response: None,
			waker: None
		}));
		let future = PFCPRequestFuture {
			shared_state: shared_state.clone(),
		};
		
		if let Some(unfinished_ongoing) = libpfcp::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.insert(matching_tuple.clone(), shared_state.clone()) {
			unreachable!();
		}
		libpfcp::PFCP_GLOBAL_REQUEST_SOCKET.get().unwrap().send_to(msg.as_slice(), SocketAddr::from((self.remote_ip, dst_port_override.map_or(8805, |f| f)))).unwrap();
		future
	}
	async fn send(&mut self, msg_type: u8, body: Vec<u8>, timeout_retry_count: Option<(std::time::Duration, u8)>, dst_port_override: Option<u16>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		let guard = self.request_lock.lock().await;
		let matching_tuple;
		let length;
		let mut msg;
		{
			let mut seq = self.seq.lock().await;
			matching_tuple = ResponseMatchingTuple {
				remote_ip: self.remote_ip,
				seq: *seq
			};
			length = body.len() + 4;
			assert!(length < 0xffff);
			msg = PFCPHeader {
				flags: PFCPHeaderFlags(0b00100000),
				msg_type: msg_type,
				length: length as u16,
				seid: None,
				seq: *seq,
				priority: None,
			}.encode();
			*seq += 1;
			if *seq > 0x00_ff_ff_ffu32 {
				*seq = 0;
			}
		}
		msg.append(&mut body.clone());
		if let Some((timeout, retry_count)) = timeout_retry_count {
			let mut er = None;
			for _ in 0..retry_count {
				let ret = tokio::time::timeout(timeout, self.send_impl(&msg, &matching_tuple, dst_port_override)).await;
				if let Err(e) = ret {
					er.replace(e);
					libpfcp::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_tuple);
					continue;
				} else {
					return Ok(ret.unwrap())
				}
			}
			println!("Timed out at attempt to send node message to {}", self.remote_ip);
			Err(er.unwrap())
		} else {
			Ok(self.send_impl(&msg, &matching_tuple, dst_port_override).await)
		}
	}
	async fn send_seid(&mut self, seid: u64, msg_type: u8, body: Vec<u8>, timeout_retry_count: Option<(std::time::Duration, u8)>, dst_port_override: Option<u16>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		let guard = self.request_lock.lock().await;
		let matching_tuple;
		let length;
		let mut msg;
		{
			let mut seq = self.seq.lock().await;
			matching_tuple = ResponseMatchingTuple {
				remote_ip: self.remote_ip,
				seq: *seq
			};
			length = body.len() + 8 + 4;
			assert!(length < 0xffff);
			msg = PFCPHeader {
				flags: PFCPHeaderFlags(0b00100001),
				msg_type: msg_type,
				length: length as u16,
				seid: Some(seid),
				seq: *seq,
				priority: None,
			}.encode();
			*seq += 1;
			if *seq > 0x00_ff_ff_ffu32 {
				*seq = 0;
			}
		}
		msg.append(&mut body.clone());
		if let Some((timeout, retry_count)) = timeout_retry_count {
			let mut er = None;
			for _ in 0..retry_count {
				let ret = tokio::time::timeout(timeout, self.send_impl(&msg, &matching_tuple, dst_port_override)).await;
				if let Err(e) = ret {
					er.replace(e);
					libpfcp::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_tuple);
					continue;
				} else {
					return Ok(ret.unwrap())
				}
			}
			println!("Timed out at attempt to send node message to {}", self.remote_ip);
			Err(er.unwrap())
		} else {
			Ok(self.send_impl(&msg, &matching_tuple, dst_port_override).await)
		}
	}
	pub async fn SendSessionReportRequest(&mut self, seid: u64, msg: Vec<u8>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		self.send_seid(seid, 56, msg, Some((Duration::from_secs(5), 3)), None).await
	}
}

pub struct PFCPContext {
	/// SMF IP/SMFSet ID
	pub smf_id: NodeID,

	/// Sequence number for all messages
	pub seq: Arc<sync::Mutex<u32>>,
	/// Allocator for PFCP session ID
	pub PfcpSessionIdAllocator: libpfcp::IDAllocator<u64>,
	/// Allocator for TEID
	pub TeidIdAllocator: libpfcp::IDAllocator<u32>,
	/// Map PFCP session ID to PFCP session
	pub PfcpSessions: HashMap<u64, Arc<RwLock<PFCPSessionUP>>>,
}

impl PFCPContext {
	pub async fn release_all(&mut self) {
		for seid in self.PfcpSessions.keys() {
			info!("[SEID={}] Deleting CreateURRPFCP session", seid);
			delete_pfcp_session(*seid).await;
		}
		// self.TeidIdAllocator.reset();
		// self.PfcpSessionIdAllocator.reset();
	}
}

pub async fn allocate_pfcp_session(cp_f_seid: &F_SEID) -> Option<Arc<RwLock<PFCPSessionUP>>> {
	let upf_para = UPF_PARAMETERS.get().unwrap();
	let mut pfcp_ctx_opt = GLOBAL_PFCP_CONTEXT.write().await;
	let pfcp_ctx = match pfcp_ctx_opt.as_mut() {
		Some(a) => a,
		None => {
			error!("pfcp_ctx_opt.as_mut() failed");
			return None;
		}
	};
	let sess_map_locked = &mut pfcp_ctx.PfcpSessions;
	let seid = match { pfcp_ctx.PfcpSessionIdAllocator.allocate() } {
		Ok(a) => a,
		Err(_) => {
			error!("pfcp_ctx.PfcpSessionIdAllocator.allocate() failed");
			return None;
		}
	};
	let mut sess = PFCPSessionUP {
		local_f_seid: F_SEID::new(upf_para.node_ip, seid),
		remote_ip: cp_f_seid.to_single_ip().unwrap(),
		remote_seid: cp_f_seid.seid,
		seq: pfcp_ctx.seq.clone(),
		request_lock: sync::Mutex::new(0),
		state: PFCPSessionState::Activated,
		active_rules: PFCPSessionRulesUP::new(),
		active_pipelines: vec![],
	};
	let guard: sync::RwLockReadGuard<'_, Option<PredefinedRules>> = PREDEFINED_RULES.read().await;
	let predefined_rules = guard.as_ref().unwrap();
	// populate with predefined FARs/QERs/URRs
	for far in predefined_rules.FARs.iter() {
		sess.active_rules
			.far_map
			.insert(far.far_id.rule_id, far.clone());
	}
	for urr in predefined_rules.URRs.iter() {
		sess.active_rules
			.urr_map
			.insert(urr.urr_id.rule_id, urr.clone());
	}
	for qer in predefined_rules.QERs.iter() {
		sess.active_rules
			.qer_map
			.insert(qer.qer_id.rule_id, qer.clone());
	}
	let sess_handle = Arc::new(RwLock::new(sess));
	if sess_map_locked.contains_key(&seid) {
		unreachable!()
	} else {
		//info!("New PFCP Session[SEID={}] created", seid);
		sess_map_locked.insert(seid, sess_handle.clone());
		Some(sess_handle.clone())
	}
}

pub async fn find_pfcp_session_by_local_seid(id: u64) -> Option<Arc<RwLock<PFCPSessionUP>>> {
	let pfcp_ctx_opt = GLOBAL_PFCP_CONTEXT.read().await;
	let pfcp_ctx = match pfcp_ctx_opt.as_ref() {
		Some(a) => a,
		None => {
			return None;
		}
	};
	if let Some(pfcp_session) = pfcp_ctx.PfcpSessions.get(&id) {
		Some(pfcp_session.clone())
	} else {
		None
	}
}

pub async fn delete_pfcp_session(seid: u64) -> (OperationPerfStats, Vec<BackendReport>) {
	let mut pfcp_ctx_opt = GLOBAL_PFCP_CONTEXT.write().await;
	let pfcp_ctx = match pfcp_ctx_opt.as_mut() {
		Some(a) => a,
		None => {
			return (OperationPerfStats { stats1: 0, stats2: 0, stats3: 0, stats4: 0 }, vec![]);
		}
	};
	if let Some(pfcp_session) = pfcp_ctx.PfcpSessions.remove(&seid) {
		let mut sess = pfcp_session.write().await;
		sess.state = PFCPSessionState::Deleted;
		// mark TEID reference (remove all TEID reference of this Session)
		crate::context::TEIDPOOL
			.write()
			.unwrap()
			.mark_reference(sess.local_f_seid.seid, Vec::new());

		let be = crate::context::BACKEND.get().unwrap();
		let perf_timer_1 = std::time::Instant::now();
		let (mut stats, reports) = be.delete_session(sess.local_f_seid.seid).await;

		pfcp_ctx.PfcpSessionIdAllocator.free(seid);
		let perf_timer_1_val = perf_timer_1.elapsed().as_micros() as u64;
		stats.stats3 = perf_timer_1_val;
		(stats, reports)
	} else {
		(OperationPerfStats { stats1: 0, stats2: 0, stats3: 0, stats4: 0 }, vec![])
	}
}

pub struct PredefinedRules {
	pub PDRs: HashMap<String, CreatePDR>,
	pub FARs: Vec<CreateFAR>,
	pub URRs: Vec<CreateURR>,
	pub QERs: Vec<CreateQER>,
}

#[derive(Debug, Clone)]
pub struct N4Handlers;

lazy_static! {
	pub static ref GLOBAL_PFCP_CONTEXT: RwLock<Option<PFCPContext>> = RwLock::new(None);
	pub static ref PREDEFINED_RULES: RwLock<Option<PredefinedRules>> = RwLock::new(None);
}
