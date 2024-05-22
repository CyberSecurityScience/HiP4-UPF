
#include "pull_worker.hpp"
#include "accounting_table_group.hpp"
#include "interface.hpp"

void pull_worker(
    TableGroup *group,
    UnixSocketInterface *interface,
    bf_rt_target_t dev_tgt,
    GlobalPerformanceObjects *perf_objs,
    bool is_ingress,
    bool *running,
    u64 log_freq_us
) {
    auto session = bfrt::BfRtSession::sessionCreate();
    u64 ctr(0);
    u64 next_log_milestone(get_ts_us() + log_freq_us);
    char const *log_id = is_ingress ? "Ingress" : "Egress";
    while (*running) {
        ++ctr;
        bool log_this_round(false);
        if (get_ts_us() >= next_log_milestone) {
            next_log_milestone = get_ts_us() + log_freq_us;
            log_this_round = true;
        }
        if (log_this_round) {
            println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] round ", ctr," begin");
        }

        // step 1:
        // select active table for PULLING
        auto& pull_table = group->get_pulling_table();

        {
            PerformanceCountObject *obj(nullptr);
            if (is_ingress) {
                obj = &perf_objs->pull_worker_ingress_ma_id_del_perf;
            } else {
                obj = &perf_objs->pull_worker_egress_ma_id_del_perf;
            }
            PerformanceCountScopeTracker perf(obj, 0);

            // step 2:
            // remove imm delete entries
            // delete WILL succeed, no transaction required
            auto ret_batch_begin(session->beginBatch());
            if (ret_batch_begin != BF_SUCCESS) {
                perrln("[!] [pull_worker] Error deleting MAIDs (beginBatch) code=", ret_batch_begin);
                break;
            }
            std::vector<std::uint32_t> entries = pull_table->get_imm_delete_entries(std::numeric_limits<std::size_t>::max()); // can limit number of returns in get_to_delete_entries, used for spreading deletion load
            perf.set_num_obj(entries.size());
            if (entries.size() != 0) {
                //ScopeTimer t("delete MAIDs");
                pull_table->delete_ma_ids(session, dev_tgt, entries);
            }
            auto ret_batch_end(session->endBatch(true));
            if (ret_batch_end != BF_SUCCESS) {
                perrln("[!] [pull_worker] Error deleting MAIDs (endBatch) code=", ret_batch_end);
                break;
            }
            // notify MAID deletion
            interface->send_deleted_ma_ids(entries);
            if (log_this_round) {
                println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] deleted ", entries.size()," MAIDs");
            }
        }

        u64 pull_ts(0);
        usize counter_values_count;
        PerformanceCountObject *read_sync_perf_obj(nullptr);
        {
            if (is_ingress) {
                read_sync_perf_obj = &perf_objs->pull_worker_ingress_sync_and_read_perf;
            } else {
                read_sync_perf_obj = &perf_objs->pull_worker_egress_sync_and_read_perf;
            }
            PerformanceCountScopeTracker perf(read_sync_perf_obj, 0);

            // step 3:
            pull_ts = get_ts_us();
            // // sync
            // pull_table->sync_counters_block(session, dev_tgt);
            // pull
            counter_values_count = pull_table->read_counters_and_send(session, dev_tgt, &interface->socket, pull_ts);
            {
                PerformanceCountObject *obj2(nullptr);
                if (is_ingress) {
                    obj2 = &perf_objs->pull_worker_ingress_update_last_pull_perf;
                } else {
                    obj2 = &perf_objs->pull_worker_egress_update_last_pull_perf;
                }
                PerformanceCountScopeTracker perf(obj2, pull_table->entry_validity.entries.size());
                pull_table->update_pull_ts(pull_ts);
            }
            // {
            //    using namespace std::chrono_literals;
            //    std::this_thread::sleep_for(1000ms);
            // }
            perf.set_num_obj(counter_values_count);
        }
        if (log_this_round) {
            auto num_entries_in_1_sec(read_sync_perf_obj->estimate_num_obj_can_be_processed(1000000));
            println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] read ", read_sync_perf_obj->last_num_obj," entries used ",read_sync_perf_obj->last_time_used,"us, at ", num_entries_in_1_sec, " entries/s, time_per_obj=", read_sync_perf_obj->avg_time_per_obj_);
        }

        // {
        //     PerformanceCountObject *obj(nullptr);
        //     if (is_ingress) {
        //         obj = &perf_objs->pull_worker_ingress_send_to_rust_perf;
        //     } else {
        //         obj = &perf_objs->pull_worker_egress_send_to_rust_perf;
        //     }
        //     PerformanceCountScopeTracker perf(obj, counter_values_count);
        //     // step 4:
        //     // push to rust
        //     if (counter_values_count != 0) {
        //         if (log_this_round) {
        //             println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] sending ", counter_values.size()," counter values to rust");
        //         }
        //         interface->send_counter_values(counter_values, is_ingress, pull_ts);
        //     }
        // }

        {
            PerformanceObject *obj(nullptr);
            if (is_ingress) {
                obj = &perf_objs->pull_worker_ingress_check_pending_queue_perf;
            } else {
                obj = &perf_objs->pull_worker_egress_check_pending_queue_perf;
            }
            PerformanceScopeTracker perf(obj);

            // step 5:
            // affected entry comparsion
            // no switching affect entry queue now, just put the queue behind a mutex
            // the affect entries will be pushed in by the update thread in batch
            //println("[pull_worker] checking pending queue");
            usize num_moved(pull_table->check_pending_queue());
            if (log_this_round) {
                println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] moved ", num_moved, " MAIDs to imm del queue");
            }
        }
        if (log_this_round) {
            PerformanceObject *obj(nullptr);
            if (is_ingress) {
                obj = &perf_objs->pull_worker_ingress_check_pending_queue_perf;
            } else {
                obj = &perf_objs->pull_worker_egress_check_pending_queue_perf;
            }
            println("[+] [pull_worker][",fast_io::mnp::os_c_str(log_id),"] check_pending_queue() used ", obj->value, "ms");
        }

        // step 6:
        // do PULLING/UPDATING table switching
        group->switch_table_usage();
    }
}
