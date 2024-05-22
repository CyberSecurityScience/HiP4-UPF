
#include "common.hpp"
#include "pull_worker.hpp"
#include "update_worker.hpp"
#include "interface.hpp"
#include "accounting_table_group.hpp"
#include "pdr_tables.hpp"
#include "pull_priority.hpp"

int main(int argc, char **argv) {
    parse_opts_and_switchd_init(argc, argv);

    bf_rt_target_t dev_tgt = {
        .dev_id = 0,
        .pipe_id = 0xffff,
        .direction = BF_DEV_DIR_ALL,
        .prsr_id = 0
    };

    bfrt::BfRtInfo const* bf_rt_info(nullptr);
    auto &dev_mgr(bfrt::BfRtDevMgr::getInstance());
    my_assert(dev_mgr.bfRtInfoGet(dev_tgt.dev_id, "upf", std::addressof(bf_rt_info)));

    println("Calibrating timer");
    g_tscns.init(1000000000);


    // create global resources
    println("Creating shared pull priorty context");
    auto shared_pull_priority_ctx(std::make_unique<SharedPullPriorityContext>());
    println("Creating ingress table group");
    auto accounting_table_group_ig(std::make_unique<TableGroup>(bf_rt_info, 2, TableGroupPlacement::Ingress, shared_pull_priority_ctx.get()));
    println("Creating egress table group");
    auto accounting_table_group_eg(std::make_unique<TableGroup>(bf_rt_info, 2, TableGroupPlacement::Egress, shared_pull_priority_ctx.get()));
    println("Creating performance objects");
    auto perf_objs(std::make_unique<GlobalPerformanceObjects>());
    println("Creating PDR tables");
    auto pdr_tables(std::make_unique<PDRTables>(bf_rt_info));
    println("Creating interface");
    auto interface(std::make_unique<UnixSocketInterface>("/tmp/upf-rust.sock", "/tmp/upf-cpp.sock"));

    auto running_ptr(std::make_unique<bool>(true));
    u64 log_freq_us(5000000);

    // create the 3 threads:
    println("Starting ingress pull worker");
    std::thread thread_pull_ig(&pull_worker, accounting_table_group_ig.get(), interface.get(), dev_tgt, perf_objs.get(), true, running_ptr.get(), log_freq_us);
    println("Starting egress pull worker");
    std::thread thread_pull_eg(&pull_worker, accounting_table_group_eg.get(), interface.get(), dev_tgt, perf_objs.get(), false, running_ptr.get(), log_freq_us);
    println("Starting update worker");
    std::thread thread_update(&update_worker, interface.get(), dev_tgt, pdr_tables.get(), accounting_table_group_ig.get(), accounting_table_group_eg.get(), perf_objs.get(), running_ptr.get(), log_freq_us);

    run_cli_or_cleanup();

    thread_pull_ig.join();
    thread_pull_eg.join();
    thread_update.join();

    return 0;
}
