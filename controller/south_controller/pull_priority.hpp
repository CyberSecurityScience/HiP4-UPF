#pragma once

#ifndef _PULL_PRIORITY_H_
#define _PULL_PRIORITY_H_

#include <optional>
#include <mutex>
#include <cstdint>
#include <vector>
#include <tuple>
#include <deque>

#include <unistd.h>

#include "common.hpp"
#include "unordered_dense.h"

#define EMA_VALUE 0.05
#define COUNTDOWN_US 5000000

struct AggregatedVolumeBasedReportingContext {
    u32 vol_thres_id;
    u64 vol_thres;
    bool vol_enabled;
    u64 last_total_vol;
    u64 vol_arrive_ts_us;
    //                    MAID  bitrate  est           seen_volume
    std::vector<std::tuple<u32, f64, BitrateEstimator, u64>> urr_id2bitrate;

    AggregatedVolumeBasedReportingContext(u32 vol_thres_id, u64 vol_thres):
        vol_thres_id(vol_thres_id),
        vol_thres(vol_thres),
        vol_enabled(true),
        last_total_vol(0),
        vol_arrive_ts_us(UINT64_MAX)
        {
        urr_id2bitrate.clear();
        urr_id2bitrate.reserve(1);
    }

    void set_estimator_ts(u32 maid, u64 ts) {
        for (auto& [m, br, est, v] : urr_id2bitrate) {
            if (m == maid) {
                est.set_ts(ts);
                break;
            }
        }
    }

    void add_maid(u32 maid) {
        bool found(false);
        for (auto& [m, br, est, v] : urr_id2bitrate) {
            if (m == maid) {
                found = true;
                break;
            }
        }
        if (!found) {
            urr_id2bitrate.push_back({maid, 0, BitrateEstimator(0, 1000000, EMA_VALUE), 0});
        }
    }

    void remove_maid(u32 maid) {
        auto it = std::find_if(urr_id2bitrate.begin(), urr_id2bitrate.end(), [maid](auto const& it) { return std::get<0>(it) == maid; });
        if (it != urr_id2bitrate.end()) {
            urr_id2bitrate.erase(it);
        }
    }

    void estimate_time_till_pull_vol_at_pull(u64 ts_us) {
        u64 total_seen_vol(0);
        double total_bitrate(0);
		// sum up bitarte from each mutually exculsive PDRs
        for (auto& [m, rate, est, last_vol] : urr_id2bitrate) {
            total_seen_vol += last_vol;
            total_bitrate += rate;
        }
        last_total_vol = total_seen_vol;
        //println("pull ", total_bitrate * 1000000.0, " bytes/s, vol/thres=",total_seen_vol,"/",vol_thres);
        if (total_seen_vol >= vol_thres && vol_enabled) {
            //println("vol reached, disabled vol est");
            vol_arrive_ts_us = ts_us;
            vol_enabled = false; // once we have reached threshold, we stop, until new threshold is given
        }
        if (total_seen_vol >= vol_thres) { // already reach threshold, pull now!
            vol_arrive_ts_us = ts_us;
            return;
        }
        double remaining_bytes(static_cast<double>(vol_thres - total_seen_vol));
        double speed = total_bitrate;
        double remaining_us = remaining_bytes / speed;
        //println("ETA ", remaining_us / 1000000.0, " s");
        if (std::isnan(remaining_us) || std::isinf(remaining_us)) {
            vol_arrive_ts_us = UINT64_MAX;
        } else {
            vol_arrive_ts_us = ts_us + static_cast<u64>(remaining_us);
        }
    }

    void pull(u32 maid, u64 ts_us, u64 vol) {
        for (auto& [m, rate, est, last_vol] : urr_id2bitrate) {
            if (m == maid) {
                u64 diff(vol - last_vol);
                last_vol = vol;
                est.pull(ts_us, diff);
                rate = est.bytes_per_us();
                break;
            }
        }
        estimate_time_till_pull_vol_at_pull(ts_us);
    }
};

struct InsertOrUpdatePerMaidURR {
    u32 ma_id;
    u64 vol_thres;
    u32 vol_thres_id;
    u64 time_thres; // min of all time based rules, unit: us
    u64 period_val; // GCD of all period rules

    // 8-bit flags
    // bit 0: if set, indicate this is an insert, meaning it will followed by an PDR update. Otherwise it is an update to an existing URR, e.g. set new threshold
    // bit 1: if set, time/period based measurement will start immediately, otherwise it will only start after first packet is detected
    u8 flags;
};

// actually storing list of AggregatedVolumeBasedReportingContext
struct SharedPullPriorityContext {
    ankerl::unordered_dense::map<u32, AggregatedVolumeBasedReportingContext> agg_vol_ctx;
};

struct PerMaidPullPriorityContext {
    u32 maid;
    u32 vol_thres_id;
    u64 last_pulled_ts_us;
    u64 last_estimated_time_till_pull;
    u64 period_us;
    bool period_enabled;
    u64 time_thres_us;
    bool time_enabled;
    u64 time_thres_reach;
    u64 next_period_ts;
    bf_rt_handle_t handle;

    u64 estimate_time_till_pull_vol(u64 ts_us, AggregatedVolumeBasedReportingContext *vol_context) {
        if (!vol_context) {
            return UINT64_MAX;
        }
        if (vol_context->last_total_vol >= vol_context->vol_thres) { // already reach threshold, pull now!
            return 0;
        }
        if (ts_us >= vol_context->vol_arrive_ts_us) { // already passed estimate ETA, pull now!
            return 0;
        }
        return vol_context->vol_arrive_ts_us - ts_us;
    }

    u64 estimate_time_till_pull_time_thres(u64 ts_us) {
        if (time_thres_reach >= ts_us) {
            return time_thres_reach - ts_us;
        }
        return 0;
    }

    u64 estimate_time_till_pull_period(u64 ts_us) {
        if (next_period_ts >= ts_us) {
            return next_period_ts - ts_us;
        }
        return 0;
    }

    u64 estimate_time_till_pull_countdown(u64 ts_us) {
        i64 elapsed(ts_us - last_pulled_ts_us);
        i64 remaining_us(COUNTDOWN_US - elapsed);
        if (remaining_us < 0)
            remaining_us = 0;
        return remaining_us;
    }

    u64 estimate_time_till_pull(u64 ts_us, AggregatedVolumeBasedReportingContext *vol_context) {
        last_estimated_time_till_pull = estimate_time_till_pull_countdown(ts_us);
        if (vol_context && vol_context->vol_enabled) {
            last_estimated_time_till_pull = std::min(last_estimated_time_till_pull, estimate_time_till_pull_vol(ts_us, vol_context));
        }
        if (time_enabled) {
            last_estimated_time_till_pull = std::min(last_estimated_time_till_pull, estimate_time_till_pull_time_thres(ts_us));
        }
        if (period_enabled) {
            last_estimated_time_till_pull = std::min(last_estimated_time_till_pull, estimate_time_till_pull_period(ts_us));
        }
        return last_estimated_time_till_pull;
    }

    void pull(u64 ts_us, u64 vol, AggregatedVolumeBasedReportingContext *vol_context) {
        last_pulled_ts_us = ts_us;
        if (vol_context) {
            //println(" -- MAID:", maid, " , vol:", vol);
            vol_context->pull(maid, ts_us, vol);
        }
        if (last_pulled_ts_us >= time_thres_reach) {
            // time threshold reached, disable time based until new threshold is set
            time_enabled = false;
        }
        if (last_pulled_ts_us >= next_period_ts) {
            next_period_ts += period_us;
        }
    }

    void set_cur_ts(u64 ts) {
        if (time_enabled) {
            time_thres_reach = ts + time_thres_us;
        }
        if (period_enabled) {
            next_period_ts = ts + period_us;
        }
    }

    PerMaidPullPriorityContext(u32 maid, u32 vol_thres_id, u64 vol_thres, u64 period_us, u64 time_thres_us, u64 ts) :
        maid(maid),
        vol_thres_id(vol_thres_id),
        last_pulled_ts_us(ts),
        last_estimated_time_till_pull(0),
        period_us(period_us),
        period_enabled(period_us != UINT64_MAX),
        time_thres_us(time_thres_us),
        time_enabled(time_thres_us != UINT64_MAX),
        time_thres_reach(UINT64_MAX),
        next_period_ts(UINT64_MAX) {
        if (time_enabled) {
            time_thres_reach = ts + time_thres_us;
        }
        if (period_enabled) {
            next_period_ts = ts + period_us;
        }
    }

    void update_urr(InsertOrUpdatePerMaidURR urr) {
        // maybe incorrect
        period_enabled = urr.period_val != UINT64_MAX;
        time_enabled = urr.time_thres != UINT64_MAX;
        if (period_enabled) {
            next_period_ts = last_pulled_ts_us + period_us;
        }
        if (time_enabled) {
            time_thres_reach = last_pulled_ts_us + time_thres_us;
        }
    }
};


// Per table partition
struct PullPriorityContext {
    ankerl::unordered_dense::map<u32, PerMaidPullPriorityContext> priority_ctx;
    SharedPullPriorityContext *shared_states;
    //std::deque<InsertOrUpdatePerMaidURR> pending_urr_updates;
    ankerl::unordered_dense::map<u32, std::tuple<InsertOrUpdatePerMaidURR, u64, bf_rt_handle_t>> pending_urr_updates;
    std::deque<u32> pending_urr_removals;
    std::mutex *pending_queue_mutex;

    PullPriorityContext(SharedPullPriorityContext *shared_states): shared_states(shared_states) {
        pending_queue_mutex = new std::mutex();
    }

    void add_or_update_per_maid_urr_impl(InsertOrUpdatePerMaidURR update) {
        // actual add/update
        PerMaidPullPriorityContext ctx(update.ma_id, update.vol_thres_id, update.vol_thres, update.period_val, update.time_thres, 0);
        if (update.vol_thres_id != 0) {
            // thres enabled
            if (shared_states->agg_vol_ctx.count(update.vol_thres_id)) {
                auto& ref = shared_states->agg_vol_ctx.at(update.vol_thres_id);
                // update vol based reporting
                ref.vol_enabled = true;
                ref.vol_thres = update.vol_thres;
            } else {
                AggregatedVolumeBasedReportingContext vol_ctx(update.vol_thres_id, update.vol_thres);
                shared_states->agg_vol_ctx.insert({update.vol_thres_id, vol_ctx});//AggregatedVolumeBasedReportingContext(update.vol_thres_id, update.vol_thres);
                //println("AGGVOLCTX for ", update.ma_id, " and ", update.vol_thres_id, " added");
                auto& ref = shared_states->agg_vol_ctx.at(update.vol_thres_id);
                ref.add_maid(update.ma_id);
                //println("AGGVOLCTX for ", update.ma_id, " and ", update.vol_thres_id, " added MAID ", update.ma_id);
            }
        }
        if (priority_ctx.count(update.ma_id) == 0) {
            //perrln("URR for ", update.ma_id, " added");
            priority_ctx.insert({update.ma_id, ctx});
        } else {
            //perrln("URR for ", update.ma_id, " updating");
            priority_ctx.at(update.ma_id).update_urr(update);
        }
    }

    // update_now is used when this partition is in WRITE state, otherwise it is put into pending URR update queue
    void add_or_update_per_maid_urr(InsertOrUpdatePerMaidURR update, bool update_now) {
        std::lock_guard<std::mutex> lock(*pending_queue_mutex);
        if (pending_urr_updates.contains(update.ma_id)) {
            std::get<0>(pending_urr_updates.at(update.ma_id)) = update;
        } else {
            pending_urr_updates.insert({update.ma_id, {update, 0, 0}});
        }
        auto ma_id(update.ma_id);
        auto new_end = std::remove_if(
            pending_urr_removals.begin(),
            pending_urr_removals.end(),
            [ma_id](auto const& item) {
                return item == ma_id;
            }
        );
        pending_urr_removals.erase(new_end, pending_urr_removals.end());

    }

    void flush_urr_updates() {
        std::lock_guard<std::mutex> lock(*pending_queue_mutex);
        for (auto [maid, value] : pending_urr_updates) {
            auto const& [update, ts, handle] = value;
            if (handle == 0) {
                continue;
            }
            add_or_update_per_maid_urr_impl(update);
            //perrln("URR for ", maid, " associated");
            // TODO: move to constructor
            auto& ctx = priority_ctx.at(maid);
            ctx.set_cur_ts(ts);
            ctx.handle = handle;
            if (ctx.vol_thres_id != 0 && shared_states->agg_vol_ctx.count(ctx.vol_thres_id) != 0) {
                auto& vol_ctx = shared_states->agg_vol_ctx.at(ctx.vol_thres_id);
                vol_ctx.set_estimator_ts(maid, ts);
            }
        }
        for (auto maid : pending_urr_removals) {
            remove_maid_impl(maid);
        }
        pending_urr_updates.clear();
        pending_urr_removals.clear();
    }

    void remove_maid(u32 ma_id, bool update_now) {
        std::lock_guard<std::mutex> lock(*pending_queue_mutex);
        pending_urr_updates.erase(ma_id);
        pending_urr_removals.push_back(ma_id);
    }

    void remove_maid_impl(u32 ma_id) {
        // step 2: remove URR
        if (priority_ctx.contains(ma_id)) {
            //perrln("priority_ctx contains MAID ", ma_id);
            auto& old_ctx = priority_ctx.at(ma_id);
            if (old_ctx.vol_thres_id != 0) {
                //perrln("AGGVOLCTX for ", ma_id, " and ", old_ctx.vol_thres_id, " removed MAID ", ma_id);
                if (shared_states->agg_vol_ctx.contains(old_ctx.vol_thres_id)) {
                    auto& agg_vol_ctx = shared_states->agg_vol_ctx.at(old_ctx.vol_thres_id);
                    agg_vol_ctx.remove_maid(ma_id);
                    if (agg_vol_ctx.urr_id2bitrate.size() == 0) {
                        // remove this agg_vol_ctx
                        shared_states->agg_vol_ctx.erase(old_ctx.vol_thres_id);
                        //perrln("AGGVOLCTX for ", ma_id, " and ", old_ctx.vol_thres_id, " removed");
                    }
                }
            }
            priority_ctx.erase(ma_id);
            //perrln("URR for ", ma_id, " removed");
        }
    }

    void associate_maid_with_handle(u32 maid, bf_rt_handle_t handle, u64 ts) {
        std::lock_guard<std::mutex> lock(*pending_queue_mutex);
        if (pending_urr_updates.contains(maid)) {
            auto& item = pending_urr_updates.at(maid);
            std::get<1>(item) = ts;
            std::get<2>(item) = handle;
        } else {
            perrln("warn!!");
        }
    }

    ankerl::unordered_dense::map<u32, bf_rt_handle_t> generate_pulling_batch(u64 ts, usize batch_size) {
        ankerl::unordered_dense::map<u32, bf_rt_handle_t> ret;
        ret.reserve(batch_size);

        // step 1: update time til pull
        std::vector<std::tuple<u32, bf_rt_handle_t, PerMaidPullPriorityContext*>> to_sort_vec;
        to_sort_vec.reserve(priority_ctx.size());
        for (auto& [maid, ctx] : priority_ctx) {
            AggregatedVolumeBasedReportingContext *volctx(nullptr);
            if (ctx.vol_thres_id != 0 && shared_states->agg_vol_ctx.count(ctx.vol_thres_id) != 0) {
                auto& vol_ctx = shared_states->agg_vol_ctx.at(ctx.vol_thres_id);
                volctx = std::addressof(vol_ctx);
            }
            ctx.estimate_time_till_pull(ts, volctx);
            to_sort_vec.emplace_back(maid, ctx.handle, std::addressof(ctx));
        }
        // step 2: sort
        std::sort(to_sort_vec.begin(), to_sort_vec.end(), [&](auto const& a, auto const& b) {
            return std::get<2>(a)->last_estimated_time_till_pull < std::get<2>(b)->last_estimated_time_till_pull; 
        });
        // step 3: re-generate batch
        usize s(std::min(batch_size, to_sort_vec.size()));
        for (usize i(0); i != s; ++i) {
            auto& [maid, handle, ctx_ptr] = to_sort_vec[i];
            ret[maid] = handle;
        }

        return ret;
    }

    void update_maid_vol(u32 maid, u64 vol, u64 ts) {
        AggregatedVolumeBasedReportingContext *volctx(nullptr);
        //perrln("acquiring context for ", maid);
        auto& ctx = priority_ctx.at(maid);
        //perrln("acquiring AGGVOLEST ", maid, " for ", ctx.vol_thres_id);
        if (ctx.vol_thres_id != 0 && shared_states->agg_vol_ctx.count(ctx.vol_thres_id) != 0) {
            auto& vol_ctx = shared_states->agg_vol_ctx.at(ctx.vol_thres_id);
            volctx = std::addressof(vol_ctx);
        }
        ctx.pull(ts, vol, volctx);
    }

};


#endif
