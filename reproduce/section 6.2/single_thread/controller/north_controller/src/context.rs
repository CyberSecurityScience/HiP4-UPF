use lazy_static::lazy_static;
use log::info;
use std::{collections::{HashMap, HashSet}, net::{Ipv4Addr, Ipv6Addr}, sync::Mutex};

use libpfcp::{models::F_SEID, IDAllocator, PFCPSessionRules};
use once_cell::sync::{Lazy, OnceCell};
use std::iter::FromIterator;
use tokio::{runtime::Handle, sync::{self}};

use crate::{datapath::BackendInterface, tscns::TSCNS};
use std::sync::{Arc, RwLock};

use super::n4::GLOBAL_PFCP_CONTEXT;

#[derive(Debug, Clone)]
pub struct UPFParameters {
	pub nfctx: crate::NetworkFunctionContext,
	/// IP(Control plane) of myself
	pub node_ip: std::net::IpAddr,
	/// Range of IPv4(User plane) assigned to this UPF, node
	pub assigned_ipv4: Option<Ipv4Addr>,
	pub assigned_ipv6: Option<Ipv6Addr>,

	pub p4json_file: String,
	pub p4info_file: String,
}

pub static GLOBAL_TIMER: OnceCell<TSCNS> = OnceCell::new();
pub static UPF_PARAMETERS: OnceCell<UPFParameters> = OnceCell::new();
pub static BACKEND_TIME_REFERENCE: OnceCell<std::time::Instant> = OnceCell::new();
pub static BACKEND: OnceCell<Box<dyn BackendInterface + Sync + Send>> = OnceCell::new();

pub struct TEIDPool {
	pub alloc: IDAllocator<u32>,
	references: HashMap<u64, HashSet<u32>>,
	references_teid: HashMap<u32, HashSet<u64>>,
}
impl TEIDPool {
	pub fn new() -> Self {
		Self {
			alloc: IDAllocator::new(),
			references: HashMap::new(),
			references_teid: HashMap::new(),
		}
	}
	pub fn allocate(&mut self) -> u32 {
		let ret = self.alloc.allocate().unwrap();
		//info!("allocating TEID {}", ret);
		ret
	}
	/// Let TEID pool know this session is using this set of TEIDs \
	/// if a TEID is no longer used by any sessions, it will be freed
	pub fn mark_reference(&mut self, seid: u64, teids: Vec<u32>) {
		let new_teids = HashSet::<u32>::from_iter(teids.iter().cloned());
		let old_teids = self
			.references
			.get(&seid)
			.map_or_else(HashSet::new, |f| f.clone());
		let added_teids: HashSet<u32> = new_teids.difference(&old_teids).cloned().collect();
		let modified_teids: HashSet<u32> = new_teids
			.symmetric_difference(&old_teids)
			.cloned()
			.collect();
		let deleted_teids: HashSet<u32> =
			modified_teids.difference(&added_teids).cloned().collect();
		if let Some(ref1) = self.references.get_mut(&seid) {
			ref1.clone_from(&new_teids);
		} else {
			self.references.insert(seid, new_teids);
		};
		for added in added_teids {
			if let Some(ref2) = self.references_teid.get_mut(&added) {
				ref2.insert(seid);
			} else {
				let hs = HashSet::<u64>::from_iter([seid].iter().cloned());
				self.references_teid.insert(added, hs);
			}
		}
		for deleted in deleted_teids {
			if let Some(ref2) = self.references_teid.get_mut(&deleted) {
				ref2.remove(&seid);
				if ref2.len() == 0 {
					self.alloc.free(deleted);
					//info!("freeing TEID {}", deleted);
				}
			} else {
				unreachable!()
			}
		}
	}
}

lazy_static! {
	pub static ref TEIDPOOL: RwLock<TEIDPool> = RwLock::new(TEIDPool::new());
}

pub static ASYNC_RUNTIME: Lazy<Mutex<Option<Handle>>> = Lazy::new(Default::default);


pub fn get_async_runtime() -> Handle {
	crate::context::ASYNC_RUNTIME.lock().unwrap().as_ref().unwrap().clone()
}
