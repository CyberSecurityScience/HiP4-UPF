
#ifndef _COMMON_H_
#define _COMMON_H_

#include "fast_io/include/fast_io.h"
#include "fast_io/include/fast_io_device.h"

using namespace fast_io::io;

#include <getopt.h>
#include <unistd.h>

extern "C" {
#include <bf_switchd/bf_switchd.h>
#include <target-utils/clish/thread.h>
#include <dvm/bf_drv_intf.h>
#include <target-sys/bf_sal/bf_sys_intf.h>
}

#include <errno.h>
#include <limits.h>
#include <pthread.h>
#include <stdio.h>
#include <time.h>
#include <bf_rt/bf_rt.hpp>
#include <inttypes.h>
#include <sys/types.h>

#include <string_view>
#include <cstddef>

#include "tscns.h"
#include <cmath>

//#define BFRT_GENERIC_FLAGS

constexpr std::uint32_t NUM_SUPPORTED_UES = 160 * 1024;

#define my_assert(status) bf_sys_assert((status) == BF_SUCCESS)
#define return_if_failed(status) {bf_status_t retcode_(status); if (retcode_ != BF_SUCCESS) return retcode_; }

/* A small enum capturing the various options for interactive CLI sessions used
 * by the program. */
enum cli_modes { no_interactive_cli, interactive_ucli, interactive_bfshell };

/* A couple global variables used to pass state between the init function
 * (parse_opts_and_switchd_init) and the final cleanup function
 * (run_cli_or_cleanup) so the caller does not need to bother with additional in
 * and out arguments to the calls. */
static enum cli_modes requested_cli_mode = no_interactive_cli;
static bf_switchd_context_t *switchd_ctx = NULL;

void parse_options(int argc, char **argv);
void parse_opts_and_switchd_init(int argc, char **argv);
void run_cli_or_cleanup();

class ScopeTimer
{
	std::string_view s;
	std::chrono::high_resolution_clock::time_point t0;
public:
	ScopeTimer(std::string_view s) :s(s), t0(std::chrono::high_resolution_clock::now()) {}
	ScopeTimer(const ScopeTimer&) = delete;
	ScopeTimer& operator=(const ScopeTimer&) = delete;
	~ScopeTimer()
	{
		std::chrono::duration<float> diff(std::chrono::high_resolution_clock::now() - t0);
		println("Time running ",s,": ",diff.count(),"s\n");
	}
};

extern TSCNS g_tscns;
std::uint64_t get_ts_us();

using u8 = std::uint8_t;
using u16 = std::uint16_t;
using u32 = std::uint32_t;
using u64 = std::uint64_t;
using i8 = std::int8_t;
using i16 = std::int16_t;
using i32 = std::int32_t;
using i64 = std::int64_t;
using usize = std::size_t;
using f32 = float;
using f64 = double;

class BitrateEstimator {
private:
    static constexpr double BITRATE_ESTIMATOR_EMA = 0.5;

    uint64_t window_size_us;
    uint64_t last_time_us;
    uint64_t current_time_window_first_pull_time_us;
    uint64_t current_time_window;
    double last_rate;
    uint64_t cum_packet_size_in_current_window;
    double rate_ema;
    double ema_value;

public:
    BitrateEstimator(uint64_t cur_time_us, uint64_t window_size_us, double ema_value)
        : window_size_us(window_size_us),
          last_time_us(cur_time_us),
          current_time_window_first_pull_time_us(cur_time_us),
          current_time_window(cur_time_us / window_size_us),
          last_rate(0.0),
          cum_packet_size_in_current_window(0),
          rate_ema(0.0),
          ema_value(ema_value) {}

    double bytes_per_us() const {
        return rate_ema;
    }

    /// @brief 
    /// @param cur_time_us 
    /// @param inc_packet_size 
    /// @return  bytes per second
    double pull(uint64_t cur_time_us, uint64_t inc_packet_size) {
        uint64_t current_time_window = cur_time_us / this->window_size_us;
        double ratio_into_current_window;
        double current_window_rate;

        if (current_time_window > this->current_time_window) {
            uint64_t elapsed_time = cur_time_us - this->current_time_window_first_pull_time_us;
            uint64_t elapsed_time_since_last_pull = cur_time_us - this->last_time_us;

            if (elapsed_time == 0 || elapsed_time_since_last_pull == 0) {
                return this->rate_ema * 1000000.0;
            }

            this->cum_packet_size_in_current_window += inc_packet_size;
            double last_window_rate = static_cast<double>(this->cum_packet_size_in_current_window) / elapsed_time;
            this->last_rate = last_window_rate;
            this->current_time_window = current_time_window;
            this->cum_packet_size_in_current_window = inc_packet_size;
            this->current_time_window_first_pull_time_us = this->last_time_us;
            this->last_time_us = cur_time_us;
            current_window_rate = static_cast<double>(this->cum_packet_size_in_current_window) / elapsed_time_since_last_pull;
            ratio_into_current_window = static_cast<double>(cur_time_us - (this->current_time_window * this->window_size_us)) / this->window_size_us;
        } else {
            uint64_t elapsed_time = cur_time_us - this->current_time_window_first_pull_time_us;

            if (elapsed_time == 0) {
                return this->rate_ema * 1000000.0;
            }

            this->cum_packet_size_in_current_window += inc_packet_size;
            current_window_rate = static_cast<double>(this->cum_packet_size_in_current_window) / elapsed_time;
            this->last_time_us = cur_time_us;
            ratio_into_current_window = static_cast<double>(cur_time_us - (this->current_time_window * this->window_size_us)) / this->window_size_us;
        }

        double rate;
        if (ratio_into_current_window > 1.0) {
            rate = current_window_rate;
            this->rate_ema = rate;
        } else {
            rate = (1.0 - ratio_into_current_window) * this->last_rate + ratio_into_current_window * current_window_rate;
            this->rate_ema = (1.0 - this->ema_value) * this->rate_ema + this->ema_value * rate;
        }

        if (std::isnan(this->rate_ema)) {
            this->rate_ema = 0.0;
        }

        this->rate_ema *= 1.0;
        return this->rate_ema * 1000000.0;
    }
};

#endif
