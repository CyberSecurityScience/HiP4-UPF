#![allow(nonstandard_style)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate derivative;
extern crate mem_cmp;
extern crate clap;

extern crate lazy_static;
extern crate packet_builder;

use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}};
use cidr::Cidr;

#[derive(Debug, Clone)]
pub struct NetworkFunctionContext {
	pub nssai: Vec<models::ExtSnssai>,
	pub allowed_nssai: Vec<models::ExtSnssai>,
	pub use_https: bool,
	pub uuid: uuid::Uuid,
	pub host: String,
	pub nrf_uri: String,
	pub name: String,
	pub services: Vec<models::NfService1>,
	pub nf_startup_time: chrono::DateTime<chrono::offset::Utc>,
}

use log::{debug, error, log_enabled, info, Level, warn};
use models::NfProfile1;
use routing::{RoutingEntry, RoutingTable};
use serde::{Serialize, Deserialize};
use clap::{App, Arg};

use context::UPFParameters;
use datapath::{BackendInterface, BackendSettings, BackendReport, tofino::{qos::QosQfiDataYaml}};
use libpfcp::{IDAllocator, messages::ForwardingParameters, models::{ApplyAction, F_TEID, F_TEIDFlags, OuterHeaderCreation, OuterHeaderCreationDescription, OuterHeaderRemoval, OuterHeaderRemovalDescription, PDR_ID, Precedence, QFI, TransportLevelMarking}};
use n4::{N4Handlers, PredefinedRules};
use tokio::{runtime::Handle, sync::mpsc::Sender};

use crate::{datapath::{PacketPipeline, tofino::{TofinoBackend, qos::{QosAndPortData, QosPortData, QosQfiData}}, Report}, ir::create_n4_usage_reporting_threads, dataplane_interface::create_dataplane_interface, tscns::TSCNS};

mod tscns;
mod n4;
mod ir;
mod context;
mod datapath;
mod utils;
mod routing;
mod packet_parser;
mod dataplane_interface;

#[derive(Debug, Serialize, Deserialize)]
struct Config_nf {
	pub host: String,
	pub nrf_uri: String,
	pub name: String,
	pub allowed_nssai: Vec<models::ExtSnssai>,
	pub nssai: Vec<models::ExtSnssai>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config_upf {
	pub enable_univmon: bool,
	pub enable_qos: bool,
	pub enable_deferred_id_del: bool,
	pub routing: Vec<RoutingEntry>,
	pub cpu_pcie_port: u16,
	pub cpu_pcie_ifname: String,
	pub dataplane_ip: Option<std::net::IpAddr>,
	pub ports: Vec<QosPortData>,
	pub qos: Vec<QosQfiDataYaml>
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
	pub nf: Config_nf,
	pub upf: Config_upf,
}

// async fn create_p4bmv2_backend(dataplane_ip: IpAddr, p4switch_addr: String, p4json: &str, p4info: &str) {
// 	info!("Connecting to P4 simple_switch_grpc at {} using p4json {}, p4info {}", p4switch_addr, p4json, p4info);
// 	let settings = BackendSettings {
// 		primary_config_file: p4json.into(),
// 		auxiliary_config_file: Some(p4info.into()),
// 		target_addr: p4switch_addr,
// 		upf_ip: dataplane_ip
// 	};
// 	//let be = DummyBackend::new(settings).await;
// 	let be = P4RuntimeBackend::new(settings).await;
// 	context::BACKEND.write().unwrap().replace(be);
// 	info!("P4 simple_switch_grpc connected");
// }

async fn create_p4tofino_backend(dataplane_ip: IpAddr, switchd_addr: String, volestlog_filename: String, routing: RoutingTable, cfg: &Config_upf, ur_sender: Sender<BackendReport>) {
	info!("Connecting to tofino bfruntime gRPC at {}", switchd_addr);
	let mut qos_and_port_data = QosAndPortData {
		port_data: vec![],
		qfi_data: vec![]
	};
	let mut qid_ctr: u8 = 0;
	for port_item in cfg.ports.iter() {
		//assert_eq!(port_item.ports_in_group.len(), 4);
	}
	for qos_item in cfg.qos.iter() {
		let qid = qid_ctr;
		if qos_item.qfi >= 32 {
			panic!("QFI[{}] out of range [0,31]", qos_item.qfi);
		}
		qid_ctr += 1;
		info!("QID[{}] <==> QFI[{}] Sojourn Target = {}ms, Target Prob = {:.3}%", qid, qos_item.qfi, qos_item.sojourn_target_us, qos_item.target_probability);
		qos_and_port_data.qfi_data.push(
			QosQfiData {
				qfi: qos_item.qfi,
				physical_qid: qid,
				gbr_kbps: 12_000, // 12 Mbps
				sojourn_target_us: qos_item.sojourn_target_us,
				target_probability: qos_item.target_probability,
				importance: qos_item.importance
			}
		);
	}
	qos_and_port_data.port_data = cfg.ports.clone();
	let settings = BackendSettings {
		primary_config_file: "empty".into(),
		auxiliary_config_file: None,
		target_addr: switchd_addr,
		upf_ip: dataplane_ip,
		routing,
		cpu_pcie_port: cfg.cpu_pcie_port,
		ur_tx: ur_sender,
		enable_univmon: cfg.enable_univmon,
		enable_qos: cfg.enable_qos,
		enable_deferred_id_del: cfg.enable_deferred_id_del,
		est_log_filename: volestlog_filename,
		qos_and_port_data
	};
	//let be = DummyBackend::new(settings).await;
	let be = TofinoBackend::new(settings).await;
	if let Err(err) = context::BACKEND.set(be) {
		panic!("Faield to set backend");
	}
	info!("P4 tofino bfruntime switchd connected");
}

fn create_teid_pool() {
	
}

async fn populate_predefined_rules() {
	let mut guard = n4::PREDEFINED_RULES.write().await;
	guard.replace(PredefinedRules {
		PDRs: HashMap::new(),
		FARs: vec![],
		URRs: vec![],
		QERs: vec![],
	});
}

#[derive(Serialize)]
struct HeartbeatStruct {
	pub op: String,
	pub path: String,
	pub value: String
}

async fn create_nf_heartbeat_task(base_url: String, nfid: uuid::Uuid, timer: isize) {
	let client = hyper::client::Client::builder().http2_only(true).build_http();
	let s = HeartbeatStruct {
		op: "replace".into(),
		path: "/nfStatus".into(),
		value: "REGISTERED".into()
	};

	let task = tokio::runtime::Handle::current().spawn(async move {
		let mut interval = tokio::time::interval(std::time::Duration::from_secs(timer as _));

		loop {
			interval.tick().await;

			let mut request = hyper::Request::builder()
				.method(hyper::Method::PATCH)
				.uri(format!("{}/nnrf-nfm/v1/nf-instances/{}", base_url, nfid))
				.body(hyper::Body::empty()).unwrap();

			// Body parameter
			let body = serde_json::to_string(&s).unwrap();

			*request.body_mut() = hyper::Body::from(body);

			let header = "application/json";
			request.headers_mut().insert(hyper::header::CONTENT_TYPE, hyper::header::HeaderValue::from_str(header).unwrap());

			let resp = client.request(request).await;
		}
	});
}

async fn register_nrf(param: &UPFParameters, local_ip: std::net::IpAddr) -> Result<(), Box<dyn std::error::Error>> {
	let mut profile = NfProfile1::new(param.nfctx.uuid, models::NfType::UPF, models::NfStatus::registered());
	profile.fqdn = Some(param.nfctx.host.clone());
	profile.s_nssais = Some(param.nfctx.nssai.clone());
	profile.allowed_nssais = Some(param.nfctx.allowed_nssai.clone());
	profile.nf_instance_name = Some(param.nfctx.name.clone());
	profile.nf_services = Some(param.nfctx.services.clone());
	match local_ip {
		IpAddr::V4(ip) => profile.ipv4_addresses = Some(vec![models::Ipv4Addr(ip.to_string())]),
		IpAddr::V6(ip) => profile.ipv6_addresses = Some(vec![models::Ipv6Addr(ip.to_string())]),
	}
	
	let client = hyper::client::Client::builder().http2_only(true).build_http();
	let mut request = hyper::Request::builder()
				.method(hyper::Method::PUT)
				.uri(format!("{}/nnrf-nfm/v1/nf-instances/{}", param.nfctx.nrf_uri, param.nfctx.uuid))
				.body(hyper::Body::empty()).unwrap();

	// Body parameter
	let body = serde_json::to_string(&profile).unwrap();

	*request.body_mut() = hyper::Body::from(body);

	let header = "application/json";
	request.headers_mut().insert(hyper::header::CONTENT_TYPE, hyper::header::HeaderValue::from_str(header).unwrap());

	let resp = client.request(request).await.unwrap();
	if resp.status().is_success() {
		let nmsl = hyper::body::to_bytes(resp.into_body()).await.unwrap();
		let body = std::str::from_utf8(&nmsl).unwrap();
		let body: models::NfProfile1 = serde_json::from_str(body).unwrap();
		create_nf_heartbeat_task(param.nfctx.nrf_uri.clone(), param.nfctx.uuid, body.heart_beat_timer.unwrap()).await;
		info!("UPF uuid={} registered in NRF", param.nfctx.uuid);
	} else {
		panic!("[*] UPF uuid={} failed to register in NRF", param.nfctx.uuid);
	};

	Ok(())
}

async fn deregister_nrf(param: &UPFParameters) {
	let client = hyper::client::Client::builder().http2_only(true).build_http();
	let mut request = hyper::Request::builder()
				.method(hyper::Method::DELETE)
				.uri(format!("{}/nnrf-nfm/v1/nf-instances/{}", param.nfctx.nrf_uri, param.nfctx.uuid))
				.body(hyper::Body::empty()).unwrap();

	let resp = client.request(request).await.unwrap();
	info!("UPF uuid={} deregistered from NRF", param.nfctx.uuid);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
	env_logger::init();

	let mut signals = signal_hook::iterator::Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM]).unwrap();
	let sigthread = std::thread::spawn(move || {
		for sig in signals.forever() {
			info!("Received signal {:?}, exiting", sig);
			std::process::exit(0);
			return;
		}
	});

    let timer = TSCNS::new(1000000000, 3 * TSCNS::NS_PER_SEC);
    context::GLOBAL_TIMER.set(timer).unwrap();
	
	let matches = App::new("UPF")
		.version("1.0")
		.author("Team VET5G")
		.about("UPF service")
		.arg(Arg::new("config")
			.long("config")
			.default_value("config.yaml")
			.required(true)
			//.about("UPF config file")
		)
		.arg(Arg::new("ip")
			.long("ip")
			.default_value("127.0.0.1")
			.required(false)
			//.about("IP")
		)
		.arg(Arg::new("grpc_ip")
			.long("grpc_ip")
			.default_value("127.0.0.1")
			.required(false)
			//.about("IP")
		)
		.arg(Arg::new("p4json")
			.long("p4json")
			.default_value("p4/upf_v1model.json")
			.required(false)
			//.about("path to p4 json file")
		)
		.arg(Arg::new("p4info")
			.long("p4info")
			.default_value("p4/upf_v1model.p4info.txt")
			.required(false)
			//.about("path to p4 info file")
		)
		.arg(Arg::new("volestlog")
			.long("volestlog")
			.default_value("volestlog.bin")
			.required(false)
			//.about("path to p4 info file")
		)
		.get_matches();

	let selfip: std::net::IpAddr = matches.value_of("ip").unwrap().parse().unwrap();
	let grpc_ip: std::net::IpAddr = matches.value_of("grpc_ip").unwrap().parse().unwrap();
	let volestlog_filename = matches.value_of("volestlog").unwrap().to_string();

	let cfg_file = std::fs::File::open(matches.value_of("config").unwrap()).unwrap();
	let config: Config = serde_yaml::from_reader(cfg_file).unwrap();

	let assigned_ipv4;
	let assigned_ipv6;
	if let Some(dataplane_ip) = config.upf.dataplane_ip {
		info!("UPF is using IP={} for control plane and IP={} for data plane", selfip, dataplane_ip);
		match dataplane_ip {
			IpAddr::V4(ip) => {
				assigned_ipv4 = Some(ip);
				assigned_ipv6 = None;
			},
			IpAddr::V6(ip) => {
				assigned_ipv4 = None;
				assigned_ipv6 = Some(ip);
			},
		}
	} else {
		info!("UPF is using IP={} for both data plane and control plane", selfip);
		match selfip {
			IpAddr::V4(ip) => {
				assigned_ipv4 = Some(ip);
				assigned_ipv6 = None;
			},
			IpAddr::V6(ip) => {
				assigned_ipv4 = None;
				assigned_ipv6 = Some(ip);
			},
		}
	};

	let upf_ctx = UPFParameters {
		nfctx: NetworkFunctionContext {
			nssai: config.nf.nssai,
			allowed_nssai: config.nf.allowed_nssai,
			use_https: false,
			host: config.nf.host,
			name: config.nf.name,
			nrf_uri: config.nf.nrf_uri,
			uuid: uuid::Uuid::new_v4(),
			services: vec![],
			nf_startup_time: chrono::Utc::now(),
		},
		p4json_file: matches.value_of("p4json").unwrap().into(),
		p4info_file: matches.value_of("p4info").unwrap().into(),
		node_ip: selfip,
		assigned_ipv4,
		assigned_ipv6,
	};

	
	let upf_ctx_clone = upf_ctx.clone();
	context::UPF_PARAMETERS.set(upf_ctx).unwrap();
	context::BACKEND_TIME_REFERENCE.set(std::time::Instant::now()).unwrap();
	*crate::context::ASYNC_RUNTIME.lock().unwrap() = Some(Handle::current());

	info!("Populating predefined rules");
	populate_predefined_rules().await;

	info!("Creating P4 backend");
	//create_p4bmv2_backend(selfip, "http://127.0.0.1:8080".into(), &upf_ctx_clone.p4json_file, &upf_ctx_clone.p4info_file).await;
	let (ur_tx, ur_rx) = tokio::sync::mpsc::channel::<BackendReport>(40000);
	create_p4tofino_backend(
		selfip, 
		format!("http://{}:50052", grpc_ip).into(),
		volestlog_filename,
		RoutingTable { routing: config.upf.routing.clone() },
		&config.upf,
		ur_tx
	).await;

	info!("Creating Dataplane interface for CPU PCIe on {}", config.upf.cpu_pcie_ifname);
	create_dataplane_interface(&config.upf.cpu_pcie_ifname);

	info!("Building PFCP Context");
	let pfcp_handler = N4Handlers {};
	let mut threads = libpfcp::setup_global_pfcp_contexts(pfcp_handler, None, Handle::current());
	threads.push(create_n4_usage_reporting_threads(ur_rx));

	info!("Registering self in NRF");
	//register_nrf(&upf_ctx_clone, selfip).await.unwrap();

	info!("UPF is running");
	sigthread.join().unwrap();

	//deregister_nrf(&upf_ctx_clone).await;

	let bfrt = context::BACKEND.get().unwrap();
	bfrt.stop().await;

	info!("UPF finishes running");
	std::process::exit(0);
}
