use std::{
	collections::HashMap,
	net::IpAddr,
	sync::{Arc, RwLock},
	thread,
	time::Duration,
};

use crate::context::UPF_PARAMETERS;

use super::{N4Handlers, PFCPContext, GLOBAL_PFCP_CONTEXT};
use async_trait::async_trait;
use libpfcp::{
	handlers::NodeRequestHandlers,
	messages::{
		AssociationReleaseResponse, AssociationSetupRequest, AssociationSetupResponse,
		HeartbeatResponse,
	},
	models::{Cause, NodeID, RecoveryTimeStamp, UPFunctionFeatures},
	IDAllocator, PFCPModel,
};
use log::info;

#[async_trait]
impl NodeRequestHandlers for N4Handlers {
	async fn handle_association_setup(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let upf_para = UPF_PARAMETERS.get().unwrap();
		let mut up_func = UPFunctionFeatures(0);
		up_func.setBUCP(1);
		up_func.setFTUP(1);
		up_func.setEMPU(1);
		up_func.setUDBC(1);
		up_func.setADPDP(1);
		up_func.setBUNDL(1);
		up_func.setNORP(1);
		up_func.setDDDS(1);
		let mut response = AssociationSetupResponse {
			node_id: NodeID::from_ip(upf_para.node_ip),
			cause: Cause::RequestAccepted,
			recovery_time_stamp: RecoveryTimeStamp::new(upf_para.nfctx.nf_startup_time),
			up_function_features: Some(up_func),
			cp_function_features: None,
		};

		let request = match AssociationSetupRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
				println!("[PFCP] Error: {:?}", e);
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				return response.encode();
			}
		};
		let new_ctx = PFCPContext {
			smf_id: request.node_id,
			seq: Arc::new(tokio::sync::Mutex::new(0)),
			PfcpSessionIdAllocator: IDAllocator::new(),
			PfcpSessions: HashMap::new(),
			TeidIdAllocator: IDAllocator::new(),
		};
		let mut pfcp_ctx_guard = GLOBAL_PFCP_CONTEXT.write().await;
		if let Some(existing_ctx) = pfcp_ctx_guard.as_mut() {
			if existing_ctx.smf_id != response.node_id {
				println!("Replacing existing PFCP association, releasing all sessions now");
				existing_ctx.release_all().await;
			}
		}
		info!(
			"Associated with new remote CP function, smf_id = {:?}",
			new_ctx.smf_id
		);
		pfcp_ctx_guard.replace(new_ctx);
		response.encode()
	}

	async fn handle_heartbeat(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let nfself = UPF_PARAMETERS.get().unwrap();
		let response = HeartbeatResponse {
			recovery_time_stamp: RecoveryTimeStamp::new(nfself.nfctx.nf_startup_time),
		};
		response.encode()
	}

	async fn handle_pfd_management(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		todo!()
	}

	async fn handle_association_update(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		todo!()
	}

	async fn handle_association_release(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		let upf_para = UPF_PARAMETERS.get().unwrap();

		let mut response = AssociationReleaseResponse {
			node_id: NodeID::from_ip(upf_para.node_ip),
			cause: Cause::RequestAccepted,
		};

		let request = match AssociationSetupRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				return response.encode();
			}
		};

		let mut pfcp_ctx_guard = GLOBAL_PFCP_CONTEXT.write().await;
		if let Some(mut existing_ctx) = pfcp_ctx_guard.take() {
			if existing_ctx.smf_id != response.node_id {
				println!("PFCP association release received, releasing all sessions now");
				existing_ctx.release_all().await;
			}
		}

		response.encode()
	}

	async fn handle_node_report(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		todo!()
	}

	async fn handle_session_set_deletion(
		&self,
		header: libpfcp::models::PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8> {
		todo!()
	}
}
