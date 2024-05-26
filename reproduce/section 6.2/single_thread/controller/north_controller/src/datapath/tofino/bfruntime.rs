use std::{collections::HashMap, convert::TryInto};

use super::{bfruntime::bfrt::{bf_runtime_client::BfRuntimeClient, Subscribe, stream_message_request, subscribe::Notifications, stream_message_response, GetForwardingPipelineConfigRequest, SetForwardingPipelineConfigRequest, set_forwarding_pipeline_config_request, TargetDevice}, bfruntime_models::BfrtInfoP4};
use log::info;
use tokio::sync::mpsc;
use tonic::transport::Channel;

use self::bfrt::{StreamMessageResponse, StreamMessageRequest, ForwardingPipelineConfig, WriteRequest, write_request, Update, update, Entity, entity, TableEntry, TableKey, TableData, table_entry, KeyField, key_field::{MatchType, self, Ternary}, DataField, data_field, ReadRequest, TableFlags, WriteResponse, ReadResponse, TableOperation};

pub mod bfrt {
	tonic::include_proto!("bfrt_proto"); // The string specified here must match the proto package name
}

#[derive(Debug, Clone)]
pub struct P4TableInfo {
	table_id: HashMap<String, u32>,
	action_id: HashMap<(String, String), u32>,
	key_id: HashMap<(String, String), u32>,
	data_id: HashMap<(String, String), u32>,
}

impl P4TableInfo {
	pub fn from_bfrt_info(info: &BfrtInfoP4) -> Self {
		let mut table_id: HashMap<String, u32> = HashMap::new();
		let mut action_id: HashMap<(String, String), u32> = HashMap::new();
		let mut key_id: HashMap<(String, String), u32> = HashMap::new();
		let mut data_id: HashMap<(String, String), u32> = HashMap::new();
		for table in info.tables.iter() {
			table_id.insert(table.name.clone(), table.id);
			for key in table.key.iter() {
				key_id.insert((table.name.clone(), key.name.clone()), key.id);
			}
			key_id.insert((table.name.clone(), "$MATCH_PRIORITY".into()), 65537);
			if let Some(action_specs) = &table.action_specs {
				for action_spec in action_specs.iter() {
					action_id.insert((table.name.clone(), action_spec.name.clone()), action_spec.id);
					if let Some(data) = &action_spec.data {
						for d in data.iter() {
							if let Some(singleton) = &d.singleton {
								data_id.insert((table.name.clone(), singleton.name.clone()), singleton.id);
							}
						}
					}
				}
			}
			if let Some(data) = &table.data {
				for d in data.iter() {
					if let Some(singleton) = &d.singleton {
						data_id.insert((table.name.clone(), singleton.name.clone()), singleton.id);
					}
				}
			}
		}
		Self {
			table_id,
			action_id,
			key_id,
			data_id
		}
	}
	pub fn get_table_by_name(&self, table_name: &str) -> u32 {
		*self.table_id.get(&table_name.to_owned()).expect(&format!("Table {} not found", table_name))
	}
	pub fn get_action_id_by_name(&self, table_name: &str, action_name: &str) -> u32 {
		*self.action_id.get(&(table_name.to_owned(), action_name.to_owned())).expect(&format!("{}::{} not found", table_name, action_name))
	}
	pub fn get_key_id_by_name(&self, table_name: &str, key_name: &str) -> u32 {
		*self.key_id.get(&(table_name.to_owned(), key_name.to_owned())).expect(&format!("{}::{} not found", table_name, key_name))
	}
	pub fn get_data_id_by_name(&self, table_name: &str, data_name: &str) -> u32 {
		*self.data_id.get(&(table_name.to_owned(), data_name.to_owned())).expect(&format!("{}::{} not found", table_name, data_name))
	}
}

#[derive(Debug)]
pub struct BFRuntime {
	pub client: BfRuntimeClient<Channel>,
	stream_tx: mpsc::Sender<StreamMessageRequest>,
	pipeline_config: ForwardingPipelineConfig,
	device: TargetDevice,
	client_id: u32,
	pub p4name: String,
	bfruntime_info_raw: serde_json::Value,
	non_p4_bfruntime_info_raw: serde_json::Value,
	bfruntime_info: BfrtInfoP4,
	pub table_info: P4TableInfo,
	pub tofino_table_info: P4TableInfo,
}


fn stream_in_thread(resp: tonic::Streaming<StreamMessageResponse>, mut stream_tx: mpsc::Sender<StreamMessageRequest>) {
	let mut resp = resp;
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(async move {
			while let Some(r) = {
				let ret = resp.message().await;
				match ret {
					Ok(r) => r,
					Err(_) => {
						return;
					}
				}
			} {
				if let Some(r) = r.update {
					println!("{:?}", r);
					match r {
						bfrt::stream_message_response::Update::Subscribe(s) => {

						},
						bfrt::stream_message_response::Update::Digest(d) => {
							
						},
						bfrt::stream_message_response::Update::IdleTimeoutNotification(n) => {
							
						},
						bfrt::stream_message_response::Update::PortStatusChangeNotification(n) => {

						},
						bfrt::stream_message_response::Update::SetForwardingPipelineConfigResponse(r) => {
							
						},
					}
				}
			}
	});
}


impl BFRuntime {
	pub async fn new(grpc_addr: String, client_id: u32, device_id: u32) -> Result<BFRuntime, Box<dyn std::error::Error + Sync + Send>> {
		// step 1: setup stream channel
		let mut client = BfRuntimeClient::connect(grpc_addr).await?;
		let (mut tx, mut rx) = mpsc::channel(12);
		let outbound = async_stream::stream! {
			while let Some(r) = rx.recv().await {
				yield r;
			}
		};
		let notification = Notifications {
			enable_learn_notifications: true,
			enable_idletimeout_notifications: true,
			enable_port_status_change_notifications: true,
			enable_entry_active_notifications: true
		};
		let subscribe_request = Subscribe {
			is_master: true,
			device_id,
			notifications: Some(notification),
			status: None,
		};
		let stream_channel_request = StreamMessageRequest {
			client_id,
			update: Some(stream_message_request::Update::Subscribe(subscribe_request)),
		};
		tx.send(stream_channel_request).await?;
		let mut resp_stream = client.stream_channel(tonic::Request::new(outbound)).await?.into_inner();
		let subsribe_resp = resp_stream.message().await?.unwrap().update.unwrap();
		let subscribe_success = match subsribe_resp {
			stream_message_response::Update::Subscribe(s) => {
				let code = s.status.unwrap().code;
				code == 0
			}
			_ => unreachable!()
		};
		assert!(subscribe_success);
		let tx_clone = tx.clone();
		// step 2: get pipeline config
		let get_pipe_config_req = GetForwardingPipelineConfigRequest {
			device_id: device_id,
			client_id: client_id
		};
		let resp = client.get_forwarding_pipeline_config(get_pipe_config_req).await?.into_inner();
		let pipeline_config = resp.config[0].clone();
		let bfruntime_info = std::str::from_utf8(&pipeline_config.bfruntime_info)?;
		std::fs::write("p4.json", bfruntime_info).unwrap();
		let bfruntime_info_raw = serde_json::from_str(bfruntime_info)?;
		let bfruntime_info: BfrtInfoP4 = serde_json::from_str(bfruntime_info)?;
		let non_p4_bfruntime_info = resp.non_p4_config.unwrap().bfruntime_info;
		let non_p4_bfruntime_info = std::str::from_utf8(&non_p4_bfruntime_info)?;
		std::fs::write("non-p4.json", non_p4_bfruntime_info).unwrap();
		let non_p4_bfruntime_info_raw = serde_json::from_str(non_p4_bfruntime_info)?;
		let non_p4_bfruntime_info: BfrtInfoP4 = serde_json::from_str(non_p4_bfruntime_info)?;
		let p4name = pipeline_config.p4_name.clone();
		// step 3: set pipeline config
		let bind_req = SetForwardingPipelineConfigRequest {
			device_id,
			client_id,
			action: set_forwarding_pipeline_config_request::Action::Bind as _,
			dev_init_mode: set_forwarding_pipeline_config_request::DevInitMode::FastReconfig as _,
			base_path: "".to_string(),
			config: vec![pipeline_config.clone()],
		};
		let resp = client.set_forwarding_pipeline_config(bind_req).await?.into_inner();
		let table_info = P4TableInfo::from_bfrt_info(&bfruntime_info);
		let tofino_table_info = P4TableInfo::from_bfrt_info(&non_p4_bfruntime_info);
		let device = TargetDevice {
			device_id: device_id,
			pipe_id: 0xffffu32,
			direction: 0xffu32,
			prsr_id: 0xffu32
		};
		Ok(BFRuntime {
			client,
			stream_tx: tx_clone,
			pipeline_config,
			device,
			client_id,
			p4name,
			bfruntime_info_raw,
			non_p4_bfruntime_info_raw,
			bfruntime_info,
			table_info,
			tofino_table_info,
		})
	}
	pub async fn reset_all_tables(&mut self) {
		let mut updates = vec![];
		for table_name in self.table_info.table_id.keys() {
			let table_entry = TableEntry {
				table_id: self.table_info.get_table_by_name(&table_name),
				data: None,
				is_default_entry: false,
				table_read_flag: None,
				table_mod_inc_flag: None,
				entry_tgt: None,
				table_flags: None,
				value: None,
			};
			let entity = Entity {
				entity: Some(entity::Entity::TableEntry(table_entry))
			};
			let update = Update {
				r#type: update::Type::Delete as _,
				entity: Some(entity)
			};
			updates.push(update);
		}
		self.write_update_no_transaction(updates).await;
	}
	pub async fn write_update(&mut self, updates: Vec<Update>) -> Result<WriteResponse, tonic::Status> {
		let req = WriteRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::RollbackOnError as _,
			p4_name: self.p4name.clone(),
		};
		let resp = self.client.write(req).await?;
		Ok(resp.into_inner())
	}
	pub async fn write_update_reorder(&mut self, updates: Vec<Update>, sqn: u64) -> Result<WriteResponse, tonic::Status> {
		let req = WriteRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::RollbackOnError as _,
			p4_name: self.p4name.clone(),
		};
		let resp = self.client.write(req).await?;
		Ok(resp.into_inner())
	}
	pub async fn write_update_no_transaction(&mut self, updates: Vec<Update>) -> Result<WriteResponse, tonic::Status> {
		let req = WriteRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::ContinueOnError as _,
			p4_name: self.p4name.clone(),
		};
		let resp = self.client.write(req).await?;
		Ok(resp.into_inner())
	}
	pub async fn write_update_no_transaction_with_device(&mut self, updates: Vec<Update>, device: TargetDevice) -> Result<WriteResponse, tonic::Status> {
		let req = WriteRequest {
			target: Some(device),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::ContinueOnError as _,
			p4_name: self.p4name.clone(),
		};
		let resp = self.client.write(req).await?;
		Ok(resp.into_inner())
	}
	pub async fn update_table_entries(&mut self, entries: Vec<TableEntry>) -> Result<WriteResponse, tonic::Status> {
		let updates = entries.into_iter().map(|f| {
			let e = Entity { entity: Some(entity::Entity::TableEntry(f)) };
			Update {
				r#type: update::Type::Modify as _,
				entity: Some(e)
			}
		}).collect::<Vec<_>>();
		let req = WriteRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::ContinueOnError as _,
			p4_name: self.p4name.clone(),
		};
		let resp = self.client.write(req).await?;
		Ok(resp.into_inner())
	}
	pub async fn read_table_entries(&mut self, updates: Vec<TableEntry>) -> Result<Vec<ReadResponse>, tonic::Status> {
		let l = updates.len();
		let enities = updates.into_iter().map(|f| Entity { entity: Some(entity::Entity::TableEntry(f)) }).collect::<Vec<_>>();
		let req = ReadRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			entities: enities,
			p4_name: self.p4name.clone()
		};
		let mut resp = self.client.read(req).await?.into_inner();
		let mut ret = Vec::with_capacity(l);
		while let Some(msg) = resp.message().await? {
			ret.push(msg);
		}
		Ok(ret)
	}
	pub async fn sync_table(&mut self, table_names: Vec<&str>) {
		let updates = table_names.iter().map(|table_name| {
			let table_op = TableOperation {
				table_id: self.table_info.get_table_by_name(table_name),
				table_operations_type: "SyncCounters".into(),
			};
			let entity = Entity {
				entity: Some(entity::Entity::TableOperation(table_op))
			};
			let update = Update {
				r#type: update::Type::Insert as _,
				entity: Some(entity)
			};
			update
		}).collect::<Vec<_>>();
		let req = WriteRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			updates: updates,
			atomicity: write_request::Atomicity::ContinueOnError as _,
			p4_name: self.p4name.clone(),
		};
		self.client.write(req).await.unwrap();
	}
	pub async fn read_table_all_entries(&mut self, table_name: &str) -> Vec<Entity> {
		let flag = TableFlags {
			from_hw: false,
			key_only: false,
			mod_del: false,
			reset_ttl: false,
			reset_stats: false
		};
		let table_entry = TableEntry {
			table_id: self.table_info.get_table_by_name(table_name),
			data: None,
			is_default_entry: false,
			table_read_flag: None,
			table_mod_inc_flag: None,
			entry_tgt: Some(self.device.clone()),
			table_flags: Some(flag),
			value: None
		};
		let entity = Entity {
			entity: Some(entity::Entity::TableEntry(table_entry))
		};
		let req = ReadRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			entities: vec![entity],
			p4_name: self.p4name.clone()
		};
		let mut resp_stream = self.client.read(req).await.unwrap().into_inner();
		let et = resp_stream.message().await
			.unwrap()
			.unwrap()
			.entities;
		et
	}
	pub async fn read_register(&mut self, table_name: &str, reg_name: &str, table_size: usize) -> Vec<u32> {
		let mut values = vec![0u32; table_size];
		let flag = TableFlags {
			from_hw: false,
			key_only: false,
			mod_del: false,
			reset_ttl: false,
			reset_stats: false
		};
		let table_entry = TableEntry {
			table_id: self.table_info.get_table_by_name(table_name),
			data: None,
			is_default_entry: false,
			table_read_flag: None,
			table_mod_inc_flag: None,
			entry_tgt: Some(self.device.clone()),
			table_flags: Some(flag),
			value: None,//Some(table_entry::Value::Key(table_key)),
		};
		let entity = Entity {
			entity: Some(entity::Entity::TableEntry(table_entry))
		};
		let req = ReadRequest {
			target: Some(self.device.clone()),
			client_id: self.client_id,
			entities: vec![entity],
			p4_name: self.p4name.clone()
		};
		let mut resp_stream = self.client.read(req).await.unwrap().into_inner();
		let ctr_id = self.table_info.get_data_id_by_name(table_name, reg_name);
		let et = resp_stream.message().await
			.unwrap()
			.unwrap()
			.entities;
		for resp_enitity in et {
			let r = resp_enitity.entity.unwrap();
			match r {
				entity::Entity::TableEntry(t) => {
					let mut idx = 0;
					if let Some(val) = t.value {
						match val {
							table_entry::Value::Key(k) => {
								match k.fields[0].match_type.as_ref().unwrap() {
									MatchType::Exact(s) => {
										idx = u32::from_be_bytes(s.value.as_slice().try_into().unwrap());
									},
									MatchType::Ternary(_) => todo!(),
									MatchType::Lpm(_) => todo!(),
									MatchType::Range(_) => todo!(),
									MatchType::Optional(_) => todo!(),
								}
							},
							table_entry::Value::HandleId(_) => todo!(),
						}
					}
					for d in t.data.unwrap().fields.iter() {
						if d.field_id == ctr_id {
							let val = d.value.as_ref().unwrap();
							match val {
								data_field::Value::Stream(s) => {
									let val = match s.len() {
										8 => {
											let v = u64::from_be_bytes(s.as_slice().try_into().unwrap());
											v
										}
										4 => {
											let v = u32::from_be_bytes(s.as_slice().try_into().unwrap());
											v as u64
										}
										_ => unreachable!()
									};
									values[idx as usize] = val as _;
								},
								_ => unreachable!()
							}
						}
					}
				},
				_ => unreachable!()
			}
		};
		values
	}
}
