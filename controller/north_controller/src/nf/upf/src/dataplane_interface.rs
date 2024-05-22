use log::info;
use once_cell::sync::OnceCell;
use pnet::datalink::{Channel::Ethernet, self, NetworkInterface};
use lazy_static::lazy_static;
use tokio::sync::{Mutex, RwLock};

fn packet_reception_thread(is_loopback: bool, mut if_rx: Box<dyn datalink::DataLinkReceiver + 'static>) {
	let runtime = tokio::runtime::Builder::new_current_thread().enable_time().thread_name("UPF dataplane interface packet_reception_thread").build().unwrap();
	runtime.block_on(async {
		loop {
			match if_rx.next() {
				Ok(pkt) => {
					let be = crate::context::BACKEND.get().unwrap();
					let payload_offset;
					if is_loopback {
						// The pnet code for BPF loopback adds a zero'd out Ethernet header
						payload_offset = 14 + 14;
					} else {
						// Maybe is TUN interface
						payload_offset = 0 + 14;
					};
					if pkt.len() > payload_offset {
						be.on_to_cpu_packet_received(&pkt[payload_offset..]).await;
					}
				}
				_ => (),
			}
		}
	});
}

fn packet_sending_thread(mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>, mut if_tx: Box<dyn datalink::DataLinkSender>) {
	let runtime = tokio::runtime::Builder::new_current_thread().enable_time().thread_name("UPF dataplane interface packet_sending_thread").build().unwrap();
	runtime.block_on(async {
		while let Some(pkt) = rx.recv().await {
			if_tx.send_to(&pkt, None);
		}
	});
}


pub fn create_dataplane_interface(interface_name: &str) {
	if interface_name == "disabled" {
		info!("CPU port disabled");
		return;
	}
	let interfaces = datalink::interfaces();
	let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;
	let interface = interfaces
		.into_iter()
		.filter(interface_names_match)
		.next()
		.unwrap_or_else(|| panic!("No such network interface: {}", interface_name));

	// Create a channel to receive on
	let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
		Ok(Ethernet(tx, rx)) => (tx, rx),
		Ok(_) => panic!("unhandled channel type"),
		Err(e) => panic!("unable to create channel: {}", e),
	};
	let (itx, irx) = tokio::sync::mpsc::channel(256);
	let is_loopback = interface.is_loopback();
	DATAPLANE_INTERFACE_SENDER.set(itx).unwrap();
	let handle_rx = std::thread::spawn(move || {
		packet_sending_thread(irx, tx)
	});
	let handle_tx = std::thread::spawn(move || {
		packet_reception_thread(is_loopback, rx)
	});
}

pub static DATAPLANE_INTERFACE_SENDER: OnceCell<tokio::sync::mpsc::Sender<Vec<u8>>> = OnceCell::new();

pub async fn send_packet(pkt: &[u8]) {
	if let Err(e) = DATAPLANE_INTERFACE_SENDER.get().unwrap().send(pkt.to_vec()).await {
		
	}
}
