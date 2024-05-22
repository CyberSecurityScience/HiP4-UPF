
#ifndef _BATCH_SCHEDULER_H_
#define _BATCH_SCHEDULER_H_

#include "common.hpp"
#include "pdr_tables.hpp"
#include "performance_tracker.hpp"
#include "interface.hpp"

#include <optional>

struct BatchScheduler {
    std::chrono::high_resolution_clock clock;

    std::vector<CopyableTransaction> cur_batch;


    std::size_t cur_batch_ops_count;
    /// @brief  earliest deadline
    std::uint64_t cur_batch_deadline_min_ts;
    std::uint64_t cur_batch_deadline_max_ts;

    GlobalPerformanceObjects *all_perf_objs;

    BatchScheduler(GlobalPerformanceObjects *all_perf_objs) :
        clock(),
        cur_batch(),
        cur_batch_ops_count(0),
        cur_batch_deadline_min_ts(UINT64_MAX),
        cur_batch_deadline_max_ts(0),
        all_perf_objs(all_perf_objs)
    {

    }

    std::tuple<std::uint64_t, std::optional<std::vector<CopyableTransaction>>> submit_request(
        std::vector<CopyableTransaction>::const_iterator begin,
        std::vector<CopyableTransaction>::const_iterator end,
        std::size_t &num_ops
    ) { // must be contiguous
        auto cur(begin);
        for (; cur != end; ++cur) {
            // add cur into current batch
            cur_batch.push_back(*cur);
            cur_batch_ops_count += cur->ops.size();
            cur_batch_deadline_min_ts = std::min(cur_batch_deadline_min_ts, cur->deadline);
            cur_batch_deadline_max_ts = std::min(cur_batch_deadline_max_ts, cur->deadline);
        }
        std::uint64_t cur_ts = get_ts_us();
        auto ts_complete(cur_ts + time_needed_to_perform_n_updates(cur_batch_ops_count));
        //println(" -- [BatchScheduler::submit_request] ts_complete=", ts_complete, " cur_batch_deadline_min_ts=", cur_batch_deadline_min_ts);
        if (ts_complete >= cur_batch_deadline_min_ts) {
            // can't meet latency requirement anyway, flush
            std::optional<std::vector<CopyableTransaction>> ret_batch = {std::move(cur_batch)};
            cur_batch.reserve(256);
            num_ops = cur_batch_ops_count;
            cur_batch_ops_count = 0;
            cur_batch_deadline_min_ts = UINT64_MAX;
            cur_batch_deadline_max_ts = 0;
            //println("[BatchScheduler::submit_request] yielding ", ret_batch->size(), " transactions with ", num_ops, " ops");
            return {UINT64_MAX, ret_batch};
        } else {
            num_ops = 0;
            return {cur_batch_deadline_min_ts - ts_complete, {}};
        }
    }

    /*std::tuple<std::uint64_t, std::optional<std::vector<PDRUpdateRequest>>> submit_request2(std::vector<PDRUpdateRequest>::const_iterator begin, std::vector<PDRUpdateRequest>::const_iterator end) { // must be contiguous
        // for each entry we see, we calculate if adding it will result in any of pending requests to go over delay budget
        // if so we stop and yield a batch while putting the result of requests to batches after
        std::optional<std::vector<PDRUpdateRequest>> ret_batch;
        std::uint64_t ret_batch_ops(0);
        auto cur(begin);
        for (; cur != end; ++cur) {
            auto n_entry(cur_batch_ops_count + cur->ops.size());
            std::uint64_t cur_ts = get_ts_us();
            auto ts_complete(cur_ts + time_needed_to_perform_n_updates(n_entry));
            if (ts_complete > cur_batch_deadline_min_ts) {
                // can't add anymore
                ret_batch = {std::move(cur_batch)};
                cur_batch.reserve(256);
                ret_batch_ops = cur_batch_ops_count;
                cur_batch_ops_count = 0;
                cur_batch_deadline_min_ts = UINT64_MAX;
                cur_batch_deadline_max_ts = 0;
                break;
            } else {
                // add cur into current batch
                cur_batch.push_back(*cur);
                cur_batch_ops_count += cur->ops.size();
                cur_batch_deadline_min_ts = std::min(cur_batch_deadline_min_ts, cur->deadline);
                cur_batch_deadline_max_ts = std::min(cur_batch_deadline_max_ts, cur->deadline);
            }
        }
        for (; cur != end; ++cur) {
            // add remaining requests into cur_batch (becomes next batch after a flush)
            cur_batch.push_back(*cur);
            cur_batch_ops_count += cur->ops.size();
            cur_batch_deadline_min_ts = std::min(cur_batch_deadline_min_ts, cur->deadline);
            cur_batch_deadline_max_ts = std::min(cur_batch_deadline_max_ts, cur->deadline);
        }

        // calculate max waiting time based how many entries are in queue
        std::uint64_t cur_ts = get_ts_us();
        auto ts_complete_if_flush_now(cur_ts + time_needed_to_perform_n_updates(ret_batch_ops) + time_needed_to_perform_n_updates(cur_batch_ops_count));
        std::uint64_t max_wait_timeout(0);
        if (cur_batch_deadline_min_ts >= ts_complete_if_flush_now) {
            max_wait_timeout = cur_batch_deadline_min_ts - ts_complete_if_flush_now;
        }
        if (cur_batch_ops_count == 0) {
            max_wait_timeout = UINT64_MAX;
        }
        return { max_wait_timeout, ret_batch };

        if (ret_batch.has_value()) {
            std::uint64_t cur_ts = get_ts_us();
            if (cur_batch_ops_count != 0) {
                auto ts_complete_if_flush_now(cur_ts + time_needed_to_perform_n_updates(cur_batch_ops_count));
                std::uint64_t max_wait_timeout(0);
                return { 0, ret_batch };
            } else {
                return { UINT64_MAX, ret_batch };
            }
        }

        std::uint64_t cur_ts = get_ts_us();
        auto ts_complete_if_flush_now(cur_ts + time_needed_to_perform_n_updates(cur_batch_ops_count));
        std::uint64_t max_wait_timeout(0);
        if (cur_batch_deadline_min_ts > ts_complete_if_flush_now) {
            // can wait for incoming requests a little bit longer
            max_wait_timeout = cur_batch_deadline_min_ts - ts_complete_if_flush_now;
            return { max_wait_timeout, {} };
        } else {
            ret_batch = {std::move(cur_batch)};
            cur_batch.reserve(256);
            cur_batch_ops_count = 0;
            cur_batch_deadline_min_ts = UINT64_MAX;
            cur_batch_deadline_max_ts = 0;
            return { UINT64_MAX, ret_batch };
        }
    }*/

    std::vector<CopyableTransaction> get_current_batch(std::size_t &num_ops) {
        std::vector<CopyableTransaction> ret_batch(std::move(cur_batch));
        cur_batch.reserve(256);
        num_ops = cur_batch_ops_count;
        cur_batch_ops_count = 0;
        cur_batch_deadline_min_ts = UINT64_MAX;
        cur_batch_deadline_max_ts = 0;
        return ret_batch;
    }

    std::uint64_t time_needed_to_perform_n_updates(std::size_t n) {
        auto r = all_perf_objs->update_worker_all_updates_perf.estimate_time_needed(n);
        //println(" -- Need ",r," us to update ",n," item.");
        return r;
    }
};

#endif
