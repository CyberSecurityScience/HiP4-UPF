use atomic_counter::AtomicCounter;
use lazy_static::lazy_static;
use log::{error, warn};
use once_cell::sync::OnceCell;
use std::{os::unix::net::UnixDatagram, sync::Arc, task::{Waker, Poll}, future::Future};

use super::{requests_generated};

pub struct PDRUpdateRequestFutureSharedState {
	pub response: Option<i32>,
	pub waker: Option<Waker>
}

pub struct PDRUpdateRequestFuture {
	pub shared_state: Arc<std::sync::RwLock<PDRUpdateRequestFutureSharedState>>,
}
impl Future for PDRUpdateRequestFuture {
	type Output = i32;

	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		let mut guard = self.shared_state.write().unwrap();
		if guard.response.is_some() {
			Poll::Ready(guard.response.as_ref().unwrap().clone())
		} else {
			guard.waker = Some(cx.waker().clone());
			Poll::Pending
		}
	}
}

#[derive(Debug)]
struct GlobalStatesIPC {
    pub socket: UnixDatagram
}

struct GlobalStates {
    pub transaction_id_gen: atomic_counter::RelaxedCounter,
    pub ongoing_requests: dashmap::DashMap<u32, Arc<std::sync::RwLock<PDRUpdateRequestFutureSharedState>>>,
    pub max_concurrent_tasks: std::sync::Arc<std::sync::Mutex<i32>>,
    pub cur_concurrent_tasks: std::sync::Arc<std::sync::Mutex<i32>>
}

impl GlobalStates {
    pub fn new() -> Self {
        Self {
            transaction_id_gen: atomic_counter::RelaxedCounter::new(0),
            ongoing_requests: dashmap::DashMap::new(),
            max_concurrent_tasks: std::sync::Arc::new(std::sync::Mutex::new(0)),
            cur_concurrent_tasks: std::sync::Arc::new(std::sync::Mutex::new(0))
        }
    }
}

#[derive(Debug)]
struct GlobalStatesCallbacks {
    pub counter_value_cb: fn (&flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<super::response_generated::upfdriver::response::CounterValue<'_>>>, bool, u64),
    pub ma_id_reclaim_cb: fn (&flatbuffers::Vector<'_, u32>)
}

pub fn set_callbacks(
    counter_value_cb: fn (&flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<super::response_generated::upfdriver::response::CounterValue<'_>>>, bool, u64),
    ma_id_reclaim_cb: fn (&flatbuffers::Vector<'_, u32>)
) {
    let s = GlobalStatesCallbacks {
        counter_value_cb,
        ma_id_reclaim_cb,
    };
    GLOBAL_UPF_DRIVER_CALLBACK_STATES.set(s).unwrap();
}

fn recv_impl(data: &[u8]) {
    let resp = super::response_generated::upfdriver::response::root_as_response(data).unwrap();
    if let Some(cv) = resp.response_as_counter_values() {
        if let Some(v) = cv.values() {
            // for item in v {
            //     println!("[{}]: {} pkts, {} bytes", item.ma_id(), item.pkts(), item.bytes());
            // }
            if let Some(cbs) = GLOBAL_UPF_DRIVER_CALLBACK_STATES.get() {
                (cbs.counter_value_cb)(&v, cv.is_ig(), cv.ts());
            }
        }
    }
    if let Some(resp) = resp.response_as_table_operation_complete() {
        if let Some(ids) = resp.transaction_ids() {
            // {
            //     let mut guard = GLOBAL_UPF_DRIVER_STATES.cur_concurrent_tasks.lock().unwrap();
            //     *guard -= 1;
            // }
            let code = resp.code();
            for tid in ids {
                if let Some((_, shared_state)) = GLOBAL_UPF_DRIVER_STATES.ongoing_requests.remove(&tid) {
                    let mut shared_state = shared_state.write().unwrap();
                    shared_state.response = Some(code);
                    if let Some(waker) = shared_state.waker.take() {
                        //println!("[if] proc {} done", tid);
                        waker.wake();
                    }
                } else {
                    warn!(
                        "No entry found for incoming response transaction_id={}",
                        tid
                    );
                }
            }
        }
    }
    if let Some(reclaim) = resp.response_as_ma_id_reclaimed() {
        if let Some(ids) = reclaim.ma_ids() {
            // for id in ids {
            //     println!("MAID reclaimed {}", id);
            // }
            if let Some(cbs) = GLOBAL_UPF_DRIVER_CALLBACK_STATES.get() {
                (cbs.ma_id_reclaim_cb)(&ids);
            }
        }
    }
}

fn recv_thread(source: String) {
    let socket = UnixDatagram::bind(source).unwrap();
    let mut buffer = vec![0u8; 10 * 1024 * 1024];
    loop {
        let size = match socket.recv(buffer.as_mut_slice()) {
            Ok(s) => s,
            Err(e) => break,
        };
        recv_impl(&buffer.as_slice()[..size]);
    }
}

pub fn init(target: String, source: String) {
    let socket = UnixDatagram::unbound().unwrap();
    socket.connect(target).unwrap();
    let ipc_states = GlobalStatesIPC {
        socket: socket
    };
    GLOBAL_UPF_DRIVER_NETWORK_STATES.set(ipc_states).unwrap();
    std::thread::spawn(move || { recv_thread(source) });
}

//#[derive(Debug)]
pub enum PDRUpdate {
    DL_N6_SimpleInsert(requests_generated::upfdriver::requests::DL_N6_SimpleInsertRequestArgs),
    DL_N6_SimpleUpdate(requests_generated::upfdriver::requests::DL_N6_SimpleUpdateRequestArgs),
    DL_N6_SimpleRemove(requests_generated::upfdriver::requests::DL_N6_SimpleRemoveRequestArgs),
    UL_N6_SimpleInsert(requests_generated::upfdriver::requests::UL_N6_SimpleInsertRequestArgs),
    UL_N6_SimpleUpdate(requests_generated::upfdriver::requests::UL_N6_SimpleUpdateRequestArgs),
    UL_N6_SimpleRemove(requests_generated::upfdriver::requests::UL_N6_SimpleRemoveRequestArgs),
    InsertOrUpdatePerMaidURR(requests_generated::upfdriver::requests::InsertOrUpdatePerMaidURRArgs),
}


fn send_pdr_update_impl(tid: u32, data: &[u8]) -> std::io::Result<PDRUpdateRequestFuture> {
    let network = GLOBAL_UPF_DRIVER_NETWORK_STATES.get().unwrap();
    assert_eq!(GLOBAL_UPF_DRIVER_STATES.ongoing_requests.contains_key(&tid), false);
    let shared_state = Arc::new(std::sync::RwLock::new(PDRUpdateRequestFutureSharedState {
        response: None,
        waker: None
    }));
    let future = PDRUpdateRequestFuture {
        shared_state: shared_state.clone(),
    };
    GLOBAL_UPF_DRIVER_STATES.ongoing_requests.insert(tid, shared_state);
    if let Err(er) = network.socket.send(data) {
        GLOBAL_UPF_DRIVER_STATES.ongoing_requests.remove(&tid);
        return Err(er);
    } else {
        // let mut guard = GLOBAL_UPF_DRIVER_STATES.cur_concurrent_tasks.lock().unwrap();
        // *guard += 1;
        // let mut guard_max = GLOBAL_UPF_DRIVER_STATES.max_concurrent_tasks.lock().unwrap();
        // if *guard > *guard_max {
        //     *guard_max = *guard;
        //     println!("new max concurrency {}", *guard_max);
        // }
    }
    //println!("[if] proc {}", tid);
    Ok(future)
}

pub async fn send_pdr_update(updates: Vec<PDRUpdate>, delay_budget_us: u64) -> Result<(), i32> {
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let mut requests_vec_fb = vec![];
    requests_vec_fb.reserve(updates.len());
    for update in updates {
        let request_args = match update {
            PDRUpdate::DL_N6_SimpleInsert(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::DL_N6_SimpleInsertRequest,
                    request: Some(requests_generated::upfdriver::requests::DL_N6_SimpleInsertRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::DL_N6_SimpleUpdate(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::DL_N6_SimpleUpdateRequest,
                    request: Some(requests_generated::upfdriver::requests::DL_N6_SimpleUpdateRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::DL_N6_SimpleRemove(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::DL_N6_SimpleRemoveRequest,
                    request: Some(requests_generated::upfdriver::requests::DL_N6_SimpleRemoveRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::UL_N6_SimpleInsert(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::UL_N6_SimpleInsertRequest,
                    request: Some(requests_generated::upfdriver::requests::UL_N6_SimpleInsertRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::UL_N6_SimpleUpdate(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::UL_N6_SimpleUpdateRequest,
                    request: Some(requests_generated::upfdriver::requests::UL_N6_SimpleUpdateRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::UL_N6_SimpleRemove(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::UL_N6_SimpleRemoveRequest,
                    request: Some(requests_generated::upfdriver::requests::UL_N6_SimpleRemoveRequest::create(&mut builder, &r).as_union_value()),
                }
            },
            PDRUpdate::InsertOrUpdatePerMaidURR(r) => {
                requests_generated::upfdriver::requests::RequestArgs {
                    request_type: requests_generated::upfdriver::requests::RequestUnion::InsertOrUpdatePerMaidURR,
                    request: Some(requests_generated::upfdriver::requests::InsertOrUpdatePerMaidURR::create(&mut builder, &r).as_union_value()),
                }
            }
        };
        let request_fb = requests_generated::upfdriver::requests::Request::create(&mut builder, &request_args);
        requests_vec_fb.push(request_fb);
    }


    let vec: flatbuffers::WIPOffset<flatbuffers::Vector<'_, _>> = builder.create_vector(&requests_vec_fb);
    let cur_ts = crate::context::GLOBAL_TIMER.get().unwrap().get_cur_us();
    let trans_id = (GLOBAL_UPF_DRIVER_STATES.transaction_id_gen.inc() & 0xFFFFFFFF) as u32;
    let trans_args = requests_generated::upfdriver::requests::TransactionArgs {
        transaction_id: trans_id,
        deadline: cur_ts + delay_budget_us,
        ops: Some(vec)
    };
    let trans = requests_generated::upfdriver::requests::Transaction::create(&mut builder, &trans_args);
    builder.finish(trans, None);
    let send_resp = send_pdr_update_impl(trans_id, builder.finished_data());
    match send_resp {
        Ok(future) => {
            let ret = future.await;
            if ret == 0 {
                Ok(())
            } else {
                Err(ret)
            }
        },
        Err(err) => {
            error!("Error: {:?}", err);
            Err(-1000)
        },
    }
    
}

lazy_static! {
	static ref GLOBAL_UPF_DRIVER_STATES: GlobalStates = GlobalStates::new();
	static ref GLOBAL_UPF_DRIVER_NETWORK_STATES: OnceCell<GlobalStatesIPC> = OnceCell::new();
	static ref GLOBAL_UPF_DRIVER_CALLBACK_STATES: OnceCell<GlobalStatesCallbacks> = OnceCell::new();
}
