
use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}, convert::TryInto};
use linear_map::LinearMap;
use log::{info, error};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

use crate::datapath::tofino::bfruntime::bfrt::key_field::{Lpm, Exact};

use super::bfruntime::{bfrt::{*, data_field::IntArray, key_field::{Ternary, Range, MatchType}}, P4TableInfo, self};

pub const NUM_QFI: usize = 4;
pub const NUM_PORTS: usize = 4;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QosPortData {
	pub ports_in_group: Vec<u16>,
	pub max_bitrate_kbps: u64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QosQfiData {
	pub qfi: u8,
	pub physical_qid: u8,
	pub gbr_kbps: u64,
	pub sojourn_target_us: u32,
	pub target_probability: f32,
	pub importance: f32
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QosQfiDataYaml {
	pub qfi: u8,
	pub sojourn_target_us: u32,
	pub gbr: bool,
	pub target_probability: f32,
	pub importance: f32
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeQosQfiData {
	pub qfi: u8,
	pub port_id: u16,
	pub physical_qid: u8,
	pub gbr_kbps: Option<u64>,
	pub sojourn_target_us: u32,
	pub target_probability: f32,
	pub dwrr_weight: u16,
	pub drop_ratio: f32
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListOfRuntimeQosQfiData {
	pub data: Vec<RuntimeQosQfiData>
}

/// Fixed during runtime, set at startup
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QosAndPortData {
	pub port_data: Vec<QosPortData>,
	pub qfi_data: Vec<QosQfiData>
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListOfQosPortData {
	pub port_data: Vec<QosPortData>
}