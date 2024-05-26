
#ifndef _UPDATE_WORKER_H_
#define _UPDATE_WORKER_H_

#include "common.hpp"
#include "accounting_table_group.hpp"
#include "pdr_tables.hpp"
#include "interface.hpp"
#include "performance_tracker.hpp"

void update_worker(
    UnixSocketInterface *interface,
    bf_rt_target_t dev_tgt,
    PDRTables *pdr_tables,
    AccountingTable *egress_table_group,
    AccountingTable *ingress_table_group,
    GlobalPerformanceObjects *perf_objs,
    bool *running,
    u64 log_freq_us
);

#endif
