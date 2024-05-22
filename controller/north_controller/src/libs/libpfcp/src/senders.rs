use tokio::{sync::Mutex, task};

use async_trait::async_trait;
use crate::{IDAllocator, PFCPError, PFCPModel, PFCPSessionRules, PFCPSessionTransactionState, messages::{AssociationSetupRequest, AssociationSetupResponse, CreatedPDR, HeartbeatRequest, HeartbeatResponse, PFCPSessionDeletionRequest, PFCPSessionEstablishmentRequest, PFCPSessionEstablishmentResponse}, models::{UPFunctionFeatures, CPFunctionFeatures, Cause, F_SEID, NodeID, PDNType, RecoveryTimeStamp, AgoUpfPerfReport}};

use super::{PFCPNodeManager, PFCPRequestFuture, PFCPRequestFutureSharedState, PFCPSessionCP, ResponseMatchingTuple, models::PFCPHeader, models::PFCPHeaderFlags, PFCPSessionState};
use core::time;
use std::{collections::HashMap, net::{IpAddr, SocketAddr, UdpSocket}, sync::{Arc}, time::Duration};
use tokio::sync::RwLock;

impl PFCPNodeManager {
	fn send_impl(&self, msg: &Vec<u8>, matching_tuple: &ResponseMatchingTuple, dst_port_override: Option<u16>) -> PFCPRequestFuture {
		let shared_state = Arc::new(std::sync::RwLock::new(PFCPRequestFutureSharedState {
			response: None,
			waker: None
		}));
		let future = PFCPRequestFuture {
			shared_state: shared_state.clone(),
		};
		
		if let Some(unfinished_ongoing) = super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.insert(matching_tuple.clone(), shared_state.clone()) {
			unreachable!();
		}
		super::PFCP_GLOBAL_REQUEST_SOCKET.get().unwrap().send_to(msg.as_slice(), SocketAddr::from((self.remote_ip, dst_port_override.map_or(8805, |f| f)))).unwrap();
		future
	}
	async fn send(&mut self, msg_type: u8, body: Vec<u8>, timeout_retry_count: Option<(std::time::Duration, u8)>, dst_port_override: Option<u16>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		let guard = self.up_lock.lock().await;
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
					super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_tuple);
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
	pub async fn new(
		cp_ip: IpAddr,
		up_ip: IpAddr,
		pfcp_entity_startup_time: chrono::DateTime<chrono::offset::Utc>,
		dst_port_override: Option<u16>,
		up_func: Option<UPFunctionFeatures>,
		cp_func: Option<CPFunctionFeatures>
	) -> Option<Self> {
		let mut ret = PFCPNodeManager {
			local_ip: cp_ip,
			remote_ip: up_ip,
			seq: Arc::new(Mutex::new(0)),
			up_lock: Mutex::new(0),
			seid_pool: Arc::new(Mutex::new(IDAllocator::new())),
			managed_pfcp_sessions: dashmap::DashMap::new(),
			dst_port_override: dst_port_override
		};
		// send association request
		let associate_request = AssociationSetupRequest {
			node_id: NodeID::from_ip(cp_ip),
			recovery_time_stamp: RecoveryTimeStamp::new(pfcp_entity_startup_time),
			up_function_features: up_func,
			cp_function_features: cp_func
		};
		match ret.send(AssociationSetupRequest::ID as u8, associate_request.encode(), Some((Duration::from_secs(5), 3)), dst_port_override).await {
			Ok((_, resp_body)) => {
				let mut resp_body = resp_body.clone();
				let resp = match AssociationSetupResponse::decode(resp_body.as_mut_slice()) {
					Ok(a) => a,
					Err(e) => { println!("decode error: {:?}", e); return None; }
				};
				if resp.cause == Cause::RequestAccepted {
					Some(ret)
				} else {
					None
				}
			},
			Err(_) => None
		}
	}
	pub async fn heartbeat(
		&mut self, 
		pfcp_entity_startup_time: chrono::DateTime<chrono::offset::Utc>,
		dst_port_override: Option<u16>
	) -> Result<(), PFCPError> {
		let heartbeat_request = HeartbeatRequest { recovery_time_stamp: RecoveryTimeStamp::new(pfcp_entity_startup_time), source_ip_address: None };
		match self.send(HeartbeatRequest::ID as u8, heartbeat_request.encode(), Some((Duration::from_secs(5), 3)), dst_port_override).await {
			Ok((_, resp_body)) => {
				let mut resp_body = resp_body.clone();
				let resp = match HeartbeatResponse::decode(resp_body.as_mut_slice()) {
					Ok(a) => a,
					Err(_) => { return Err(PFCPError::new("Failed to decode Heartbeat response")); }
				};
				Ok(())
			},
			Err(_) => Err(PFCPError::new("No Heartbeat response"))
		}
	}
	pub async fn create_pfcp_session(&self, rules: Option<PFCPSessionRules>, pdn_type: Option<PDNType>) -> Result<(Arc<RwLock<PFCPSessionCP>>, Vec<CreatedPDR>, Option<AgoUpfPerfReport>), PFCPError> {
		let local_seid = self.seid_pool.lock().await.allocate()?;
		let mut pfcp_sess = PFCPSessionCP {
			local_f_seid: F_SEID::new(self.local_ip, local_seid),
			remote_ip: self.remote_ip,
			remote_seid: None,
			seq: self.seq.clone(),
			request_lock: Mutex::new(0),
			state: PFCPSessionState::Creating,
			seid_alloc: Some(self.seid_pool.clone()),
			transaction_state: PFCPSessionTransactionState::Commited,
			active_rules: rules.as_ref().map_or_else(PFCPSessionRules::new, |f| f.clone()),
			last_checkpoint: None,
			dst_port_override: self.dst_port_override
		};
		// println!("Creating PFCP Session in UPF[{}] CP-SEID[{}]", self.remote_ip, local_seid);

		let request = match rules {
			Some(r) => {
				PFCPSessionEstablishmentRequest {
					node_id: NodeID::from_ip(self.local_ip),
					cp_f_seid: F_SEID::new(self.local_ip, local_seid),
					create_pdr: r.pdr_map.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>(),
					create_far: r.far_map.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>(),
					create_urr: r.urr_map.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>(),
					create_qer: r.qer_map.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>(),
					create_bar: {
						let bars = r.bar_map.iter().map(|(_, o)| o.clone()).collect::<Vec<_>>();
						if bars.len() != 0 {
							Some(bars[0].clone())
						} else {
							None
						}
					},
					create_traffic_endpoint: vec![],
					pdn_type: pdn_type,
				}
			}
			None => {
				PFCPSessionEstablishmentRequest {
					node_id: NodeID::from_ip(self.local_ip),
					cp_f_seid: F_SEID::new(self.local_ip, local_seid),
					create_pdr: vec![],
					create_far: vec![],
					create_urr: vec![],
					create_qer: vec![],
					create_bar: None,
					create_traffic_endpoint: vec![],
					pdn_type: pdn_type,
				}
			}
		};
		let (header, response_body) = match pfcp_sess.SendSessionEstablishmentRequest(request.encode()).await {
			Ok(a) => a,
			Err(e) => {
				println!("Timeout sending SessionEstablishmentRequest");
				return Err(PFCPError::new("request timed out"));
			}
		};

		let response = match PFCPSessionEstablishmentResponse::decode(response_body.clone().as_mut_slice()) {
			Ok(a) => a,
			Err(e) => { 
				println!("failed to decode response, {}", e);
				return Err(PFCPError::new("failed to decode response"));
			}
		};
		if response.cause != Cause::RequestAccepted {
			return Err(PFCPError::new("request rejected"));
		};
		let up_f_seid = match response.up_f_seid {
			Some(a) => a,
			None => { return Err(PFCPError::new("response semantic error")); }
		};

		// remote SEID is all we want
		pfcp_sess.remote_seid = Some(up_f_seid.seid);
		pfcp_sess.state = PFCPSessionState::Activated;
		// println!("PFCP Session created in UPF[{}] CP-SEID[{}] UP-SEID[{}]", self.remote_ip, local_seid, up_f_seid.seid);

		let rules = &mut pfcp_sess.active_rules;
		for resp_pdr in response.created_pdr.iter() {
			if let Some(pdr) = rules.pdr_map.get_mut(&resp_pdr.pdr_id.rule_id) {
				// fill assigned TEID
				pdr.pdi.local_f_teid = resp_pdr.local_f_teid.clone();
			}
		}

		let pfcp_sess_ptr = Arc::new(RwLock::new(pfcp_sess));
		self.managed_pfcp_sessions.insert(local_seid, pfcp_sess_ptr.clone());
		Ok((pfcp_sess_ptr, response.created_pdr, response.ago_perf))
	}
}

impl PFCPSessionCP {
	fn send_impl(&self, msg: &Vec<u8>, matching_tuple: &ResponseMatchingTuple, dst_port_override: Option<u16>) -> PFCPRequestFuture {
		let shared_state = Arc::new(std::sync::RwLock::new(PFCPRequestFutureSharedState {
			response: None,
			waker: None
		}));
		let future = PFCPRequestFuture {
			shared_state: shared_state.clone(),
		};
		
		if let Some(unfinished_ongoing) = super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.insert(matching_tuple.clone(), shared_state.clone()) {
			unreachable!();
		}
		super::PFCP_GLOBAL_REQUEST_SOCKET.get().unwrap().send_to(msg.as_slice(), SocketAddr::from((self.remote_ip, dst_port_override.map_or(8805, |f| f)))).unwrap();
		future
	}
	async fn send(&mut self, msg_type: u8, body: Vec<u8>, timeout_retry_count: Option<(std::time::Duration, u8)>, dst_port_override: Option<u16>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		let guard = self.request_lock.lock().await;

		let remote_seid = self.remote_seid.map_or(0, |f| f);

		let matching_tuple;
		let length;
		let seq_n;
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
				seid: Some(remote_seid),
				seq: *seq,
				priority: None,
			}.encode();
			seq_n = *seq;
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
					println!("retrying");
					er.replace(e);
					super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_tuple);
					continue;
				} else {
					return Ok(ret.unwrap())
				}
			}
			println!("Timed out at attempt to send session message to {}, SEID = {}", self.remote_ip, remote_seid);
			Err(er.unwrap())
		} else {
			Ok(self.send_impl(&msg, &matching_tuple, dst_port_override).await)
		}
	}
	pub async fn SendSessionEstablishmentRequest(&mut self, msg: Vec<u8>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		self.send(50, msg, Some((Duration::from_secs(50), 3)), self.dst_port_override).await
	}
	pub async fn SendSessionModificationRequest(&mut self, msg: Vec<u8>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		self.send(52, msg, Some((Duration::from_secs(50), 3)), self.dst_port_override).await
	}
	pub async fn SendSessionDeletionRequest(&mut self, msg: Vec<u8>) -> Result<(PFCPHeader, Vec<u8>), tokio::time::error::Elapsed> {
		self.send(54, msg, Some((Duration::from_secs(50), 3)), self.dst_port_override).await
	}
}

use futures::executor;

impl Drop for PFCPSessionCP {
	fn drop(&mut self) {
		if self.state == PFCPSessionState::Activated {
			executor::block_on(async move {
				let request = PFCPSessionDeletionRequest {};
				match self.SendSessionDeletionRequest(request.encode()).await {
					Ok(_) => {}
					Err(_) => {}
				};
				self.seid_alloc.as_ref().unwrap().lock().await.free(self.local_f_seid.seid);
			});
		}
	}
}
