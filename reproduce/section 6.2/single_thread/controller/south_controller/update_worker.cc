
#include "common.hpp"
#include "update_worker.hpp"
#include "interface.hpp"
#include "batch_scheduler.hpp"
#include "accounting_table_group.hpp"
#include "pdr_tables.hpp"

void update_worker(
    UnixSocketInterface *interface,
    bf_rt_target_t dev_tgt,
    PDRTables *pdr_tables,
    AccountingTable *egress_table,
    AccountingTable *ingress_table,
    GlobalPerformanceObjects *perf_objs,
    bool *running,
    u64 log_freq_us
) {
    auto session = bfrt::BfRtSession::sessionCreate();
    std::uint64_t wait_for_requets_timeout_us(UINT64_MAX);
    BatchScheduler batch_sch(perf_objs);
    u64 ctr(0);
    u64 next_log_milestone(get_ts_us() + log_freq_us);

    while (*running) {
        ++ctr;
        bool log_this_round(false);
        if (get_ts_us() >= next_log_milestone) {
            next_log_milestone = get_ts_us() + log_freq_us;
            log_this_round = true;
        }
        if (log_this_round) {
            println("[+] [update_worker] round ", ctr," begin");
        }
        // step 1:
        // form a batch of requests
        // using epoll with timeout
        std::vector<CopyableTransaction> transactions;
        std::size_t num_updates(0);
        std::optional<std::vector<CopyableTransaction>> requests_opt{};
        try {
            requests_opt = interface->wait_for_requests(1);
            if (requests_opt.has_value()) {
                auto requests(requests_opt.value());
                auto [new_timeout, updates_opt] = batch_sch.submit_request(requests.cbegin(), requests.cend(), num_updates);
                if (updates_opt.has_value()) {
                    transactions = updates_opt.value();
                }
                wait_for_requets_timeout_us = new_timeout;
            }
        } catch (std::exception ex) {
            perrln("[!] [update_worker] error getting requests:", fast_io::mnp::os_c_str(ex.what()));
        }
        usize num_pull(0);
        if (log_this_round) {
            println("[+] [update_worker] round ", ctr," issued ", transactions.size() ," transactions with ", num_updates," update ops");
            println("[+] [update_worker] round ", ctr," time for 10 update ", batch_sch.time_needed_to_perform_n_updates(10), " us");
        }

//#if !defined(NDEBUG)
        //println("[+] [update_worker] round ", ctr," issued ", transactions.size() ," transactions with ", num_updates," update ops");
        // auto cur_ts22(get_ts_us());
        // for (auto const& v:transactions) {
        //     println(" -- [update_worker] cur_ts ", cur_ts22," while msg is sent at ", v.deadline);
        // }
//#endif
        u64 pull_used(0);

        if (num_updates != 0) {
            PerformanceCountScopeTracker all_perf(&perf_objs->update_worker_all_updates_perf, num_updates);

            // step 2:
            // issue to table groups
            bool succeed(false);

            // track changed ma_ids
            std::vector<std::tuple<std::uint32_t, u64>> new_ma_ids;
            new_ma_ids.reserve(256);
            std::vector<std::uint32_t> old_ma_ids;
            old_ma_ids.reserve(256);
            std::vector<CopyablePDROperation> update_ops;
            update_ops.reserve(num_updates);
            for (auto const& op : transactions) {
                update_ops.insert(update_ops.end(), op.ops.cbegin(), op.ops.cend());
            }

            //session->beginTransaction(true);
            bf_status_t errcode(BF_SUCCESS);
            errcode = session->beginBatch();
            if (errcode != BF_SUCCESS) {
                perrln("[!] [update_worker] begin batch failed with code ", errcode);
                goto END_ROUND;
            }
            {
                u64 update_ts(get_ts_us());

                // update PDR table
                {
                    PerformanceCountScopeTracker pdr_perf(&perf_objs->update_worker_pdr_updates_perf, update_ops.size());
                    errcode = pdr_tables->batch_update(session, dev_tgt, update_ops, new_ma_ids, old_ma_ids);
                }
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] PDR update failed with code ", errcode);
                    goto END_ROUND;
                }
                // update eg usage table
                {
                    errcode = egress_table->delete_ma_ids(session, dev_tgt, old_ma_ids);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Egress DEL update failed with code ", errcode);
                        goto END_ROUND;
                    }
                    errcode = egress_table->insert_new_ma_ids(session, dev_tgt, new_ma_ids);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Egress DEL update failed with code ", errcode);
                        goto END_ROUND;
                    }
                }
                // update ig usage table
                {
                    errcode = ingress_table->delete_ma_ids(session, dev_tgt, old_ma_ids);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Egress DEL update failed with code ", errcode);
                        goto END_ROUND;
                    }
                    errcode = ingress_table->insert_new_ma_ids(session, dev_tgt, new_ma_ids);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Egress DEL update failed with code ", errcode);
                        goto END_ROUND;
                    }
                }
                // commit batch
                errcode = session->endBatch(true);
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] commit failed with code ", errcode);
                    goto END_ROUND;
                }
                succeed = true;
            }

            // step 3:
            // let UPF rust know
            // send finished updates to UNIX socket
END_ROUND:
            {
                PerformanceCountScopeTracker send_perf(&perf_objs->update_worker_send_to_rust_perf, transactions.size());
                // TODO: log error
                if (succeed) {
                    //perrln("[!] [update_worker] round ", ctr," all transactions succeed");
                    interface->send_deleted_ma_ids(old_ma_ids);
                } else {
                    perrln("[!] [update_worker] round ", ctr," all transactions failed, code ", errcode);
                    session->abortTransaction();
                    session->endBatch(false);
                }
                interface->complete_requests(transactions, errcode);
            }
        }  {
            // pull
            u64 pull_ts(get_ts_us());
            // bf_status_t errcode(BF_SUCCESS);
            // errcode = session->beginBatch();
            // if (errcode != BF_SUCCESS) {
            //     perrln("[!] [update_worker] [pull] failed to beginBatch ", errcode);
            // }
            num_pull = ingress_table->read_counters_and_send(session, dev_tgt, &interface->socket, pull_ts);
            egress_table->read_counters_and_send(session, dev_tgt, &interface->socket, pull_ts);
            // errcode = session->endBatch(true);
            // if (errcode != BF_SUCCESS) {
            //     perrln("[!] [update_worker] [pull] failed to endBatch ", errcode);
            // }
            //perf_objs->update_worker_all_updates_perf.last_num_obj = 0;
            pull_used = get_ts_us() - pull_ts;
        }
        if (log_this_round) {
            println("[+] [update_worker] round ", ctr," performed ", perf_objs->update_worker_all_updates_perf.last_num_obj," ops in ", perf_objs->update_worker_all_updates_perf.last_time_used, " us, read ", num_pull, " entries, used ", pull_used, " us");
            println("[+] [pull] round ", ctr, " ig count ", ingress_table->read_context->handles.size(), ", eg count ", egress_table->read_context->handles.size());
        }
    }
}
