
#include "common.hpp"
#include "pull_worker.hpp"
#include "update_worker.hpp"
#include "interface.hpp"
#include "accounting_table_group.hpp"
#include "pdr_tables.hpp"


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
    println("Creating egress table");
    auto accounting_table_eg(std::make_unique<AccountingTable>(bf_rt_info, "pipe.Egress.accounting.accounting_exact1", "ma_id", "Egress.accounting.inc_counter1", false));
    println("Creating ingress table");
    auto accounting_table_ig(std::make_unique<AccountingTable>(bf_rt_info, "pipe.Ingress.accounting.accounting_exact1", "ma_id", "Ingress.accounting.inc_counter1", true));
    println("Creating performance objects");
    auto perf_objs(std::make_unique<GlobalPerformanceObjects>());
    println("Creating PDR tables");
    auto pdr_tables(std::make_unique<PDRTables>(bf_rt_info));
    println("Creating interface");
    auto interface(std::make_unique<UnixSocketInterface>("/tmp/upf-rust.sock", "/tmp/upf-cpp.sock"));

    auto running_ptr(std::make_unique<bool>(true));
    u64 log_freq_us(5000000);

    // create the 3 threads:
    // println("Starting egress pull worker");
    // std::thread thread_pull_eg(&pull_worker, accounting_table_group_eg.get(), interface.get(), dev_tgt, perf_objs.get(), false, running_ptr.get(), log_freq_us);
    println("Starting update worker");
    std::thread thread_update(&update_worker, interface.get(), dev_tgt, pdr_tables.get(), accounting_table_eg.get(), accounting_table_ig.get(), perf_objs.get(), running_ptr.get(), log_freq_us);

    run_cli_or_cleanup();

    //thread_pull_eg.join();
    thread_update.join();

    return 0;
}
