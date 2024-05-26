use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use log::info;

#[derive(Debug)]
pub struct TSCNS {
    param_seq: AtomicU32,
    ns_per_tsc: f64,
    base_tsc: i64,
    base_ns: i64,
    calibrate_interval_ns: i64,
    base_ns_err: i64,
    next_calibrate_tsc: i64,
}

impl TSCNS {
    pub const NS_PER_SEC: i64 = 1_000_000_000;

    pub fn new(init_calibrate_ns: i64, calibrate_interval_ns: i64) -> Self {
        let mut ret = TSCNS {
            param_seq: AtomicU32::new(0),
            ns_per_tsc: 0.0,
            base_tsc: 0,
            base_ns: 0,
            calibrate_interval_ns: 0,
            base_ns_err: 0,
            next_calibrate_tsc: 0,
        };
        info!("Calibrating timer");
        ret.init(init_calibrate_ns, calibrate_interval_ns);
        ret
    }

    pub fn init(&mut self, init_calibrate_ns: i64, calibrate_interval_ns: i64) {
        self.calibrate_interval_ns = calibrate_interval_ns;
        let (mut base_tsc, mut base_ns) = Self::sync_time();
        let expire_ns = base_ns + init_calibrate_ns;
        while Self::rd_sys_ns() < expire_ns {
            thread::yield_now();
        }
        let (delayed_tsc, delayed_ns) = Self::sync_time();
        let init_ns_per_tsc = (delayed_ns - base_ns) as f64 / (delayed_tsc - base_tsc) as f64;
        self.save_param(base_tsc, base_ns, base_ns, init_ns_per_tsc);
    }

    pub fn calibrate(&mut self) {
        if Self::rdtsc() < self.next_calibrate_tsc {
            return;
        }

        let (tsc, ns) = Self::sync_time();
        let calculated_ns = self.tsc2ns(tsc);
        let ns_err = calculated_ns - ns;
        let expected_err_at_next_calibration =
            ns_err + (ns_err - self.base_ns_err) * self.calibrate_interval_ns / (ns - self.base_ns + self.base_ns_err);
        let new_ns_per_tsc = self.ns_per_tsc * (1.0 - expected_err_at_next_calibration as f64 / self.calibrate_interval_ns as f64);

        self.save_param(tsc, calculated_ns, ns, new_ns_per_tsc);
    }

    pub fn rdtsc() -> i64 {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdtsc;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdtsc;

        let r = unsafe { _rdtsc() };
        r as _
    }

    pub fn tsc2ns(&self, tsc: i64) -> i64 {
        loop {
            let before_seq = self.param_seq.load(Ordering::Acquire) & !1;
            let ns = self.base_ns + ((tsc - self.base_tsc) as f64 * self.ns_per_tsc) as i64;
            let after_seq = self.param_seq.load(Ordering::Acquire);
            if before_seq == after_seq {
                return ns;
            }
        }
    }

    pub fn rdns(&self) -> i64 {
        self.tsc2ns(Self::rdtsc())
    }

    pub fn rd_sys_ns() -> i64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64
    }

    pub fn get_tsc_ghz(&self) -> f64 {
        1.0 / self.ns_per_tsc
    }

    pub fn sync_time() -> (i64, i64) {
        const N: usize = 3;
        let mut tsc = [0i64; N + 1];
        let mut ns = [0i64; N + 1];

        tsc[0] = Self::rdtsc();
        for i in 1..=N {
            ns[i] = Self::rd_sys_ns();
            tsc[i] = Self::rdtsc();
        }

        let mut best = 1;
        for i in 2..=N {
            if tsc[i] - tsc[i - 1] < tsc[best] - tsc[best - 1] {
                best = i;
            }
        }

        ((tsc[best] + tsc[best - 1]) / 2, ns[best])
    }

    fn save_param(&mut self, base_tsc: i64, base_ns: i64, sys_ns: i64, new_ns_per_tsc: f64) {
        self.base_ns_err = base_ns - sys_ns;
        self.next_calibrate_tsc = base_tsc + ((self.calibrate_interval_ns - 1000) as f64 / new_ns_per_tsc) as i64;
        let seq = self.param_seq.load(Ordering::Relaxed);
        self.param_seq.store(seq + 1, Ordering::Release);
        self.base_tsc = base_tsc;
        self.base_ns = base_ns;
        self.ns_per_tsc = new_ns_per_tsc;
        self.param_seq.store(seq + 2, Ordering::Release);
    }

    pub fn get_cur_us(&self) -> u64 {
        (self.rdns() as u64) / 1000
    }
}

// The implementation for the rest of the methods will follow the logic of the C++ code,
// adapting to Rust's syntax, types, and idiomatic practices.