
#include "common.hpp"
#include "update_worker.hpp"
#include "interface.hpp"
#include "batch_scheduler.hpp"
#include "accounting_table_group.hpp"
#include "pdr_tables.hpp"
#include "gk.hpp"

void update_worker(
    UnixSocketInterface *interface,
    bf_rt_target_t dev_tgt,
    PDRTables *pdr_tables,
    TableGroup *ingress_table_group,
    TableGroup *egress_table_group,
    GlobalPerformanceObjects *perf_objs,
    bool *running,
    u64 log_freq_us
) {
    auto session = bfrt::BfRtSession::sessionCreate();
    std::uint64_t wait_for_requets_timeout_us(UINT64_MAX);
    BatchScheduler batch_sch(perf_objs);
    u64 ctr(0);
    u64 next_log_milestone(get_ts_us() + log_freq_us);

    stmpct::gk<u64> g_update(0.1), g_send(0.1), g_os_overhead(0.01);

    while (*running) {
        ++ctr;
        u64 os_delay(0);
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
            requests_opt = interface->wait_for_requests(wait_for_requets_timeout_us);
            if (requests_opt.has_value()) {
                auto requests(requests_opt.value());
                auto [new_timeout, updates_opt] = batch_sch.submit_request(requests.cbegin(), requests.cend(), num_updates);
                if (updates_opt.has_value()) {
                    transactions = updates_opt.value();
                }
                wait_for_requets_timeout_us = new_timeout;
                // os_delay = get_ts_us() - requests[0].deadline;
                // g_os_overhead.insert(os_delay);
                // transactions = requests;
                // for (auto const &c : transactions) {
                //     num_updates += c.ops.size();
                // }
            } else {
                // timed out
                transactions = batch_sch.get_current_batch(num_updates);
                wait_for_requets_timeout_us = UINT64_MAX;
            }
        } catch (std::exception ex) {
            perrln("[!] [update_worker] error getting requests:", fast_io::mnp::os_c_str(ex.what()));
        }
        if (num_updates == 0) {
            continue;
        }
        if (log_this_round) {
            println("[+] [update_worker] round ", ctr," issued ", transactions.size() ," transactions with ", num_updates," update ops");
            println("[+] [update_worker] round ", ctr," time for 10 update ", batch_sch.time_needed_to_perform_n_updates(10), " us");
        }

//#if !defined(NDEBUG)
        // println("[+] [update_worker] round ", ctr," issued ", transactions.size() ," transactions with ", num_updates," update ops");
        // auto cur_ts22(get_ts_us());
        // for (auto const& v:transactions) {
        //     println(" -- [update_worker] cur_ts ", cur_ts22," while msg is sent at ", v.deadline);
        // }
//#endif

        {
            PerformanceCountScopeTracker all_perf(&perf_objs->update_worker_all_updates_perf, num_updates);
            u64 update_start_ts(get_ts_us());

            // step 2:
            // issue to table groups
            bool succeed(false);

            // track changed ma_ids
            std::vector<std::uint32_t> new_ma_ids;
            new_ma_ids.reserve(8);
            std::vector<std::uint32_t> old_ma_ids;
            old_ma_ids.reserve(8);
            std::vector<CopyablePDROperation> update_ops;
            update_ops.reserve(num_updates);
            for (auto const& op : transactions) {
                update_ops.insert(update_ops.end(), op.ops.cbegin(), op.ops.cend());
            }


            // track URR updates
            std::vector<InsertOrUpdatePerMaidURR> urr_updates;

            std::vector<u32> deleted_maids_ig;
            std::vector<u32> deleted_maids_eg;

            //session->beginTransaction(true);
            bf_status_t errcode(BF_SUCCESS);
            errcode = session->beginBatch();
            if (errcode != BF_SUCCESS) {
                perrln("[!] [update_worker] begin batch failed with code ", errcode);
                goto END_ROUND;
            }
            {
                std::lock_guard<std::mutex> lock_ig_update_table(ingress_table_group->table_usage_update_mutex);
                std::lock_guard<std::mutex> lock_eg_update_table(egress_table_group->table_usage_update_mutex);

                u64 update_ts(get_ts_us());

                {
                    PerformanceCountScopeTracker pdr_perf(&perf_objs->update_worker_pdr_updates_perf, update_ops.size());
                    errcode = pdr_tables->batch_update(session, dev_tgt, update_ops, new_ma_ids, old_ma_ids, urr_updates);
                }
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] PDR update failed with code ", errcode);
                    goto END_ROUND;
                }
                {
                    errcode = ingress_table_group->delete_ids_in_imm_del_queue(session, dev_tgt, deleted_maids_ig);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Ingress DEL IMM update failed with code ", errcode);
                        goto END_ROUND;
                    }
                }
                {
                    PerformanceCountScopeTracker ig_perf(&perf_objs->update_worker_ingress_accounting_updates_perf, new_ma_ids.size() + old_ma_ids.size());
                    errcode = ingress_table_group->commit_batch(update_ts, session, dev_tgt, new_ma_ids, old_ma_ids, urr_updates);
                }
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] Ingress update failed with code ", errcode);
                    goto END_ROUND;
                }
                {
                    std::vector<u32> deleted_maids;
                    errcode = egress_table_group->delete_ids_in_imm_del_queue(session, dev_tgt, deleted_maids_eg);
                    if (errcode != BF_SUCCESS) {
                        perrln("[!] [update_worker] Egress DEL IMM update failed with code ", errcode);
                        goto END_ROUND;
                    }
                }
                {
                    PerformanceCountScopeTracker eg_perf(&perf_objs->update_worker_egress_accounting_updates_perf, new_ma_ids.size() + old_ma_ids.size());
                    errcode = egress_table_group->commit_batch(update_ts, session, dev_tgt, new_ma_ids, old_ma_ids, urr_updates);
                }
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] Egress update failed with code ", errcode);
                    goto END_ROUND;
                }
                errcode = session->endBatch(true);
                if (errcode != BF_SUCCESS) {
                    perrln("[!] [update_worker] commit failed with code ", errcode);
                    goto END_ROUND;
                }
                auto valid_util(get_ts_us());
                ingress_table_group->update_ma_ids_valid_until(valid_util, old_ma_ids);
                egress_table_group->update_ma_ids_valid_until(valid_util, old_ma_ids);
                succeed = true;

                // u64 update_end_ts(get_ts_us());
                // u64 diff(update_end_ts - update_start_ts);
                // g_update.insert(diff);
            }

            // step 3:
            // let UPF rust know
            // send finished updates to UNIX socket
END_ROUND:
            {
                u64 send_start_ts(get_ts_us());
                PerformanceCountScopeTracker send_perf(&perf_objs->update_worker_send_to_rust_perf, transactions.size());
                // TODO: log error
                if (succeed) {
                    //perrln("[!] [update_worker] round ", ctr," all transactions succeed");
                    interface->send_deleted_ma_ids(deleted_maids_ig);
                    interface->send_deleted_ma_ids(deleted_maids_eg);
                } else {
                    perrln("[!] [update_worker] round ", ctr," all transactions failed, code ", errcode);
                    session->abortTransaction();
                    session->endBatch(false);
                }
                // u64 send_end_ts(get_ts_us());
                // u64 diff(send_end_ts - send_start_ts);
                // g_send.insert(diff);
                interface->complete_requests(transactions, errcode);
            }
        }
        if (log_this_round) {
            double update_p50(g_update.quantile(0.5));
            double update_p95(g_update.quantile(0.95));
            double send_p50(g_send.quantile(0.5));
            double send_p95(g_send.quantile(0.95));
            println("[+] [update_worker] round ", ctr," performed ", perf_objs->update_worker_all_updates_perf.last_num_obj," ops in ", perf_objs->update_worker_all_updates_perf.last_time_used, " us"
                // ,
                // ", update p50=", update_p50,
                // ", update p95=", update_p95,
                // ", send p50=", send_p50,
                // ", send p95=", send_p95,
                // ", os_delay=", os_delay,
                // ", os_delay p50=", g_os_overhead.quantile(0.5),
                // ", os_delay p95=", g_os_overhead.quantile(0.95)
            );
        }
    }
}
