
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
pub struct BfrtTableKey {
	pub name: String,
	pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct BfrtTableActionSpec {
	pub name: String,
	pub id: u32,
	pub data: Option<Vec<BfrtTableData>>
}

#[derive(Debug, Deserialize)]
pub struct BfrtTableSingleton {
	pub name: String,
	pub id: u32,
}

#[derive(Debug, Deserialize)]
pub struct BfrtTableData {
	pub mandatory: bool,
	//pub read_only: bool,
	pub singleton: Option<BfrtTableSingleton>
}

#[derive(Debug, Deserialize)]
pub struct BfrtTable {
	pub name: String,
	pub id: u32,
	pub table_type: String,
	pub size: u32,
	pub key: Vec<BfrtTableKey>,
	pub action_specs: Option<Vec<BfrtTableActionSpec>>,
	pub data: Option<Vec<BfrtTableData>>
}

#[derive(Debug, Deserialize)]
pub struct BfrtInfoP4 {
	pub schema_version: Option<String>,
	pub tables: Vec<BfrtTable>
}
