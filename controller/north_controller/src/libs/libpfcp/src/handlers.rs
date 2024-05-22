#![allow(irrefutable_let_patterns)]

use async_trait::async_trait;

use super::models::PFCPHeader;
use super::ResponseMatchingTuple;
use std::{
	net::{IpAddr, UdpSocket, SocketAddr},
	thread::{self, JoinHandle},
};

#[async_trait]
pub trait SessionRequestHandlers {
	async fn handle_session_establishment(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
	async fn handle_session_modification(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
	async fn handle_session_deletion(&self, header: PFCPHeader, body: Vec<u8>, src_ip: IpAddr)
		-> Vec<u8>;
	async fn handle_session_report(&self, header: PFCPHeader, body: Vec<u8>, src_ip: IpAddr) -> Vec<u8>;
}

#[async_trait]
pub trait NodeRequestHandlers {
	async fn handle_heartbeat(&self, header: PFCPHeader, body: Vec<u8>, src_ip: IpAddr) -> Vec<u8>;
	async fn handle_pfd_management(&self, header: PFCPHeader, body: Vec<u8>, src_ip: IpAddr) -> Vec<u8>;
	async fn handle_association_setup(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
	async fn handle_association_update(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
	async fn handle_association_release(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
	async fn handle_node_report(&self, header: PFCPHeader, body: Vec<u8>, src_ip: IpAddr) -> Vec<u8>;
	async fn handle_session_set_deletion(
		&self,
		header: PFCPHeader,
		body: Vec<u8>,
		src_ip: IpAddr,
	) -> Vec<u8>;
}

pub fn handler_thread_response<C>(handlers: C, recv_socket: UdpSocket, async_runtime: tokio::runtime::Handle)
where
C: SessionRequestHandlers + NodeRequestHandlers + Send + Sync + Clone +'static,
{
	let mut buf = [0; 65536];
	while let (number_of_bytes, src_addr) = recv_socket
		.recv_from(&mut buf)
		.expect("Didn't receive data")
	{
		//println!("receive {} bytes from {}", number_of_bytes, src_addr);
		let mut content = &buf[..number_of_bytes];
		let mut fo_flag_set = true;
		while content.len() != 0 && fo_flag_set {
			if let Ok((body, next_msg_pointer, header)) = PFCPHeader::decode(content) {
				fo_flag_set = header.flags.getFO() != 0;
				if header.is_request() {
					if let Some(mut seq) = super::PFCP_NODE_GLOBAL_CONTEXT.receive_seq_counters.get_mut(&src_addr.ip()) {
						// if header.seq <= *seq.value_mut() && *seq != 0x00_ff_ff_ff {
						// 	println!("Received message whose seq num[{}] <= current seq num[{}], message is considered dupulicated and is discarded", header.seq, *seq);
						// 	break;
						// } else {
						// 	*seq.value_mut() = header.seq;
						// }
						*seq.value_mut() = header.seq;
					} else {
						super::PFCP_NODE_GLOBAL_CONTEXT.receive_seq_counters.insert(src_addr.ip(), header.seq);
					}

					let handlers_cloned = handlers.clone();
					let socket_cloned = recv_socket.try_clone().unwrap();

					async_runtime.spawn(async move {
						//let t = std::time::Instant::now();
						let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
						req_handle_task(handlers_cloned, header, body, socket, src_addr).await;
						//let delay = t.elapsed();
						//println!("delay {}ms", (delay.as_micros() as f64) / 1000.0f64);
					});
				} else {
					let matching_triplet = ResponseMatchingTuple {
						remote_ip: src_addr.ip(),
						seq: header.seq,
					};
					{
						match super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_triplet) {
							Some((_, shared_state)) => {
								let mut shared_state = shared_state.write().unwrap();
								shared_state.response = Some((header, body));
								if let Some(waker) = shared_state.waker.take() {
									waker.wake();
								}
							}
							None => {
								println!(
									"No entry found for incoming response {:?}",
									matching_triplet
								);
							}
						};
					}
				}
				content = next_msg_pointer;
			} else {
				println!("Failed to decode PFCP header, message discarded");
				break;
			}
		}
	}
}

async fn req_handle_task<C: 'static>(handlers: C, header: PFCPHeader, body: Vec<u8>, resp_socket: UdpSocket, src_addr: SocketAddr)
where
	C: SessionRequestHandlers + NodeRequestHandlers + Send + Sync + Clone,
{
	// handle received request
	let mut response_header = header.clone();
	let mut response_body = match header.msg_type {
		1 => handlers.handle_heartbeat(header, body, src_addr.ip()).await,
		3 => handlers.handle_pfd_management(header, body, src_addr.ip()).await,
		5 => handlers.handle_association_setup(header, body, src_addr.ip()).await,
		7 => handlers.handle_association_update(header, body, src_addr.ip()).await,
		9 => handlers.handle_association_release(header, body, src_addr.ip()).await,
		12 => handlers.handle_node_report(header, body, src_addr.ip()).await,
		14 => handlers.handle_session_set_deletion(header, body, src_addr.ip()).await,
		50 => handlers.handle_session_establishment(header, body, src_addr.ip()).await,
		52 => handlers.handle_session_modification(header, body, src_addr.ip()).await,
		54 => handlers.handle_session_deletion(header, body, src_addr.ip()).await,
		56 => handlers.handle_session_report(header, body, src_addr.ip()).await,
		_ => unreachable!(),
	};
	response_header.msg_type += 1; // request -> response
	let mut length = response_body.len() + 4;
	if response_header.seid.is_some() {
		length += 8;
	}
	assert!(length < 0xffff);
	response_header.length = length as _;
	let mut resp_msg = response_header.encode();
	resp_msg.append(&mut response_body);
	// send response back
	resp_socket.send_to(resp_msg.as_slice(), src_addr).unwrap();
}

pub fn handler_thread_request<C: 'static>(handlers: C, request_recv_port_override: Option<u16>, async_runtime: tokio::runtime::Handle)
where
	C: SessionRequestHandlers + NodeRequestHandlers + Send + Sync + Clone,
{
	let socket = UdpSocket::bind(&format!(
		"0.0.0.0:{}",
		request_recv_port_override.map_or(8805, |f| f)
	))
	.expect("couldn't bind to address");
	let mut buf = [0; 65536];
	while let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data")
	{
		//println!("receive {} bytes from {}", number_of_bytes, src_addr);
		let mut content = &buf[..number_of_bytes];
		let mut fo_flag_set = true;
		while content.len() != 0 && fo_flag_set {
			if let Ok((body, next_msg_pointer, header)) = PFCPHeader::decode(content) {
				fo_flag_set = header.flags.getFO() != 0;
				if header.is_request() {
					if let Some(mut seq) = super::PFCP_NODE_GLOBAL_CONTEXT.receive_seq_counters.get_mut(&src_addr.ip()) {
						// if header.seq <= *seq.value_mut() && *seq != 0x00_ff_ff_ff {
						// 	println!("Received message whose seq num[{}] <= current seq num[{}], message is considered dupulicated and is discarded", header.seq, *seq);
						// 	break;
						// } else {
						// 	*seq.value_mut() = header.seq;
						// }
						*seq.value_mut() = header.seq;
					} else {
						super::PFCP_NODE_GLOBAL_CONTEXT.receive_seq_counters.insert(src_addr.ip(), header.seq);
					}

					let handlers_cloned = handlers.clone();
					let socket_cloned = socket.try_clone().unwrap();

					async_runtime.spawn(async move {
						//let t = std::time::Instant::now();
						let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
						req_handle_task(handlers_cloned, header, body, socket, src_addr).await;
						//let delay = t.elapsed();
						//println!("delay {}ms", (delay.as_micros() as f64) / 1000.0f64);
					});
					
				} else {
					println!("[!] Response received on request handler thread, proceed to process regardless.");
					let matching_triplet = ResponseMatchingTuple {
						remote_ip: src_addr.ip(),
						seq: header.seq,
					};
					{
						match  super::PFCP_NODE_GLOBAL_CONTEXT.ongoing_requests.remove(&matching_triplet) {
							Some((_, shared_state)) => {
								let mut shared_state = shared_state.write().unwrap();
								shared_state.response = Some((header, body));
								if let Some(waker) = shared_state.waker.take() {
									waker.wake();
								}
							}
							None => {
								println!(
									"No entry found for incoming response {:?}",
									matching_triplet
								);
							}
						};
					}
				}
				content = next_msg_pointer;
			} else {
				println!("Failed to decode PFCP header, message discarded");
				break;
			}
		}
	}
}

pub fn create_handler_thread<C: 'static>(
	handlers: C,
	send_request_socket: UdpSocket,
	request_recv_port_override: Option<u16>,
	async_runtime: tokio::runtime::Handle
) -> Vec<JoinHandle<()>>
where
	C: SessionRequestHandlers + NodeRequestHandlers + Send + Sync + Clone,
{
	let h1 = handlers.clone();
	let async_runtime_cloned = async_runtime.clone();
	let handle1 = thread::spawn(move || handler_thread_response(h1, send_request_socket, async_runtime_cloned));
	let handle2 =
		thread::spawn(move || handler_thread_request(handlers, request_recv_port_override, async_runtime));
	vec![handle1, handle2]
}
