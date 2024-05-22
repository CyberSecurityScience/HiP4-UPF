
use std::iter::FromIterator;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use derivative::Derivative;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionTables {
	NoTable,
	UlDecap,
	UlEncap,
	DlEncap
}

impl ActionTables {
	pub fn p4_table_name(&self) -> &'static str {
		match self {
			ActionTables::NoTable => "None",
			ActionTables::UlDecap => "pipe.Ingress.ul_mark_tos_table",
			ActionTables::UlEncap => "pipe.Ingress.ul_to_N3N9_table",
			ActionTables::DlEncap => "pipe.Ingress.dl_to_N3N9_table",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionInstance {
	Nop,
	Decap(bool),
	DecapMarkDSCP(bool, u8),
	EncapDl(bool, std::net::Ipv4Addr, u8),
	EncapUl(bool, std::net::Ipv4Addr, u8),
	Buffer(bool),
	DropDl(bool),
	DropUl(bool),
}

impl ActionInstance {
	pub fn which_table(&self) -> ActionTables {
		match self {
			ActionInstance::Nop => ActionTables::NoTable,
			ActionInstance::Decap(_) => ActionTables::UlDecap,
			ActionInstance::DecapMarkDSCP(_, _) => ActionTables::UlDecap,
			ActionInstance::EncapDl(_, _, _) => ActionTables::DlEncap,
			ActionInstance::EncapUl(_, _, _) => ActionTables::UlEncap,
			ActionInstance::Buffer(_) => ActionTables::DlEncap,
			ActionInstance::DropDl(_) => ActionTables::DlEncap,
			ActionInstance::DropUl(_) => ActionTables::UlDecap,
		}
	}
}

pub struct ActionInstanceCount {
	pub at: ActionInstance,
	pub count: usize,
	pub rounded_count: usize
}

pub fn round_up_pwd2(val: usize) -> usize {
	let mut rounded_count = val;
	rounded_count -= 1;
	rounded_count |= rounded_count >> 1;
	rounded_count |= rounded_count >> 2;
	rounded_count |= rounded_count >> 4;
	rounded_count |= rounded_count >> 8;
	rounded_count |= rounded_count >> 16;
	rounded_count |= rounded_count >> 32;
	rounded_count += 1;
	rounded_count
}

impl ActionInstanceCount {
	pub fn round_up(&mut self) {
		self.rounded_count = round_up_pwd2(self.count);
	}
}

#[test]
pub fn test_round_up_1() {
	let mut atc = ActionInstanceCount {
		at: ActionInstance::Buffer(true),
		count: 9,
		rounded_count: 0,
	};
	atc.round_up();
	assert_eq!(atc.rounded_count, 16);
}


#[test]
pub fn test_round_up_2() {
	let mut atc = ActionInstanceCount {
		at: ActionInstance::Buffer(true),
		count: 8,
		rounded_count: 0,
	};
	atc.round_up();
	assert_eq!(atc.rounded_count, 8);
}

#[derive(Derivative)]
#[derivative(Clone, Eq, PartialEq)]
struct TreeNode {
	pub value: usize,
	pub mask: usize,

	pub assigned_action_tuple: Option<ActionInstance>,
	pub total_capacity: usize,
	pub assigned_capacity: usize,
	pub unassigned_capacity: usize,
	pub used_slots: usize,
	pub free_assigned_slots: usize,
	pub free_id_list: Vec<usize>,

	pub is_leaf: bool,
	pub is_root: bool,

	#[derivative(PartialEq="ignore")]
	#[derivative(Hash="ignore")]
	pub left_child: Option<TreeNodePtr>,
	#[derivative(PartialEq="ignore")]
	#[derivative(Hash="ignore")]
	pub right_child: Option<TreeNodePtr>,
	#[derivative(PartialEq="ignore")]
	#[derivative(Hash="ignore")]
	pub parent: Option<TreeNodePtr>,
}

impl TreeNode {
	pub fn set_mask(&mut self, mask: usize) {
		self.mask = mask;
	}
	pub fn make_empty(&mut self) {
		self.assigned_action_tuple = None;
		self.assigned_capacity = 0;
		self.unassigned_capacity = self.total_capacity;
		self.used_slots = 0;
		self.free_assigned_slots = 0;
		self.free_id_list.clear();
		self.left_child.take().map(|mut x| x.release());
		self.right_child.take().map(|mut x| x.release());
		self.is_leaf = true;
	}
	pub fn assign(&mut self, action_tuple: ActionInstance) {
		self.assigned_action_tuple = Some(action_tuple);
		self.assigned_capacity = self.total_capacity;
		self.unassigned_capacity = 0;
		self.used_slots = 0;
		self.free_assigned_slots = self.total_capacity;
		self.free_id_list.clear();
		self.left_child.take().map(|mut x| x.release());
		self.right_child.take().map(|mut x| x.release());
		self.is_leaf = true;
	}
	pub fn allocate(&mut self) -> Option<(usize, bool, usize)> {
		if let Some(val) = self.free_id_list.pop() {
			return Some((val as _, self.free_assigned_slots == 0 && self.free_id_list.len() == 0, 0));
		}
		if self.free_assigned_slots != 0 {
			self.free_assigned_slots -= 1;
			let mut val = self.used_slots + self.value;
			self.used_slots += 1;
			if val == 0 {
				// prevent getting MAID=0
				assert!(self.free_assigned_slots > 1);
				val += 1;
				self.free_assigned_slots -= 1;
				self.used_slots += 1;
			}
			Some((val, self.free_assigned_slots == 0 && self.free_id_list.len() == 0, 1))
		} else {
			None
		}
	}
	pub fn free(&mut self, maid: usize) -> bool {
		self.free_id_list.push(maid);
		self.free_id_list.len() + self.free_assigned_slots == self.total_capacity
	}
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for TreeNode {
	fn cmp(&self, other: &Self) -> Ordering {
		// Notice that the we flip the ordering on costs.
		// In case of a tie we compare positions - this step is necessary
		// to make implementations of `PartialEq` and `Ord` consistent.
		other.total_capacity.cmp(&self.total_capacity)
	}
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for TreeNode {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone, Copy)]
struct TreeNodePtr {
	pub p: *mut TreeNode
}

impl TreeNodePtr {
	pub fn new(n: TreeNode) -> Self {
		unsafe {
			Self {
				p: Box::into_raw(Box::new(n))
			}
		}
	}
	pub fn release(&mut self) {
		unsafe { drop(Box::from_raw(self.p)) }
	}
	pub fn as_mut(&self) -> &mut TreeNode {
		unsafe { self.p.as_mut().unwrap() }
	}
	pub fn as_ref(&self) -> &TreeNode {
		unsafe { self.p.as_ref().unwrap() }
	}
}

unsafe impl Send for TreeNodePtr {}
unsafe impl Sync for TreeNodePtr {}

use super::action_tables::ActionTableError;

#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ActionTableEntry {
	pub action_tuple: ActionInstance,
	pub value: usize,
	pub mask: usize
}


#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ActionTableOperation {
	Insert(ActionTableEntry),
	Modify(ActionTableEntry),
	Delete(ActionTableEntry)
}

pub struct MaidTree {
	pub maid_bits: u8,
	pub maid_capacity: usize,
	pub root: TreeNodePtr,
	pub action_tuple_free_list: HashMap<ActionInstance, Vec<TreeNodePtr>>,
	pub free_space_list: Vec<TreeNodePtr>,
	in_transaction: bool,
	action_table_ops: Vec<ActionTableOperation>,
	ul_decap_cap: usize,
	ul_decap_used: usize,
	ul_encap_cap: usize,
	ul_encap_used: usize,
	dl_encap_cap: usize,
	dl_encap_used: usize,
}

pub const CAPACITY_HEADROOM: usize = 1;

impl MaidTree {
	pub fn new(mut initial_action_tuples: Vec<ActionInstanceCount>, maid_bits: u8, ul_decap_cap: usize, ul_encap_cap: usize, dl_encap_cap: usize) -> Result<(Self, Vec<ActionTableOperation>), ActionTableError> {
		let maid_capacity = 1usize << (maid_bits as usize);

		// phase 1: break up ranges

		loop {
			for it in initial_action_tuples.iter_mut() {
				it.round_up();
			}

			let total_slots: usize = initial_action_tuples
				.iter()
				.map(|f| f.rounded_count)
				.sum();

			if CAPACITY_HEADROOM * total_slots <= maid_capacity {
				break;
			}

			let mut i: usize = 0;
			let mut max_diff = 0usize;
			for (idx, it) in initial_action_tuples.iter().enumerate() {
				let diff = it.rounded_count - it.count;
				if diff > max_diff {
					max_diff = diff;
					i = idx;
				}
			}

			if max_diff <= 1 {
				return Err(ActionTableError::CannotBreakRangeAnymore);
			}

			let it = initial_action_tuples.get_mut(i).unwrap();
			let rounded_down_size = it.rounded_count >> 1;
			let new_action_tuple = ActionInstanceCount {
				at: it.at,
				count: it.count - rounded_down_size,
				rounded_count: 0
			};
			it.count = rounded_down_size;
			initial_action_tuples.push(new_action_tuple);
		}

		// phase 2: tree building
		for it in initial_action_tuples.iter_mut() {
			it.round_up();
		}

		let mut heap = BinaryHeap::from_iter(
			initial_action_tuples
				.iter()
				.map(|at| {
					TreeNode {
						value: 0,
						mask: 0,
						assigned_action_tuple: Some(at.at),
						total_capacity: at.rounded_count,
						assigned_capacity: at.rounded_count,
						unassigned_capacity: 0,
						used_slots: 0,
						free_assigned_slots: at.rounded_count,
						free_id_list: vec![],
						is_leaf: true,
						is_root: false,
						left_child: None,
						right_child: None,
						parent: None
					}
				})
		);

		let mut action_tuple_free_list: HashMap<ActionInstance, Vec<TreeNodePtr>> = HashMap::new();
		let mut free_space_list: Vec<TreeNodePtr> = Vec::new();

		while heap.len() >= 2 {
			let mut node1 = heap.pop().unwrap();
			let node2 = heap.pop().unwrap();
			if node1.total_capacity == node2.total_capacity {
				let node1_ptr = TreeNodePtr::new(node1.clone());
				let node2_ptr = TreeNodePtr::new(node2.clone());
				if let Some(at) = &node1_ptr.as_ref().assigned_action_tuple {
					action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(node1_ptr.clone());
				}
				if let Some(at) = &node2_ptr.as_ref().assigned_action_tuple {
					action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(node2_ptr.clone());
				}
				let merged_node = TreeNode {
					value: 0,
					mask: 0,
					assigned_action_tuple: None,
					total_capacity: node1.total_capacity + node2.total_capacity,
					assigned_capacity: node1.assigned_capacity + node2.assigned_capacity,
					unassigned_capacity: node1.unassigned_capacity + node2.unassigned_capacity,
					used_slots: node1.used_slots + node2.used_slots,
					free_assigned_slots: node1.free_assigned_slots + node2.free_assigned_slots,
					free_id_list: vec![],
					is_leaf: false,
					is_root: false,
					left_child: Some(node1_ptr),
					right_child: Some(node2_ptr),
					parent: None
				};
				heap.push(merged_node);
			} else if node1.total_capacity < node2.total_capacity {
				let mut diff = node2.total_capacity - node1.total_capacity;
				while diff > 0 {
					let empty_leaf = TreeNode {
						value: 0,
						mask: 0,
						assigned_action_tuple: None,
						total_capacity: node1.total_capacity,
						assigned_capacity: 0,
						unassigned_capacity: node1.total_capacity,
						used_slots: 0,
						free_assigned_slots: 0,
						free_id_list: vec![],
						is_leaf: true,
						is_root: false,
						left_child: None,
						right_child: None,
						parent: None
					};
					let node1_ptr = TreeNodePtr::new(node1.clone());
					if let Some(at) = &node1_ptr.as_ref().assigned_action_tuple {
						action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(node1_ptr.clone());
					}
					let empty_leaf_ptr = TreeNodePtr::new(empty_leaf.clone());
					free_space_list.push(empty_leaf_ptr.clone());
					let merged_node = TreeNode {
						value: 0,
						mask: 0,
						assigned_action_tuple: None,
						total_capacity: node1.total_capacity + empty_leaf.total_capacity,
						assigned_capacity: node1.assigned_capacity + empty_leaf.assigned_capacity,
						unassigned_capacity: node1.unassigned_capacity + empty_leaf.unassigned_capacity,
						used_slots: node1.used_slots + empty_leaf.used_slots,
						free_assigned_slots: node1.free_assigned_slots + empty_leaf.free_assigned_slots,
						free_id_list: vec![],
						is_leaf: false,
						is_root: false,
						left_child: Some(node1_ptr),
						right_child: Some(empty_leaf_ptr),
						parent: None
					};
					diff -= node1.total_capacity;
					node1 = merged_node;
				}
				let node1_ptr = TreeNodePtr::new(node1.clone());
				let node2_ptr = TreeNodePtr::new(node2.clone());
				if let Some(at) = &node1_ptr.as_ref().assigned_action_tuple {
					action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(node1_ptr.clone());
				}
				if let Some(at) = &node2_ptr.as_ref().assigned_action_tuple {
					action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(node2_ptr.clone());
				}
				let merged_node = TreeNode {
					value: 0,
					mask: 0,
					assigned_action_tuple: None,
					total_capacity: node1.total_capacity + node2.total_capacity,
					assigned_capacity: node1.assigned_capacity + node2.assigned_capacity,
					unassigned_capacity: node1.unassigned_capacity + node2.unassigned_capacity,
					used_slots: node1.used_slots + node2.used_slots,
					free_assigned_slots: node1.free_assigned_slots + node2.free_assigned_slots,
					free_id_list: vec![],
					is_leaf: false,
					is_root: false,
					left_child: Some(node1_ptr),
					right_child: Some(node2_ptr),
					parent: None
				};
				heap.push(merged_node);
			}
		}
		let root = if initial_action_tuples.len() == 0 {
			TreeNode {
				value: 0,
				mask: 0,
				assigned_action_tuple: None,
				total_capacity: maid_capacity,
				assigned_capacity: 0,
				unassigned_capacity: maid_capacity,
				used_slots: 0,
				free_assigned_slots: 0,
				free_id_list: vec![],
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child: None,
				parent: None
			}
		} else {
			heap.pop().unwrap()
		};
		let mut root_ptr = TreeNodePtr::new(root);
		while root_ptr.as_mut().total_capacity < maid_capacity {
			let cap = root_ptr.as_mut().total_capacity;
			let empty_leaf = TreeNode {
				value: 0,
				mask: 0,
				assigned_action_tuple: None,
				total_capacity: cap,
				assigned_capacity: 0,
				unassigned_capacity: cap,
				used_slots: 0,
				free_assigned_slots: 0,
				free_id_list: vec![],
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child: None,
				parent: None
			};
			if let Some(at) = &root_ptr.as_mut().assigned_action_tuple {
				action_tuple_free_list.entry(*at).or_insert(Vec::new()).push(root_ptr.clone());
			}
			let empty_leaf_ptr = TreeNodePtr::new(empty_leaf.clone());
			free_space_list.push(empty_leaf_ptr.clone());
			let merged_node = TreeNode {
				value: 0,
				mask: 0,
				assigned_action_tuple: None,
				total_capacity: root_ptr.as_mut().total_capacity + empty_leaf.total_capacity,
				assigned_capacity: root_ptr.as_mut().assigned_capacity + empty_leaf.assigned_capacity,
				unassigned_capacity: root_ptr.as_mut().unassigned_capacity + empty_leaf.unassigned_capacity,
				used_slots: root_ptr.as_mut().used_slots + empty_leaf.used_slots,
				free_assigned_slots: root_ptr.as_mut().free_assigned_slots + empty_leaf.free_assigned_slots,
				free_id_list: vec![],
				is_leaf: false,
				is_root: false,
				left_child: Some(root_ptr.clone()),
				right_child: Some(empty_leaf_ptr),
				parent: None
			};
			root_ptr = TreeNodePtr::new(merged_node); // merged_node is not the new root
		}
		if root_ptr.as_mut().total_capacity == root_ptr.as_mut().unassigned_capacity {
			free_space_list.push(root_ptr.clone());
		}

		// phase 3: finish up tree building and link list building
		let mut table_entries = vec![];
		Self::assign_node_value(&root_ptr, (maid_bits) as usize , 0, 0, &mut table_entries)?;
		root_ptr.as_mut().is_root = true;
		let mut ul_decap_used = 0;
		let mut ul_encap_used = 0;
		let mut dl_encap_used = 0;
		for e in table_entries.iter() {
			let table = match e {
				ActionTableOperation::Insert(x) => x.action_tuple.which_table(),
				ActionTableOperation::Modify(_) => unreachable!(),
				ActionTableOperation::Delete(_) => unreachable!(),
			};
			match table {
				ActionTables::NoTable => {},
				ActionTables::UlDecap => ul_decap_used += 1,
				ActionTables::UlEncap => ul_encap_used += 1,
				ActionTables::DlEncap => dl_encap_used += 1,
			}
		}

		if ul_decap_used >= ul_decap_cap || ul_encap_used >= ul_encap_cap || dl_encap_used >= dl_encap_cap {
			return Err(ActionTableError::InsufficientActionTableCapacity);
		}

		free_space_list.sort_unstable_by_key(|f| f.as_mut().total_capacity); // sort in acscending capacity
		for (_, v) in action_tuple_free_list.iter_mut() {
			v.sort_unstable_by_key(|f| f.as_mut().value); // sort in ascending maid value 
		}

		Ok(
			(
				Self {
					maid_bits,
					maid_capacity,
					root: root_ptr,
					action_tuple_free_list,
					free_space_list,
					in_transaction: false,
					action_table_ops: vec![],
					ul_decap_cap,
					ul_encap_cap,
					dl_encap_cap,
					ul_decap_used,
					ul_encap_used,
					dl_encap_used,
				},
				table_entries
			)
		)
	}

	fn assign_node_value(root: &TreeNodePtr, depth: usize, value: usize, mask: usize, table_ops: &mut Vec<ActionTableOperation>) -> Result<(), ActionTableError> {
		let mut root_ref = root.as_mut();
		root_ref.is_root = false;
		root_ref.is_leaf = root_ref.left_child.is_none() && root_ref.right_child.is_none();
		root_ref.mask = mask;
		root_ref.value = value;
		if root_ref.is_leaf && root_ref.assigned_action_tuple.is_some() {
			table_ops.push(
				ActionTableOperation::Insert(
					ActionTableEntry {
						action_tuple: root_ref.assigned_action_tuple.unwrap(),
						value: value as _,
						mask: mask as _
					}
				)
			);
		}
		if depth == 0 && (root_ref.left_child.is_some() || root_ref.right_child.is_some()) {
			return Err(ActionTableError::TreeTooDeep);
		}
		if let Some(left) = &root_ref.left_child {
			Self::assign_node_value(left, depth - 1, value | (0usize << (depth - 1)), mask | (1usize << (depth - 1)), table_ops)?;
			left.as_mut().parent = Some(root.clone());
		}
		if let Some(right) = &root_ref.right_child {
			Self::assign_node_value(right, depth - 1, value | (1usize << (depth - 1)), mask | (1usize << (depth - 1)), table_ops)?;
			right.as_mut().parent = Some(root.clone());
		}

		Ok(())
	}

	pub fn action_tables_transaction_begin(&mut self) {
		self.in_transaction = true;
		self.action_table_ops.clear();
	}

	pub fn action_tables_transaction_commit(&mut self) -> Result<Vec<ActionTableOperation>, ActionTableError> {
		let mut existing_set: HashSet<ActionTableEntry> = HashSet::new();
		let mut insert_set: HashSet<ActionTableEntry> = HashSet::new();
		let mut delete_set: HashSet<ActionTableEntry> = HashSet::new();
		// println!("==action_tables_transaction_commit==");
		// println!("{:#?}", self.action_table_ops);

		for op in self.action_table_ops.iter() {
			match op {
				ActionTableOperation::Insert(item) => {
					if delete_set.contains(item) {
						delete_set.remove(item);
					}
					if insert_set.contains(item) {
						insert_set.remove(item);
						insert_set.insert(*item);
					}
					insert_set.insert(*item);
				},
				ActionTableOperation::Delete(item) => {
					if !insert_set.contains(item) {
						existing_set.insert(*item);
					}
					insert_set.remove(item);
					if existing_set.contains(item) {
						delete_set.insert(*item);
					}
				},
				ActionTableOperation::Modify(_) => unreachable!(),
			}
		}

		for e in insert_set.iter() {
			match e.action_tuple.which_table() {
				ActionTables::NoTable => {},
				ActionTables::UlDecap => self.ul_decap_used += 1,
				ActionTables::UlEncap => self.ul_encap_used += 1,
				ActionTables::DlEncap => self.dl_encap_used += 1,
			}
		}

		for e in delete_set.iter() {
			match e.action_tuple.which_table() {
				ActionTables::NoTable => {},
				ActionTables::UlDecap => self.ul_decap_used -= 1,
				ActionTables::UlEncap => self.ul_encap_used -= 1,
				ActionTables::DlEncap => self.dl_encap_used -= 1,
			}
		}

		if self.ul_decap_used >= self.ul_decap_cap || self.ul_encap_used >= self.ul_encap_cap || self.dl_encap_used >= self.dl_encap_cap {
			return Err(ActionTableError::InsufficientActionTableCapacity);
		}

		let mut ret = Vec::with_capacity(insert_set.len() + delete_set.len());
		ret.extend(delete_set.into_iter().map(|f| ActionTableOperation::Delete(f)));
		ret.extend(insert_set.into_iter().map(|f| ActionTableOperation::Insert(f)));
		self.in_transaction = false;
		self.action_table_ops.clear();
		Ok(ret)
	}

	pub fn allocate_maid(&mut self, action_tuple: ActionInstance) -> Result<usize, ActionTableError> {
		if let Some(list_of_nodes) = self.action_tuple_free_list.get_mut(&action_tuple) {
			// case #1: an existing leaf node with free space
			if list_of_nodes.len() != 0 {
				let first_node = list_of_nodes.get_mut(0).unwrap();
				let mut cur_node = first_node.clone();
				let (maid, full, parent_diff) = first_node.as_mut().allocate().unwrap();
				if full {
					list_of_nodes.remove(0);
				}
				// propagate allocation change to root
				loop {
					if let Some(parent) = cur_node.as_mut().parent.clone() {
						cur_node = parent.clone();
					} else {
						break;
					}
					let mut cur_node_ref = cur_node.as_mut();
					cur_node_ref.free_assigned_slots -= parent_diff;
					cur_node_ref.used_slots += parent_diff;
				}
				return Ok(maid);
			}
		}
		// find largest unassigned space
		if let Some(free_space_node) = self.free_space_list.pop() {
			// case #2: an empty leaf node will be assigned this action_tuple
			let mut cur_node = free_space_node.clone();
			let id = {
				// allocate
				let mut free_space_node_ref = free_space_node.as_mut();
				free_space_node_ref.assign(action_tuple);
				let (id, full, _) = free_space_node_ref.allocate().unwrap();
				if !full {
					self.action_tuple_free_list.entry(action_tuple).or_insert(vec![]).push(free_space_node.clone());
					self.action_tuple_free_list.get_mut(&action_tuple).unwrap().sort_unstable_by_key(|f| f.as_ref().value);
				}
				id
			};
			// propagate capacity change to root
			{
				let free_space_node_ref = free_space_node.as_ref();
				loop {
					if let Some(parent) = cur_node.as_mut().parent {
						cur_node = parent;
					} else {
						break;
					}
					let mut cur_node_ref = cur_node.as_mut();
					cur_node_ref.assigned_capacity += free_space_node_ref.total_capacity;
					cur_node_ref.unassigned_capacity -= free_space_node_ref.total_capacity;
					cur_node_ref.free_assigned_slots += free_space_node_ref.free_assigned_slots;
					cur_node_ref.used_slots += free_space_node_ref.used_slots;
				}
			}
			let free_space_node_ref = free_space_node.as_mut();
			self.action_table_ops.push(
				ActionTableOperation::Insert(
					ActionTableEntry {
						action_tuple: action_tuple,
						value: free_space_node_ref.value as _,
						mask: free_space_node_ref.mask as _
					}
				)
			);
			return Ok(id);
		} else {
			// case #3: an assigned leaf ndoe will be subdivided
			// find leaf node with largest free slots
			let mut cur_node = self.root.clone();
			let mut is_leaf = cur_node.as_ref().is_leaf;
			while !is_leaf {
				let left_unallocated_slots = cur_node.as_ref().left_child.unwrap().as_ref().free_assigned_slots;
				let right_unallocated_slots = cur_node.as_ref().right_child.unwrap().as_ref().free_assigned_slots;
				if left_unallocated_slots >= right_unallocated_slots {
					cur_node = cur_node.as_ref().left_child.unwrap();
				} else {
					cur_node = cur_node.as_ref().right_child.unwrap();
				}
				is_leaf = cur_node.as_ref().is_leaf;
			}
			let leaf_node = cur_node;
			let leaf_node_ref = leaf_node.as_ref();
			if leaf_node_ref.free_assigned_slots == 0 {
				return Err(ActionTableError::InsufficientMAID);
			}
			let left_action_tuple = leaf_node_ref.assigned_action_tuple.unwrap();
			let total_cap = leaf_node_ref.total_capacity;
			let free_slots = leaf_node_ref.free_assigned_slots;
			let mut right_capacity = total_cap >> 1;
			while right_capacity >= free_slots {
				right_capacity >>= 1;
			}
			self.action_table_ops.push(
				ActionTableOperation::Delete(
					ActionTableEntry {
						action_tuple: left_action_tuple,
						value: leaf_node_ref.value as _,
						mask: leaf_node_ref.mask as _
					}
				)
			);
			// remove leaf_node which will be subdivided from allocatable list
			if let Some(list_of_allocatable_nodes) = self.action_tuple_free_list.get_mut(&left_action_tuple) {
				list_of_allocatable_nodes.retain(|n| n.as_ref().value != leaf_node_ref.value);
			}
			drop(leaf_node_ref);
			// create subdivision
			self.subdivide_leaf(leaf_node, action_tuple, total_cap - right_capacity, right_capacity)?;
			// allocate using case #1
			return self.allocate_maid(action_tuple);
		}
	}

	fn subdivide_leaf(&mut self, leaf: TreeNodePtr, right_action_tuple: ActionInstance, left_capacity: usize, right_capacity: usize) -> Result<(), ActionTableError> {
		let mut leaf_ref = leaf.as_mut();
		leaf_ref.is_leaf = false;
		let left_action_tuple = leaf_ref.assigned_action_tuple.take().unwrap();
		let shift_bits = self.maid_bits as usize - leaf_ref.mask.count_ones() as usize - 1;
		if left_capacity == right_capacity {
			let new_mask = leaf_ref.mask | (1usize << shift_bits);
			let left_has_free_slots = left_capacity > leaf_ref.used_slots;
			let left_node = TreeNode {
				value: leaf_ref.value,
				mask: new_mask,
				assigned_action_tuple: Some(left_action_tuple),
				total_capacity: left_capacity,
				assigned_capacity: left_capacity,
				unassigned_capacity: 0,
				used_slots: std::cmp::min(left_capacity, leaf_ref.used_slots),
				free_assigned_slots: if left_has_free_slots { left_capacity - leaf_ref.used_slots } else { 0 },
				free_id_list: leaf_ref.free_id_list.clone(),
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child: None,
				parent: Some(leaf.clone())
			};
			let left_node_ptr = TreeNodePtr::new(left_node);
			if left_has_free_slots {
				self.action_tuple_free_list.entry(left_action_tuple).or_insert(Vec::new()).push(left_node_ptr.clone());
			}
			let right_value = leaf_ref.value | (1usize << shift_bits);
			let right_node = TreeNode {
				value: right_value,
				mask: new_mask,
				assigned_action_tuple: Some(right_action_tuple),
				total_capacity: right_capacity,
				assigned_capacity: right_capacity,
				unassigned_capacity: 0,
				used_slots: 0,
				free_assigned_slots: right_capacity,
				free_id_list: vec![],
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child:  None,
				parent: Some(leaf.clone())
			};
			let right_node_ptr = TreeNodePtr::new(right_node);
			self.action_tuple_free_list.entry(right_action_tuple).or_insert(Vec::new()).push(right_node_ptr.clone());
			self.action_table_ops.push(
				ActionTableOperation::Insert(
					ActionTableEntry {
						action_tuple: left_action_tuple,
						value: left_node_ptr.as_ref().value as _,
						mask: left_node_ptr.as_ref().mask as _
					}
				)
			);
			self.action_table_ops.push(
				ActionTableOperation::Insert(
					ActionTableEntry {
						action_tuple: right_action_tuple,
						value: right_node_ptr.as_ref().value as _,
						mask: right_node_ptr.as_ref().mask as _
					}
				)
			);
			leaf_ref.left_child = Some(left_node_ptr);
			leaf_ref.right_child = Some(right_node_ptr);
			return Ok(());
		} else if left_capacity > right_capacity {
			let half_capacity = leaf_ref.total_capacity >> 1;
			let new_mask = leaf_ref.mask | (1usize << shift_bits);
			let left_free_ids = leaf_ref.free_id_list.iter().filter(|id| **id < leaf_ref.value + (half_capacity >> 1)).cloned().collect::<Vec<_>>();
			let right_free_ids = leaf_ref.free_id_list.iter().filter(|id| **id >= leaf_ref.value + (half_capacity >> 1)).cloned().collect::<Vec<_>>();
			let left_node = TreeNode {
				value: leaf_ref.value,
				mask: new_mask,
				assigned_action_tuple: Some(left_action_tuple),
				total_capacity: half_capacity,
				assigned_capacity: half_capacity,
				unassigned_capacity: 0,
				used_slots: half_capacity,
				free_assigned_slots: 0,
				free_id_list: left_free_ids,
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child: None,
				parent: Some(leaf.clone())
			};
			let left_node_ptr = TreeNodePtr::new(left_node);
			let right_value = leaf_ref.value | (1usize << shift_bits);
			let right_node = TreeNode {
				value: right_value,
				mask: new_mask,
				assigned_action_tuple: Some(left_action_tuple),
				total_capacity: half_capacity,
				assigned_capacity: half_capacity,
				unassigned_capacity: 0,
				used_slots: leaf_ref.used_slots - half_capacity,
				free_assigned_slots: half_capacity - (leaf_ref.used_slots - half_capacity),
				free_id_list: right_free_ids,
				is_leaf: true,
				is_root: false,
				left_child: None,
				right_child: None,
				parent: Some(leaf.clone())
			};
			let right_node_ptr = TreeNodePtr::new(right_node);
			self.action_table_ops.push(
				ActionTableOperation::Insert(
					ActionTableEntry {
						action_tuple: left_action_tuple,
						value: left_node_ptr.as_ref().value as _,
						mask: left_node_ptr.as_ref().mask as _
					}
				)
			);
			self.subdivide_leaf(right_node_ptr, right_action_tuple, left_capacity - half_capacity, half_capacity - (left_capacity - half_capacity))?;
			leaf_ref.left_child = Some(left_node_ptr);
			leaf_ref.right_child = Some(right_node_ptr);
			return Ok(());
		} else {
			unreachable!()
		}
	}

	fn try_merge_children(&mut self, mut node: TreeNodePtr) {
		let left_child = node.as_ref().left_child.unwrap();
		let right_child = node.as_ref().right_child.unwrap();
		if left_child.as_ref().assigned_action_tuple.is_none() && right_child.as_ref().assigned_action_tuple.is_none() && left_child.as_ref().is_leaf && right_child.as_ref().is_leaf {
			// we can merge

			// remove from free space list
			self.free_space_list.retain(|n| n.as_ref().value != left_child.as_ref().value && n.as_ref().value != right_child.as_ref().value);

			// make current node a bigger empty node
			node.as_mut().make_empty();

			// put node in free list
			self.free_space_list.push(node.clone());
		}

		// recursively merge till root
		if let Some(parent) = node.as_ref().parent {
			self.try_merge_children(parent);
		}
	}

	pub fn free_maid(&mut self, maid: usize) {
		let leaf_node = {
			// find leaf node of this PDR-ID
			let mut cur_node = self.root.clone();
			let mut is_leaf = cur_node.as_ref().is_leaf;
			while !is_leaf {
				let left_child_ptr = cur_node.as_ref().left_child.unwrap();
				let left_child = left_child_ptr.as_ref();
				let right_child_ptr = cur_node.as_ref().right_child.unwrap();
				let right_child = right_child_ptr.as_ref();
				if maid & left_child.mask == left_child.value {
					cur_node = cur_node.as_ref().left_child.unwrap();
				} else if maid & right_child.mask == right_child.value {
					cur_node = cur_node.as_ref().right_child.unwrap();
				} else {
					unreachable!()
				}
				is_leaf = cur_node.as_ref().is_leaf;
			}
			cur_node
		};
		let assigned_tuple;
		let empty = {
			let mut leaf_node_ref = leaf_node.as_mut();
			assigned_tuple = leaf_node_ref.assigned_action_tuple.unwrap();
			self.action_tuple_free_list.entry(assigned_tuple).or_default().push(leaf_node.clone());
			leaf_node_ref.free(maid)
		};
		self.action_tuple_free_list.get_mut(&assigned_tuple).unwrap().sort_unstable_by_key(|f| f.as_ref().value);
		{
			
			if empty {
				// new empty leaf!
				{
					let leaf_node_ref = leaf_node.as_ref();
					self.action_table_ops.push(
						ActionTableOperation::Delete(
							ActionTableEntry {
								action_tuple: leaf_node_ref.assigned_action_tuple.unwrap(),
								value: leaf_node_ref.value as _,
								mask: leaf_node_ref.mask as _
							}
						)
					);
				}

				// propagate assigned capacity changes to root
				{
					let cur_ref = leaf_node.as_ref();
					let free_assigned_slots = cur_ref.free_assigned_slots;
					let total_capacity = cur_ref.total_capacity;
					let mut cur_node = leaf_node.clone();
					while let Some(parent) = cur_node.as_ref().parent {
						let mut par_ref = parent.as_mut();
						par_ref.free_assigned_slots -= free_assigned_slots;
						par_ref.unassigned_capacity += total_capacity;
						cur_node = parent.clone();
					}
				}
				
				// remove leaf_node which will be emptied from allocatable list
				if let Some(list_of_allocatable_nodes) = self.action_tuple_free_list.get_mut(&leaf_node.as_ref().assigned_action_tuple.unwrap()) {
					list_of_allocatable_nodes.retain(|n| n.as_ref().value != leaf_node.as_ref().value);
				}
				leaf_node.as_mut().make_empty();
				self.free_space_list.push(leaf_node.clone());
				if let Some(parent) = leaf_node.as_ref().parent.clone() {
					self.try_merge_children(parent.clone());
				}
				self.free_space_list.sort_unstable_by_key(|f| f.as_ref().total_capacity); // sort in acscending capacity
			}
		}
	}

	fn delete_tree(node: TreeNodePtr) {
		if let Some(left) = node.as_mut().left_child.take() {
			Self::delete_tree(left);
			left.as_mut().make_empty();
		}
		if let Some(right) = node.as_mut().right_child.take() {
			Self::delete_tree(right);
			right.as_mut().make_empty();
		}
	}

	pub fn clear(&mut self) {
		Self::delete_tree(self.root);
		self.free_space_list.clear();
		self.action_tuple_free_list.clear();
	}
}

impl Drop for MaidTree {
	fn drop(&mut self) {
		self.clear();
	}
}

#[test]
pub fn test_1() {
	let (mut tree, acts) = MaidTree::new(vec![], 4, 2048, 2048, 7168).unwrap();
	tree.action_tables_transaction_begin();
	println!("acts: {:?}", acts);
	let act1 = ActionInstance::Buffer(true);
	let act2 = ActionInstance::Buffer(false);
	let act3 = ActionInstance::DropDl(false);

	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 1);
	let id = tree.allocate_maid(act2).unwrap();
	assert_eq!(id, 8);
	let id = tree.allocate_maid(act3).unwrap();
	assert_eq!(id, 12);

	tree.free_maid(0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 0);
	tree.free_maid(1);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 1);
	tree.free_maid(8);
	tree.free_maid(1);
	tree.free_maid(12);
	tree.free_maid(0);

	assert!(tree.root.as_ref().left_child.is_none());
	assert!(tree.root.as_ref().right_child.is_none());
	assert_eq!(tree.root.as_ref().unassigned_capacity, 16);
	assert_eq!(tree.root.as_ref().assigned_capacity, 0);
	assert_eq!(tree.root.as_ref().free_assigned_slots, 0);
	assert_eq!(tree.root.as_ref().used_slots, 0);

	// empty again

	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 1);
	let id = tree.allocate_maid(act2).unwrap();
	assert_eq!(id, 8);
	let id = tree.allocate_maid(act3).unwrap();
	assert_eq!(id, 12);

	tree.free_maid(0);
	tree.free_maid(1);

	let act4 = ActionInstance::DropUl(false);

	let id = tree.allocate_maid(act4).unwrap();
	assert_eq!(id, 0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 4);

	tree.free_maid(4);
	let id = tree.allocate_maid(act4).unwrap();
	assert_eq!(id, 1);
	let id = tree.allocate_maid(act4).unwrap();
	assert_eq!(id, 2);
	let id = tree.allocate_maid(act4).unwrap();
	assert_eq!(id, 3);
	let id = tree.allocate_maid(act4).unwrap();
	assert_eq!(id, 4);
	
	tree.free_maid(0);
	tree.free_maid(1);
	tree.free_maid(2);
	tree.free_maid(3);
	tree.free_maid(4);
	tree.free_maid(12);
	tree.free_maid(8);

	assert!(tree.root.as_ref().left_child.is_none());
	assert!(tree.root.as_ref().right_child.is_none());
	assert_eq!(tree.root.as_ref().unassigned_capacity, 16);
	assert_eq!(tree.root.as_ref().assigned_capacity, 0);
	assert_eq!(tree.root.as_ref().free_assigned_slots, 0);
	assert_eq!(tree.root.as_ref().used_slots, 0);

	// empty again

	assert_eq!(tree.action_tables_transaction_commit().unwrap(), vec![]);

	tree.clear();

	println!("passed!");

}

#[test]
pub fn test_2() {
	let act1 = ActionInstance::Buffer(true);
	let act2 = ActionInstance::Buffer(false);
	//let act3 = ActionInstance::DropDl(false);
	let init_tuples = vec![
		ActionInstanceCount {
			at: act1,
			count: 9,
			rounded_count: 0
		},
		ActionInstanceCount {
			at: act2,
			count: 2,
			rounded_count: 0
		}
	];
	let (mut tree, acts) = MaidTree::new(init_tuples, 4, 2048, 2048, 7168).unwrap();
	tree.action_tables_transaction_begin();
	println!("acts: {:?}", acts);
	let id = tree.allocate_maid(act2).unwrap();
	assert_eq!(id, 2);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 8);
	tree.free_maid(0);
	let id = tree.allocate_maid(act1).unwrap();
	assert_eq!(id, 9);
	println!("{:?}", tree.action_tables_transaction_commit());
	println!("passed!");
}

pub fn main() {

}
