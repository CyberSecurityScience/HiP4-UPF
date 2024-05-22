
use lazy_static::lazy_static;
use libpfcp::models::CPFunctionFeatures;
use log::{info, warn};
use std::{collections::HashMap, net::{self, IpAddr, Ipv4Addr}, sync::{Arc}};
use tokio::sync::RwLock;

pub mod handlers;

#[derive(Debug, Clone)]
pub struct UPFNodeContext {
	pub node: Arc<RwLock<libpfcp::PFCPNodeManager>>,
}

pub struct PFCPNodesGlobalContext {
	pub nodes: RwLock<HashMap<IpAddr, UPFNodeContext>>
}

impl PFCPNodesGlobalContext {
	pub fn new() -> PFCPNodesGlobalContext {
		PFCPNodesGlobalContext {
			nodes: RwLock::new(HashMap::new())
		}
	}
}

lazy_static! {
	pub static ref PFCP_NODES_GLOBAL_CONTEXT: PFCPNodesGlobalContext = PFCPNodesGlobalContext::new();
}

pub async fn add_upf(node_ip: IpAddr, self_ip: IpAddr) -> Option<UPFNodeContext> {
	if let Some(exitsing_context) = { PFCP_NODES_GLOBAL_CONTEXT.nodes.read().await.get(&node_ip) } {
		warn!("UPF already exists: {}", node_ip);
		return Some(exitsing_context.clone());
	};
	let mut cp_func = CPFunctionFeatures(0);
	cp_func.setBUNDL(1);
	match libpfcp::PFCPNodeManager::new(self_ip, node_ip, chrono::Utc::now(), None, None, Some(cp_func)).await {
		Some(node_handler) => {
			info!("Connected to {}", node_ip);
			let node_handler_ptr = Arc::new(RwLock::new(node_handler));
			let ctx = UPFNodeContext {
				node: node_handler_ptr
			};
			PFCP_NODES_GLOBAL_CONTEXT.nodes.write().await.insert(node_ip, ctx.clone());
			Some(ctx)
		},
		None => None
	}
}

pub async fn del_upf(node_ip: IpAddr) {
	if let Some(exitsing_context) = { PFCP_NODES_GLOBAL_CONTEXT.nodes.read().await.get(&node_ip) } {
		info!("Deleting UPF[ip={}]", node_ip);
	} else {
		warn!("Attempting to delete UPF[ip={}] which does not exist", node_ip);
	}
}
