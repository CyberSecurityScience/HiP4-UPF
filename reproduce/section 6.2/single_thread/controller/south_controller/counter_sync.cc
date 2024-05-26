
#include "counter_sync.hpp"


void cb_bfrt_table_sync_synchriozed_counter_sync_cookie(const bf_rt_target_t &dev_tgt, void *cookie) {
    synchriozed_counter_sync_cookie *sync_obj(reinterpret_cast<synchriozed_counter_sync_cookie*>(cookie));
    sync_obj->on_complete();
}

void do_sync_counters(bfrt::BfRtTable const* table, bf_rt_target_t dev_tgt, auto session) {
    std::unique_ptr<bfrt::BfRtTableOperations> opSync;
    my_assert(table->operationsAllocate(bfrt::TableOperationsType::COUNTER_SYNC, std::addressof(opSync)));
    std::function cb(cb_bfrt_table_sync_synchriozed_counter_sync_cookie);
    synchriozed_counter_sync_cookie cookie;
    opSync->counterSyncSet(*session, dev_tgt, cb, std::addressof(cookie));
    cookie.on_sync();
    my_assert(table->tableOperationsExecute(*opSync));
    cookie.wait_for_completion();
}

