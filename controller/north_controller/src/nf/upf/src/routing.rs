
use std::fmt;

use cidr::IpCidr;
use pnet::util::MacAddr;
use serde::{Serialize, Deserialize};


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DeviceInterfaceType {
	N6,
	N3,
	N9
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RoutingEntry {
	pub device_port: u16,
	pub interface: DeviceInterfaceType,
	pub cidr: IpCidr,
	pub src_mac: MacAddr,
	pub dst_mac: MacAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutingTable {
	pub routing: Vec<RoutingEntry>
}
