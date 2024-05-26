
/*
2 ingress simple tables
TODO: ingress complex tables ignored
1 ingress priority switching table

4 egress accounting tables
*/

#ifndef _ACCOUNTING_TABLES_H_
#define _ACCOUNTING_TABLES_H_

#include <cstdint>
#include <span>
#include <functional>
#include <unordered_set>
#include <new>
#include <mutex>
#include <deque>

#include "common.hpp"
#include "performance_tracker.hpp"
#include "counter_sync.hpp"
#include "domain_socket.hpp"

#include "unordered_dense.h"

#define COUNTDOWN_US 5000000

struct read_table_context {
    ankerl::unordered_dense::map<u32, bf_rt_handle_t> handles;
    usize bs;
    usize cur_batch_idx;
    usize num_batches;
    std::vector<std::unique_ptr<ankerl::unordered_dense::map<u32, bf_rt_handle_t>>> batched_handles;


    read_table_context() {
        bs = 100;
        cur_batch_idx = 0;
        num_batches = 0;
    }

    bool should_i_sync_table() {
        return cur_batch_idx == 0 && handles.size() >= 300000;
    }

    void task_to_do_when_syncing_table() {
        if (handles.size() == 0) {
            num_batches = 0;
            return;
        }
        u64 start = get_ts_us();
        // re-generate batches
        batched_handles.clear();
        num_batches = (handles.size() - 1) / bs + 1;
        batched_handles.reserve(num_batches);
        for (usize i(0); i != num_batches; ++i) {
            batched_handles.emplace_back(std::make_unique<ankerl::unordered_dense::map<u32, bf_rt_handle_t>>());
        }
        usize i(0);
        for (auto const& [maid, handle] : handles) {
            usize batch_idx(i / bs);
            (*batched_handles[batch_idx])[maid] = handle;
            ++i;
        }
        u64 elp = get_ts_us() - start;
        //println("[!!] [task_to_do_when_syncing_table] formed ",num_batches," batches in ",elp," us");
    }

    void init(bfrt::BfRtTable const *table, u32 n) {
    }

    void add_maid(u32 maid, bf_rt_handle_t handle) {
        handles[maid] = handle;
    }

    void remove_maid(u32 maid) {
        handles.erase(maid);
        for (usize i(cur_batch_idx); i != num_batches; ++i) {
            batched_handles[i]->erase(maid);
        }
    }

    bool has_batch() const {
        return batched_handles.size() != 0;
    }

    ankerl::unordered_dense::map<u32, bf_rt_handle_t> const& get_batch() {
        return *batched_handles[cur_batch_idx];
    }

    void advance_to_next_batch() {
        ++cur_batch_idx;
        //println("[!!] [advance_to_next_batch] at batch idx=",cur_batch_idx);
        if (cur_batch_idx >= num_batches) {
            cur_batch_idx = 0;
        }
    }
};

struct CounterResult {
    std::uint64_t bytes;
    std::uint64_t pkts;
    std::uint32_t ma_id;

    CounterResult(std::uint32_t ma_id, std::uint64_t bytes, std::uint64_t pkts): bytes(bytes), pkts(pkts), ma_id(ma_id) {

    }
};


/////////////////////////////////////////////////////////////////////
//                        Per AccountingTable
/////////////////////////////////////////////////////////////////////

/// 
struct AccountingTablePendingDeleteionQueue {

};

struct AccountingTableValidityEntry {
    std::uint64_t last_pulled_ts;
    std::uint64_t valid_until_ts;
};

/// @brief For each table entry we have two timestamps
struct AccountingTableValidity {
    ankerl::unordered_dense::map<std::uint32_t, AccountingTableValidityEntry> entries;

    /// @brief call me after counter sync and before counter read
    void update_pull_ts(std::uint64_t new_ts) {
        for (auto& [k, v] : entries) {
            v.last_pulled_ts = new_ts;
        }
    }

    std::unordered_set<std::uint32_t> find_invalid_maids(std::deque<std::uint32_t> const& to_del_maids) {
        std::unordered_set<std::uint32_t> ret(to_del_maids.size());
        for (auto const maid : to_del_maids) {
            auto const& entry = entries.at(maid);
            if (entry.last_pulled_ts >= entry.valid_until_ts) {
                ret.insert(maid);
            }
        }
        return ret;
    }

    AccountingTableValidity() {
        entries.reserve(168 * 1024);
    }

    ~AccountingTableValidity() noexcept {
    }
};


usize read_counters_impl(
    bfrt::BfRtTable const *table,
    bf_rt_id_t id_key_ma_id,
    bf_rt_id_t id_data_counter_pkts,
    bf_rt_id_t id_data_counter_bytes,
    std::shared_ptr<bfrt::BfRtSession> session,
    bf_rt_target_t dev_tgt,
    read_table_context *ctx, // per table
    UnixDomainSocket *raw_if,
    bool is_ingress,
    u64 ts
);

struct AccountingTable {
    bf_rt_id_t id_key_ma_id;
    bf_rt_id_t id_action_inc_counter;
    bf_rt_id_t id_data_counter_pkts;
    bf_rt_id_t id_data_counter_bytes;

    bfrt::BfRtTable const *table;
    read_table_context *read_context;

    synchriozed_counter_sync_cookie counter_sync_cookie;
    bool is_ingress;

    AccountingTable(
        bfrt::BfRtInfo const *bf_rt_info,
        std::string table_name,
        std::string key_ma_id,
        std::string action_inc_counter,
        bool is_ingress
    ): table(nullptr), is_ingress(is_ingress) {
        my_assert(bf_rt_info->bfrtTableFromNameGet(table_name, std::addressof(table)));
        my_assert(table->keyFieldIdGet(key_ma_id, std::addressof(id_key_ma_id)));
        my_assert(table->actionIdGet(action_inc_counter, std::addressof(id_action_inc_counter)));
        my_assert(table->dataFieldIdGet("$COUNTER_SPEC_PKTS", std::addressof(id_data_counter_pkts)));
        my_assert(table->dataFieldIdGet("$COUNTER_SPEC_BYTES", std::addressof(id_data_counter_bytes)));
        read_context = new read_table_context;
    }

    ~AccountingTable() noexcept {
        if (table) {
            delete table;
            table = nullptr;
        }
        if (read_context) {
            delete read_context;
            read_context = nullptr;
        }
    }

    bf_status_t insert_new_ma_ids(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, std::vector<std::tuple<std::uint32_t, u64>> const& ma_ids) {
        std::unique_ptr<bfrt::BfRtTableKey> key;
        my_assert(table->keyAllocate(std::addressof(key)));
        std::unique_ptr<bfrt::BfRtTableData> data;
        my_assert(table->dataAllocate(std::addressof(data)));
        my_assert(table->dataReset(id_action_inc_counter, data.get()));
        u64 flags(0);
        for (auto [ma_id, vol_thres] : ma_ids) {
            my_assert(table->keyReset(key.get()));
            my_assert(key->setValue(id_key_ma_id, uint64_t(ma_id)));

            return_if_failed(table->tableEntryAdd(*session, dev_tgt, flags, *key, *data));
            bf_rt_handle_t handle;
            return_if_failed(table->tableEntryHandleGet(*session, dev_tgt, flags, *key, &handle));
            read_context->add_maid(ma_id, handle);
        }
        return BF_SUCCESS;
    }

    bf_status_t delete_ma_ids(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, std::vector<std::uint32_t> const& ma_ids) {
        std::unique_ptr<bfrt::BfRtTableKey> key;
        my_assert(table->keyAllocate(std::addressof(key)));
        for (auto ma_id : ma_ids) {
            my_assert(table->keyReset(key.get()));
            my_assert(key->setValue(id_key_ma_id, uint64_t(ma_id)));

            return_if_failed(table->tableEntryDel(*session, dev_tgt, *key));
        }
        return BF_SUCCESS;
    }

    void sync_counters_block(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, u64 ts) {
        std::unique_ptr<bfrt::BfRtTableOperations> opSync;
        my_assert(table->operationsAllocate(bfrt::TableOperationsType::COUNTER_SYNC, std::addressof(opSync)));
        std::function cb(cb_bfrt_table_sync_synchriozed_counter_sync_cookie);
        opSync->counterSyncSet(*session, dev_tgt, cb, std::addressof(counter_sync_cookie));
        counter_sync_cookie.on_sync();
        my_assert(table->tableOperationsExecute(*opSync));
        read_context->task_to_do_when_syncing_table();
        counter_sync_cookie.wait_for_completion();
    }

    // you need to call sync_counters_block before reading
    usize read_counters_and_send(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, UnixDomainSocket *raw_if, u64 ts) {
        if (read_context->should_i_sync_table()) {
            sync_counters_block(session, dev_tgt, ts);
        }
        return read_counters_impl(
            table,
            id_key_ma_id,
            id_data_counter_pkts,
            id_data_counter_bytes,
            session,
            dev_tgt,
            read_context,
            raw_if,
            is_ingress,
            ts
        );
    }

    u32 size(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt) {
        uint32_t entry_count = 0;
        table->tableUsageGet(*session, dev_tgt, bfrt::BfRtTable::BfRtTableGetFlag::GET_FROM_SW, std::addressof(entry_count));
        return entry_count;
    }
};

#endif
