
#ifndef _ACCOUNTING_TABLE_GROUP_H_
#define _ACCOUNTING_TABLE_GROUP_H_

#include <ranges>

#include "common.hpp"
#include "accounting_tables.hpp"
#include "unordered_dense.h"

enum class TableGroupPlacement {
    Ingress,
    Egress
};

struct TableGroup {
    std::vector<std::unique_ptr<AccountingTableWithQueues>> tables;
    // why 2 tables instead of 3 or 4?
    // 1. less supported UEs (160k to 157k)
    // 2. not singinificant pulling delay advantage (maybe 100ms better)
    // 3. more threads needed on a low-end CPU
    // 4. threads need to synchronize introduce additional wait time and complexity
    ankerl::unordered_dense::map<std::uint32_t, std::uint8_t> ma_id2table_id;
    std::uint8_t update_table_id;
    std::mutex table_usage_update_mutex;

    TableGroup(
        bfrt::BfRtInfo const *bf_rt_info,
        std::size_t n_tables,
        TableGroupPlacement placement,
        SharedPullPriorityContext *shared_priority_context_state
    ): tables(), ma_id2table_id(2 * 168 * 1024), update_table_id(0) {
        char buf_tablename[256];
        char buf_actionname[256];
        for (std::size_t i(0); i != n_tables; ++i) {
            bool is_ingress;
            if (placement == TableGroupPlacement::Ingress) {
                is_ingress = true;
                sprintf(buf_tablename, "pipe.Ingress.accounting.accounting_exact%u", std::uint32_t(i + 1));
                sprintf(buf_actionname, "Ingress.accounting.inc_counter%u", std::uint32_t(i + 1));
            } else {
                is_ingress = false;
                sprintf(buf_tablename, "pipe.Egress.accounting.accounting_exact%u", std::uint32_t(i + 1));
                sprintf(buf_actionname, "Egress.accounting.inc_counter%u", std::uint32_t(i + 1));
            }
            tables.push_back(std::make_unique<AccountingTableWithQueues>(bf_rt_info, buf_tablename, "ma_id", buf_actionname, i, is_ingress, shared_priority_context_state));
        }
    }

    ~TableGroup() noexcept {
        try {
            ;
        } catch(...) {

        }
    }

    auto& get_pulling_table() {
        //std::lock_guard<std::mutex> lock(table_usage_update_mutex);
        return tables[update_table_id ^ 1];
    }

    /// @brief  call me within a BfRt transaction, after PDR table insert
    /// @param inserts 
    /// @param deletes 
    bf_status_t commit_batch(
        u64 ts,
        std::shared_ptr<bfrt::BfRtSession> session,
        bf_rt_target_t dev_tgt,
        std::vector<std::uint32_t> const& inserts,
        std::vector<std::uint32_t> const& deletes,
        std::vector<InsertOrUpdatePerMaidURR> const& urr_updates
    ) {
        {
            // std::lock_guard<std::mutex> lock(table_usage_update_mutex); // already lock in update_worker.cc
            auto update_table_id_copy = update_table_id;

            auto& update_table = tables[update_table_id_copy];

            // step 0: update MAID to table map
            for (auto ma_id : inserts) {
                ma_id2table_id[ma_id] = update_table_id_copy;
            }

            // step 1: update URRs
            for (auto& urr_update : urr_updates) {
                auto its_table_id(ma_id2table_id[urr_update.ma_id]);
                if (its_table_id == update_table_id_copy) {
                    // this URR is found in table partition in WRITE state, can update now
                    update_table->insert_or_update_urr(urr_update, true);
                } else {
                    // this URR is found in table partition in PULL state, put into pending queue to avoid race condition since URR is used during pull
                    tables[its_table_id]->insert_or_update_urr(urr_update, false);
                }
            }

            // step 2: insert
            return_if_failed(update_table->insert_new_ma_ids(ts, session, dev_tgt, inserts));
        }

        // step 3: put deleted PDRs into pending del queue
        std::vector<std::vector<std::uint32_t>> per_table_to_del_ma_ids;
        per_table_to_del_ma_ids.reserve(tables.size());
        for (std::size_t i(0); i != tables.size(); ++i) {
            per_table_to_del_ma_ids.push_back({});
            per_table_to_del_ma_ids[i].reserve(deletes.size());
        }
        for (auto ma_id : deletes) {
            auto table_id(ma_id2table_id.at(ma_id));
            per_table_to_del_ma_ids[table_id].push_back(ma_id);
        }

        for (std::size_t i(0); i != tables.size(); ++i) {
            if (per_table_to_del_ma_ids[i].size() != 0) {
                tables[i]->add_to_pending_del_queue(per_table_to_del_ma_ids[i]);
            }
        }

        return BF_SUCCESS;
    }

    bf_status_t delete_ids_in_imm_del_queue(
        std::shared_ptr<bfrt::BfRtSession> session,
        bf_rt_target_t dev_tgt,
        std::vector<u32>& deleted_maids
    ) {
        auto& update_table = tables[update_table_id];
        deleted_maids = update_table->get_imm_delete_entries(10);
        if (deleted_maids.size() == 0)
            return BF_SUCCESS;
        bf_status_t ret(update_table->delete_ma_ids(session, dev_tgt, deleted_maids));
        if (ret != BF_SUCCESS) {
            // rollback
            update_table->imm_del_queue.insert(update_table->imm_del_queue.end(), deleted_maids.cbegin(), deleted_maids.cend());
        }
        return ret;
    }

    void update_ma_ids_valid_until(u64 valid_until, std::vector<u32> const& to_del_ma_ids) {
        for (auto ma_id : to_del_ma_ids) {
            auto table_id(ma_id2table_id.at(ma_id));
            tables[table_id]->update_valid_until(ma_id, valid_until);
        }
    }

    void switch_table_usage() {
        //println("trying to lock table_usage_update_mutex");
        std::lock_guard<std::mutex> lock(table_usage_update_mutex);
        //println("done locking table_usage_update_mutex");

        // find the table with least amount of entries AND not the current one, use that as the next update table
        update_table_id ^= 1;
    }
};

#endif
