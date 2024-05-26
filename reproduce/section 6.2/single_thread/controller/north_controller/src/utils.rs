
use std::error::Error;
use std::fmt;

use linear_map::set::LinearSet;
use log::info;

#[derive(Debug)]
pub struct UpfError {
	details: String
}

impl UpfError {
	pub fn new(msg: &str) -> Box<UpfError> {
		Box::new(UpfError{details: msg.to_string()})
	}
}

impl fmt::Display for UpfError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f,"{}",self.details)
	}
}

impl Error for UpfError {
	fn description(&self) -> &str {
		&self.details
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketInterface {
	AccessSide,
	CoreSide,
	SMF,
	N6_LAN,
	_5G_VN_internal,
	Spare
}

pub const BITRATE_ESTIMATOR_EMA: f64 = 0.5f64;

pub struct BitrateEstimator {
	window_size_ms: u64,
	last_time_ms: u64,
	current_time_window_first_pull_time_ms: u64,
	current_time_window: u64,
	last_rate: f64,
	cum_packet_size_in_current_window: u64,
	rate_ema: f64,
	ema_value: f64
}
impl BitrateEstimator {
	pub fn new(cur_time_ms: u64, window_size_ms: u64, ema_value: f64) -> Self {
		Self {
			window_size_ms,
			last_time_ms: cur_time_ms,
			current_time_window_first_pull_time_ms: cur_time_ms,
			current_time_window: cur_time_ms / window_size_ms,
			last_rate: 0f64,
			cum_packet_size_in_current_window: 0,
			rate_ema: 0f64,
			ema_value
		}
	}
	pub fn bytes_per_ms(&self) -> f64 {
		self.rate_ema
	}
	pub fn pull(&mut self, cur_time_ms: u64, inc_packet_size: u64) -> f64 {
		let current_time_window = cur_time_ms / self.window_size_ms;
		let ratio_into_current_window;
		let current_window_rate;
		if current_time_window > self.current_time_window {
			//info!("current_time_window > self.current_time_window");
			let elapsed_time = cur_time_ms - self.current_time_window_first_pull_time_ms;
			let elapsed_time_since_last_pull = cur_time_ms - self.last_time_ms;
			if elapsed_time == 0 || elapsed_time_since_last_pull == 0 {
				return self.rate_ema * 1000.0f64;
			}
			self.cum_packet_size_in_current_window += inc_packet_size;
			let last_window_rate = self.cum_packet_size_in_current_window as f64 / elapsed_time as f64;
			self.last_rate = last_window_rate;
			self.current_time_window = current_time_window;
			self.cum_packet_size_in_current_window = inc_packet_size;
			self.current_time_window_first_pull_time_ms = self.last_time_ms;
			self.last_time_ms = cur_time_ms;
			current_window_rate = self.cum_packet_size_in_current_window as f64 / elapsed_time_since_last_pull as f64;
			ratio_into_current_window = (cur_time_ms - (self.current_time_window * self.window_size_ms)) as f64 / self.window_size_ms as f64;
		} else {
			//info!("current_time_window <= self.current_time_window");
			let elapsed_time = cur_time_ms - self.current_time_window_first_pull_time_ms;
			if elapsed_time == 0 {
				return self.rate_ema * 1000.0f64;
			}
			self.cum_packet_size_in_current_window += inc_packet_size;
			current_window_rate = self.cum_packet_size_in_current_window as f64 / elapsed_time as f64;
			self.last_time_ms = cur_time_ms;
			ratio_into_current_window = (cur_time_ms - (self.current_time_window * self.window_size_ms)) as f64 / self.window_size_ms as f64;
		}
		// info!("ratio_into_current_window={}",ratio_into_current_window);
		// info!("self.cum_packet_size_in_current_window={}",self.cum_packet_size_in_current_window);
		// info!("self.rate_ema={}",self.rate_ema);
		let rate;
		if ratio_into_current_window > 1.0f64 {
			rate = current_window_rate;
			self.rate_ema = rate;
		} else {
			rate = (1.0 - ratio_into_current_window) * self.last_rate + ratio_into_current_window * current_window_rate;
			self.rate_ema = (1.0 - self.ema_value) * self.rate_ema + self.ema_value * rate;
		}
		if self.rate_ema.is_nan() {
			self.rate_ema = 0f64;
		}
		self.rate_ema *= 1.0f64;
		//info!("rate={}",rate);
		assert!(!self.rate_ema.is_nan());
		self.rate_ema * 1000.0f64
	}
}

#[test]
fn test_estimator() {
	let mut est = BitrateEstimator::new(0, 1000, 0.99f64);
	for i in 0..1000 {
		est.pull((i + 1) * 100, 10000); // 100KBps
	}
	println!("{}", est.bytes_per_ms());
	assert!((est.bytes_per_ms() - 100f64).abs() < 1e-6f64);
}

pub fn linear_set_difference<T: Eq + Clone>(old_set: &LinearSet<T>, new_set: &LinearSet<T>) -> (LinearSet<T>, LinearSet<T>, LinearSet<T>) {
	let removed = old_set.difference(new_set).cloned().collect::<LinearSet<_>>();
	let added = new_set.difference(old_set).cloned().collect::<LinearSet<_>>();
	let unchanged = old_set.intersection(new_set).cloned().collect::<LinearSet<_>>();
	(added, unchanged, removed)
}
