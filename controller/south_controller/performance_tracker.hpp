
#ifndef _PERFORMANCE_TRACKER_H_
#define _PERFORMANCE_TRACKER_H_

#include "common.hpp"

constexpr float PERFORMANCE_TRACKER_EMA = 0.95f;

struct PerformanceObject {
    /// @brief  Unit: milliseconds
    float value;
    PerformanceObject(): value(0) {
        
    }
};

struct PerformanceScopeTracker {
    PerformanceObject *obj;
    std::chrono::high_resolution_clock::time_point t0;

    PerformanceScopeTracker(PerformanceObject *obj): obj(obj), t0(std::chrono::high_resolution_clock::now()) {

    }
    ~PerformanceScopeTracker() noexcept {
        try {
            std::chrono::duration<float, std::milli> diff(std::chrono::high_resolution_clock::now() - t0);
            auto val(diff.count());
            obj->value = obj->value * PERFORMANCE_TRACKER_EMA + val * (1.0f - PERFORMANCE_TRACKER_EMA);
        } catch(...) {

        }
    }
};

class PerformanceCountObject {
public:
    // Constructor initializes all members.
    PerformanceCountObject(double alpha = 0.05) : last_time_used(0), last_num_obj(0), alpha_(alpha), avg_time_per_obj_(0.0), initialized_(false) {}

    // Update the EMA of the time per object.
    void update_measurement(uint64_t time_used, size_t num_obj) {
        if (num_obj == 0)
            return;
        double time_per_obj = static_cast<double>(time_used) / num_obj;
        if (!initialized_) {
            // Initialize the average with the first measurement.
            avg_time_per_obj_ = time_per_obj;
            initialized_ = true;
        } else {
            // Update the EMA.
            avg_time_per_obj_ = alpha_ * time_per_obj + (1 - alpha_) * avg_time_per_obj_;
        }
        if (std::isinf(avg_time_per_obj_) || std::isnan(avg_time_per_obj_)) {
            avg_time_per_obj_ = 0;
        }
    }

    // Estimate the time needed for a given number of objects.
    uint64_t estimate_time_needed(size_t num_obj) {
        return static_cast<uint64_t>(avg_time_per_obj_ * double(num_obj));
    }

    // Estimate the number of objects that can be processed within a time budget.
    size_t estimate_num_obj_can_be_processed(uint64_t time_budget) {
        if (avg_time_per_obj_ == 0) {
            return 0; // Avoid division by zero.
        }
        return static_cast<size_t>(double(time_budget) / avg_time_per_obj_);
    }

    u64 last_time_used;
    usize last_num_obj;

public:
    double alpha_;                 // The smoothing factor for the EMA.
    double avg_time_per_obj_;      // The current EMA of time per object.
    bool initialized_;             // Flag to check if the EMA was initialized.
};

struct PerformanceCountScopeTracker {
    PerformanceCountObject *obj;
    std::chrono::high_resolution_clock::time_point t0;
    std::size_t num_obj;

    PerformanceCountScopeTracker(PerformanceCountObject *obj, std::size_t num_obj): obj(obj), t0(std::chrono::high_resolution_clock::now()), num_obj(num_obj) {

    }
    ~PerformanceCountScopeTracker() noexcept {
        try {
            std::chrono::duration<float, std::milli> diff(std::chrono::high_resolution_clock::now() - t0);
            u64 us_used(static_cast<uint64_t>(diff.count() * 1000.0f));
            obj->update_measurement(us_used, num_obj);
            obj->last_num_obj = num_obj;
            obj->last_time_used = us_used;
        } catch(...) {

        }
    }

    void set_num_obj(std::size_t num_obj) noexcept {
        this->num_obj = num_obj;
    }
};

struct GlobalPerformanceObjects {
    PerformanceCountObject pull_worker_ingress_sync_and_read_perf;
    PerformanceCountObject pull_worker_ingress_ma_id_del_perf;
    PerformanceCountObject pull_worker_ingress_send_to_rust_perf;
    PerformanceCountObject pull_worker_ingress_update_last_pull_perf;
    PerformanceObject      pull_worker_ingress_check_pending_queue_perf;

    PerformanceCountObject pull_worker_egress_sync_and_read_perf;
    PerformanceCountObject pull_worker_egress_ma_id_del_perf;
    PerformanceCountObject pull_worker_egress_send_to_rust_perf;
    PerformanceCountObject pull_worker_egress_update_last_pull_perf;
    PerformanceObject      pull_worker_egress_check_pending_queue_perf;

    PerformanceCountObject update_worker_all_updates_perf;
    PerformanceCountObject update_worker_pdr_updates_perf;
    PerformanceCountObject update_worker_ingress_accounting_updates_perf;
    PerformanceCountObject update_worker_egress_accounting_updates_perf;
    PerformanceCountObject update_worker_send_to_rust_perf;

};

#endif
