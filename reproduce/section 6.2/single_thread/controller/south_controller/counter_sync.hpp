
#ifndef _COUNTER_SYNC_H_
#define _COUNTER_SYNC_H_

#include "common.hpp"

struct synchriozed_counter_sync_cookie {
    std::uint64_t sync_counter;
    std::uint64_t completion_counter;
    pthread_mutex_t lock;
    pthread_cond_t cond;
    std::chrono::high_resolution_clock::time_point t0;
    synchriozed_counter_sync_cookie() : sync_counter(0), completion_counter(0) {

    }
    void on_sync() {
        sync_counter += 1;
        //print("synchriozed_counter_sync_cookie::on_sync\n");
        t0 = std::chrono::high_resolution_clock::now();
        lock = PTHREAD_MUTEX_INITIALIZER;
        cond = PTHREAD_COND_INITIALIZER;
    }
    void on_complete() noexcept {
        //print("synchriozed_counter_sync_cookie::on_complete wait for lock\n");
        pthread_mutex_lock(std::addressof(lock));
        //print("synchriozed_counter_sync_cookie::on_complete lock acquired\n");
        completion_counter += 1;
        if (completion_counter > sync_counter) {
            completion_counter = sync_counter;
        } else {
            //print("synchriozed_counter_sync_cookie::complete\n");
            std::chrono::duration<float> diff(std::chrono::high_resolution_clock::now() - t0);
            //print("Time counter sync:", diff.count(), "\n");
        }
        pthread_cond_signal(std::addressof(cond));
        pthread_mutex_unlock(std::addressof(lock));
    }
    void wait_for_completion() noexcept {
        pthread_mutex_lock(std::addressof(lock));
        while (completion_counter < sync_counter)
            pthread_cond_wait(std::addressof(cond), std::addressof(lock));
        pthread_mutex_unlock(std::addressof(lock));
    }
};

void cb_bfrt_table_sync_synchriozed_counter_sync_cookie(const bf_rt_target_t &dev_tgt, void *cookie);
void do_sync_counters(bfrt::BfRtTable const* table, bf_rt_target_t dev_tgt, auto session);

#endif
