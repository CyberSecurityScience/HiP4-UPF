use std::{sync::Arc, net::Ipv4Addr, time::{SystemTime, Duration, Instant, UNIX_EPOCH}, fs::File, io::Write};

use clap::{Arg, App};
use libpfcp::{messages::{PDI, ForwardingParameters}, models::{ApplyAction, UE_IPAddress, OuterHeaderCreation, F_TEID, QFI, Precedence, OuterHeaderRemovalDescription, OuterHeaderRemoval, URR_ID, PDNType, GateStatus}, PFCPError, PFCPSessionCP};
use log::info;
use n4::UPFNodeContext;
use tokio::sync::RwLock;

use crate::{sender::GtpClient, n4::{handlers::N4Handlers, add_upf}};

// vol-stable-traffic

fn generate_urr_tests_vol(rules: &mut libpfcp::PFCPSessionRules, dl_thres: Option<u64>, ul_thres: Option<u64>) -> Result<URR_ID, PFCPError> {
	use libpfcp::models::*;
	let mut mm = MeasurementMethod(0);
	mm.setVOLUM(1);
	let mut rt = ReportingTriggers(0);
	rt.setVOLTH(1);
	let vol_thres = VolumeThreshold {
		flags: {
			let mut f = VolumeThresholdFlags(0);
			if dl_thres.is_some() {
				f.setDLVOL(1);
			}
			if ul_thres.is_some() {
				f.setULVOL(1);
			}
			f
		},
		total_volume: None,
		uplink_volume: ul_thres,
		downlink_volume: dl_thres
	};
	rules.create_urr(
		mm,
		rt,
		None,
		Some(vol_thres),
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		vec![],
		None,
		None,
		None,
		vec![],
		None,
		vec![],
		vec![]
	)
}


pub async fn select_upf() -> Option<UPFNodeContext> {
	let nodes = crate::n4::PFCP_NODES_GLOBAL_CONTEXT.nodes.read().await;
	for (ip, item) in nodes.iter() {
		return Some(item.clone());
	}
	None
}

pub const MIN_BYTES_BEFORE_THRES: u64 = 200 * 2000;
pub const THRES_RANGE: u64 = 1200 * 2000;
use rand::prelude::*;
pub struct UE {
	pub upf_seid: u64,
	pub vol_thres: u64,
	pub time_thres: u64,
	pub num_bytes_sent: u64,
	pub vol_thres_reach_time_ms: u64,
	pub time_thres_reach_time_ms: u64,
	pub teid: Option<u32>,
	pub ue_ip: std::net::Ipv4Addr,
	pub session_handle: Option<Arc<RwLock<PFCPSessionCP>>>
}

impl UE {
	pub fn new(ue_ip: std::net::Ipv4Addr, rng: &mut StdRng) -> Self {
		let thres = rng.gen_range(0..THRES_RANGE);
		Self {
			upf_seid: 0,
			vol_thres: MIN_BYTES_BEFORE_THRES + thres,
			time_thres: 0,
			num_bytes_sent: 0,
			vol_thres_reach_time_ms: 0,
			time_thres_reach_time_ms: 0,
			teid: None,
			ue_ip: ue_ip,
			session_handle: None
		}
	}
	pub async fn activate(&mut self, an_ip: std::net::Ipv4Addr) {
		let mut rules = libpfcp::PFCPSessionRules::new();
		let mut upf_handle = select_upf().await.unwrap();
		let mut upf = upf_handle.node.write().await;
		let qfi = 1u8;
		let urr_id = generate_urr_tests_vol(&mut rules, None, Some(self.vol_thres)).unwrap();
		let bd_qer = rules.create_qer(
			None,
			{ let mut gs = GateStatus(0); gs.setDLDate(0); gs.setULGate(0); gs },
			None,
			None,
			Some(QFI(qfi & 0b00111111)),
			None,
			None,
			None,
			None
		).unwrap();
		let dl_far = rules.create_far(
			{
				let mut action = ApplyAction(0); // do nothing
				action.setFORW(1);
				action
			},
			Some(
				{
					let mut ohc = OuterHeaderCreation::new();
					ohc.desc.setGTP_U_UDP_IPv4(1);
					ohc.ipv4 = Some(an_ip);
					ohc.teid = Some(1);
					ForwardingParameters {
						destination_interface: libpfcp::models::DestinationInterface::AccessSide,
						network_instnace: None,
						redirect_information: None,
						outer_header_creation: Some(ohc),
						transport_level_marking: None,
						forwarding_policy: None,
						header_enrichment: None,
						linked_traffic_endpoint_id: None,
						pfcpsm_req_flags: None,
						proxying: None,
						destination_interface_type: None,
						data_network_access_identifier: None,
					}
				}
			),
			None,
			None
		).unwrap();
		let dl_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::CoreSide,
			local_f_teid: None,
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![{
				let mut ue_ip = UE_IPAddress::new();
				{
					ue_ip.flags.setV4(1);
					ue_ip.flags.setSD(1);
					ue_ip.ipv4 = Some(self.ue_ip);
				}
				ue_ip
			}],
			traffic_endpoint_id: vec![],
				sdf_filter: None,
				application_id: None,
				ethernet_pdu_session_nformation: None,
				ethernet_pakcet_filter: None,
				qfi: vec![],
				framed_route: vec![],
				framed_routing: None,
				framed_ipv6_route: vec![],
				source_interface_type: None,
				ip_multicast_addressing_info: vec![],
		};
		let dl_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			dl_pdi,
			None,
			Some(dl_far),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let ul_far = rules.create_far(
			{
				let mut action = ApplyAction(0);
				action.setFORW(1); // forwarding
				action
			},
			Some(ForwardingParameters {
				destination_interface: libpfcp::models::DestinationInterface::CoreSide,
				network_instnace: None,
				redirect_information: None,
				outer_header_creation: None, // UE's IP packet goes out without any addtions
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				linked_traffic_endpoint_id: None,
				pfcpsm_req_flags: None,
				proxying: None,
				destination_interface_type: None,
				data_network_access_identifier: None,
			}),
			None,
			None
		).unwrap();
		let ul_pdi = PDI {
			source_interface: libpfcp::models::SourceInterface::AccessSide,
			local_f_teid: Some(F_TEID::new_choose(true, false)),
			network_instnace: None,
			redundant_transmission_detection_parameters: None,
			ue_ip_address: vec![],
			traffic_endpoint_id: vec![],
			sdf_filter: None,
			application_id: None,
			ethernet_pdu_session_nformation: None,
			ethernet_pakcet_filter: None,
			qfi: vec![QFI(qfi)],
			framed_route: vec![],
			framed_routing: None,
			framed_ipv6_route: vec![],
			source_interface_type: None,
			ip_multicast_addressing_info: vec![],
		};
		let ul_pdr = rules.create_pdr(
			Precedence::default_precedence(), // lowest
			ul_pdi,
			Some(OuterHeaderRemoval {
				desc: OuterHeaderRemovalDescription::GTP_U_UDP_IP, // remove GTP-U header
				ext_header_deletion: None,
			}),
			Some(ul_far.clone()),
			vec![urr_id],
			Some(bd_qer),
			vec![],
			None,
			None,
			None,
			None,
			vec![],
			None,
			None,
			None
		).unwrap();
		let (session_handle, created_pdr, _) = upf.create_pfcp_session(Some(rules), Some(PDNType::IPv4)).await.unwrap();
		self.session_handle = Some(session_handle.clone());
		let session = session_handle.read().await;
		let cn_tunnel_endpoint = session.active_rules.pdr_map.get(&ul_pdr.rule_id).unwrap().pdi.local_f_teid.as_ref().unwrap().clone();
		self.teid = cn_tunnel_endpoint.teid;
		self.upf_seid = session.remote_seid.unwrap();
	}
	pub async fn release(&mut self) {
		let tmp = self.session_handle.as_ref().unwrap();
		let mut session = tmp.write().await;
		session.release_with_report().await;
	}
}

mod n4;
mod sender;


use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug)]
pub struct Report {
	pub vol_thres_reach_time_ms: u64,
	pub total_bytes: u64,
}

lazy_static! {
	pub static ref N4_REPORT_TIME: dashmap::DashMap<u64, Report> = dashmap::DashMap::new();
	pub static ref REF_REPORT_TIME: dashmap::DashMap<u64, Report> = dashmap::DashMap::new();
}

#[tokio::main]
async fn main() {
	env_logger::init();

	let matches = App::new("UPF")
		.version("1.0")
		.author("Team VET5G")
		.about("UPF service")
		.arg(Arg::new("ue")
			.long("ue")
			.default_value("10")
			.required(true)
			.help("Num of UE"))
		.arg(Arg::new("name")
			.long("name")
			.default_value("ref")
			.required(true)
			.help("name"))
		.get_matches();

	let num_ue: usize = matches.value_of("ue").unwrap().parse().unwrap();
	let name: String = matches.value_of("name").unwrap().into();
	let mut rng = rand::SeedableRng::seed_from_u64(1234);

	info!("Building PFCP Context");
	let pfcp_handler = N4Handlers {};
	let join_handles = libpfcp::setup_global_pfcp_contexts(pfcp_handler, None, tokio::runtime::Handle::current());

	let upf_ip_cp = std::net::IpAddr::V4("<IP of P4 switch>".parse().unwrap()); // IP of P4 switch 
	let smf_ip_cp = std::net::IpAddr::V4("<IP of current machine>".parse().unwrap()); // IP of current machine

	info!("Connecting to UPF {}",  upf_ip_cp);
	add_upf(upf_ip_cp, smf_ip_cp).await;
	
	let self_ip = std::net::Ipv4Addr::new(10, 201, 0, 1);
	let upf_ip = std::net::Ipv4Addr::new(10, 201, 0, 2);
	let gtp_client = GtpClient::new(std::net::IpAddr::V4(self_ip), std::net::IpAddr::V4(upf_ip));
	let mut ues = Vec::with_capacity(num_ue);
	info!("Creating PFCP sessions");
	for i in 0..num_ue {
		let ue_ip = Ipv4Addr::from((i as u32) | 0xff000000u32);
		let mut ue = UE::new(ue_ip, &mut rng);
		ue.activate(self_ip).await;
		ues.push(ue);
	}
	
	std::thread::sleep(Duration::from_secs(2));
	info!("Sending traffic!");

	loop {
		let mut all_ue_done = true;
		let start = Instant::now();
		for ue in ues.iter_mut() {
			let s: usize = rng.gen_range(200..1200); // random size packet from 200 to 1200 bytes
			if rng.gen_range(0..100) < 15 {
				continue; // skip packet in this round with 15% chance
			}
			if ue.num_bytes_sent < ue.vol_thres {
				ue.num_bytes_sent += (s as u64) - 44; // 44 = underlay IP + underlay UDP + GTP
				if ue.num_bytes_sent < ue.vol_thres {
					all_ue_done = false;
				}
				gtp_client.craft_and_send(&ue, s);
				if ue.num_bytes_sent >= ue.vol_thres {
					// threshold reached, record time of event
					ue.vol_thres_reach_time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
					//info!("UE [SEID={}] done at {} UL={}", ue.upf_seid, ue.vol_thres_reach_time_ms, ue.num_bytes_sent);
					REF_REPORT_TIME.insert(ue.upf_seid, Report { vol_thres_reach_time_ms: ue.vol_thres_reach_time_ms, total_bytes: ue.num_bytes_sent });
				}
			}
		}
		let dur = start.elapsed();
		let remain = std::cmp::max(100 - dur.as_millis() as i64, 0);
		//println!("dur {}", dur.as_millis());
		if all_ue_done {
			break;
		}
		std::thread::sleep(Duration::from_millis(remain as _)); // prevent all traffic being sent at once at the end of the experiment
	}

	std::thread::sleep(Duration::from_secs(30));

	let mut file = File::create(format!("result-{}-{}.csv", name, num_ue)).unwrap();

	for item in REF_REPORT_TIME.iter() {
		if let Some(n4_item) = N4_REPORT_TIME.get(item.key()) {
			file.write(&format!("{},{},{},{},{}\n", *item.key(), item.vol_thres_reach_time_ms, n4_item.vol_thres_reach_time_ms, item.total_bytes, n4_item.total_bytes).as_bytes()).unwrap();
		} else {
			file.write(&format!("{},{},-1,{},-1\n", *item.key(), item.vol_thres_reach_time_ms, item.total_bytes).as_bytes()).unwrap();
		}
	}

	println!("Done");

	// for ue in ues.iter_mut() {
	// 	ue.release().await;
	// }

	std::process::exit(0);
}
