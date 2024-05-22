
#ifndef _PULL_WORKER_H_
#define _PULL_WORKER_H_

#include "common.hpp"
#include "accounting_table_group.hpp"
#include "interface.hpp"
#include "performance_tracker.hpp"

void pull_worker(
    TableGroup *group,
    UnixSocketInterface *interface,
    bf_rt_target_t dev_tgt,
    GlobalPerformanceObjects *perf_objs,
    bool is_ingress,
    bool *running,
    u64 log_freq_us
);

#endif
