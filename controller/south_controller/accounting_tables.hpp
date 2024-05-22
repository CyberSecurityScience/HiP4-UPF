
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
#include "pull_priority.hpp"

#include "unordered_dense.h"

#define COUNTDOWN_US 5000000

struct read_table_context {
    bool enable_priority;
    usize bs;
    ankerl::unordered_dense::map<u32, bf_rt_handle_t> cur_batched_handles;
    ankerl::unordered_dense::map<u32, bf_rt_handle_t> ingress_handles; // used by ingress pull worker only
    std::unique_ptr<PullPriorityContext> priority_context;
    bool in_use;

    read_table_context(bool enable_priority, SharedPullPriorityContext *shared_priority_context_state): enable_priority(enable_priority), priority_context(nullptr) {
        bs = 10000;
        if (enable_priority) {
            priority_context = std::make_unique<PullPriorityContext>(shared_priority_context_state);
        } else {
            ingress_handles.reserve(200 * 1000);
        }
        in_use = false;
    }

    void begin_pull() {
        in_use = true;
    }

    void end_pull() {
        in_use = false;
    }

    bool should_i_sync_table() {
        return true;
    }

    void task_to_do_when_syncing_table(u64 ts) {
        begin_pull();
        if (!enable_priority) {
            // for ingress we just read the entire table
            for (auto& [maid, handle] : ingress_handles) {
                cur_batched_handles[maid] = handle;
            }
            return;
        }
        priority_context->flush_urr_updates(); // finish all pending URR updates
        cur_batched_handles = priority_context->generate_pulling_batch(ts, bs);
    }

    void insert_or_update_urr(InsertOrUpdatePerMaidURR urr_op, bool update_now) {
        if (enable_priority) {
            update_now = false;
            priority_context->add_or_update_per_maid_urr(urr_op, update_now);
        } else {
            
        }
    }

    void add_maid(u64 ts, u32 maid, bf_rt_handle_t handle) {
        if (enable_priority) {
            priority_context->associate_maid_with_handle(maid, handle, ts);
        } else {
            ingress_handles.insert({maid, handle});
        }
    }

    void remove_maid(u32 maid) {
        cur_batched_handles.erase(maid);
        if (enable_priority) {
            priority_context->remove_maid(maid, false);
        } else {
            ingress_handles.erase(maid);
        }
    }

    ankerl::unordered_dense::map<u32, bf_rt_handle_t> const& get_batch() {
        return cur_batched_handles;
    }

    void update_vol(u32 ma_id, u64 ts_us, u64 vol) {
        if (!enable_priority)
            return;
        priority_context->update_maid_vol(ma_id, vol, ts_us);
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
    AccountingTableValidity *entries,
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
        bool is_ingress,
        SharedPullPriorityContext *shared_priority_context_state
    ): table(nullptr), is_ingress(is_ingress) {
        my_assert(bf_rt_info->bfrtTableFromNameGet(table_name, std::addressof(table)));
        my_assert(table->keyFieldIdGet(key_ma_id, std::addressof(id_key_ma_id)));
        my_assert(table->actionIdGet(action_inc_counter, std::addressof(id_action_inc_counter)));
        my_assert(table->dataFieldIdGet("$COUNTER_SPEC_PKTS", std::addressof(id_data_counter_pkts)));
        my_assert(table->dataFieldIdGet("$COUNTER_SPEC_BYTES", std::addressof(id_data_counter_bytes)));
        read_context = new read_table_context(!is_ingress, shared_priority_context_state);
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

    void insert_or_update_urr(InsertOrUpdatePerMaidURR urr_op, bool update_now) {
        read_context->insert_or_update_urr(urr_op, update_now);
    }

    bf_status_t insert_new_ma_ids(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, std::vector<std::uint32_t> const& ma_ids, std::vector<bf_rt_handle_t> &out_handles) {
        std::unique_ptr<bfrt::BfRtTableKey> key;
        my_assert(table->keyAllocate(std::addressof(key)));
        std::unique_ptr<bfrt::BfRtTableData> data;
        my_assert(table->dataAllocate(std::addressof(data)));
        my_assert(table->dataReset(id_action_inc_counter, data.get()));
        out_handles.reserve(ma_ids.size());
        u64 flags(0);
        for (auto ma_id : ma_ids) {
            my_assert(table->keyReset(key.get()));
            my_assert(key->setValue(id_key_ma_id, uint64_t(ma_id)));

            return_if_failed(table->tableEntryAdd(*session, dev_tgt, flags, *key, *data));
            bf_rt_handle_t handle;
            return_if_failed(table->tableEntryHandleGet(*session, dev_tgt, flags, *key, &handle));
            out_handles.push_back(handle);
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
        read_context->task_to_do_when_syncing_table(ts);
        counter_sync_cookie.wait_for_completion();
    }

    // you need to call sync_counters_block before reading
    usize read_counters_and_send(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, UnixDomainSocket *raw_if, AccountingTableValidity *entries, u64 ts) {
        read_context->begin_pull();
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
            entries,
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

struct AccountingTableWithQueues {
    AccountingTable table;
    std::deque<std::uint32_t> pending_del_queue;
    std::deque<std::uint32_t> imm_del_queue;
    AccountingTableValidity entry_validity;

    std::mutex *pending_del_queue_mutex;

    u32 table_id;
    bool is_ingress;

    AccountingTableWithQueues(
        bfrt::BfRtInfo const *bf_rt_info,
        std::string table_name,
        std::string key_ma_id,
        std::string action_inc_counter,
        u32 table_id,
        bool is_ingress,
        SharedPullPriorityContext *shared_priority_context_state
    ) :
        table(bf_rt_info, table_name, key_ma_id, action_inc_counter, is_ingress, shared_priority_context_state),
        pending_del_queue(),
        imm_del_queue(),
        entry_validity(),
        pending_del_queue_mutex(new std::mutex()),
        table_id(table_id),
        is_ingress(is_ingress) {
    }

    ~AccountingTableWithQueues() noexcept {
        if (pending_del_queue_mutex) {
            delete pending_del_queue_mutex;
            pending_del_queue_mutex = nullptr;
        }
    }

    void add_to_pending_del_queue(std::vector<std::uint32_t> const& ma_ids) {
        std::lock_guard<std::mutex> lock(*pending_del_queue_mutex);
        pending_del_queue.insert(pending_del_queue.end(), ma_ids.cbegin(), ma_ids.cend());
        // for (auto ma_id:ma_ids) {
        //     println(" -- [add_to_pending_del_queue] [", table_id, "] added ", ma_id);
        // }
    }

    usize check_pending_queue() {
        std::lock_guard<std::mutex> lock(*pending_del_queue_mutex);
        // for (auto ma_id:pending_del_queue) {
        //     println(" -- [check_pending_queue] [", table_id, "] checking if ", ma_id, " is still valid");
        // }
        
        // loop through ALL entries and compare timestamps to find MAIDs no longer needed
        auto imm_del_entries = entry_validity.find_invalid_maids(pending_del_queue);
        imm_del_queue.insert(imm_del_queue.end(), imm_del_entries.cbegin(), imm_del_entries.cend());
        // println(" -- [check_pending_queue] [", table_id, "] imm_del_queue.size()=",imm_del_queue.size());

        // dedup imm_del_queue
        std::unordered_set<u32> imm_del_queue_set(imm_del_queue.cbegin(), imm_del_queue.cend());
        imm_del_queue.assign(imm_del_queue_set.cbegin(), imm_del_queue_set.cend());
        // println(" -- [check_pending_queue] [", table_id, "] imm_del_queue.size() after dedup=",imm_del_queue.size());

        // dedup pending_del_queue
        std::unordered_set<u32> pending_del_queue_set(pending_del_queue.cbegin(), pending_del_queue.cend());
        for (auto const ma_id : imm_del_entries)
            pending_del_queue_set.erase(ma_id);
        pending_del_queue.assign(pending_del_queue_set.cbegin(), pending_del_queue_set.cend());

        return imm_del_entries.size();
    }

    void insert_or_update_urr(InsertOrUpdatePerMaidURR urr_op, bool update_now) {
        table.insert_or_update_urr(urr_op, update_now);
    }

    bf_status_t insert_new_ma_ids(u64 ts, std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, std::vector<std::uint32_t> const& ma_ids) {
        std::vector<bf_rt_handle_t> handles;
        auto ret(table.insert_new_ma_ids(session, dev_tgt, ma_ids, handles));
        if (ret == BF_SUCCESS) {
            AccountingTableValidityEntry v;
            v.last_pulled_ts = 0;
            v.valid_until_ts = UINT64_MAX;
            for (usize i(0); i != ma_ids.size(); ++i) {
                auto ma_id = ma_ids[i];
                entry_validity.entries[ma_id] = v;
                table.read_context->add_maid(ts, ma_id, handles[i]);
                //println(" -- [insert_new_ma_ids] [",table_id,"] inserted ", ma_id);
            }
        }
        return ret;
    }

    bf_status_t delete_ma_ids(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, std::vector<std::uint32_t> const& ma_ids) {
        auto ret(table.delete_ma_ids(session, dev_tgt, ma_ids));
        if (ret == BF_SUCCESS) {
            for (u32 ma_id : ma_ids) {
                entry_validity.entries.erase(ma_id);
                table.read_context->remove_maid(ma_id);
                //println(" -- [delete_ma_ids] [",table_id,"] removed ", ma_id);
            }
        }
        return ret;
    }

    auto num_imm_del_entries() -> std::size_t {
        return imm_del_queue.size();
    }

    void sync_counters_block(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, u64 ts) {
        table.sync_counters_block(session, dev_tgt, ts);
    }

    usize read_counters_and_send(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt, UnixDomainSocket *raw_if, u64 ts) {
        return table.read_counters_and_send(session, dev_tgt, raw_if, &entry_validity, ts);
    }

    u32 size(std::shared_ptr<bfrt::BfRtSession> session, bf_rt_target_t dev_tgt) {
        return table.size(session, dev_tgt);
    }

    void update_pull_ts(u64 ts) {
        //println(" -- [update_pull_ts] pull at ", ts);
        entry_validity.update_pull_ts(ts);
    }

    void update_valid_until(u32 ma_id, u64 ts) {
        //println(" -- [update_valid_until] ma id ",ma_id," valid until ", ts);
        entry_validity.entries.at(ma_id).valid_until_ts = ts;
    }

    std::vector<std::uint32_t> get_imm_delete_entries(std::size_t limit) {
        auto const n_ret(std::min(limit, imm_del_queue.size()));
        std::vector<std::uint32_t> ret;
        ret.reserve(n_ret);
        auto it_begin(imm_del_queue.begin());
        auto it(it_begin);
        for (std::size_t i(0); i != n_ret; ++i) {
            ret.push_back(*it);
            ++it;
        }
        imm_del_queue.erase(it_begin, it);
        return ret;
    }
};

#endif
