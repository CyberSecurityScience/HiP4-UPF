
#include "accounting_tables.hpp"

#include "models/response_generated.h"
#include <cstring>

void send_counter_values(
    flatbuffers::FlatBufferBuilder& builder,
    std::vector<flatbuffers::Offset<UPFDriver::Response::CounterValue>>& values_fb,
    UnixDomainSocket *raw_if,
    bool is_ingress,
    u64 ts
) {
    auto counter_values_fb(builder.CreateVector(values_fb));
    auto counter_value_reports(UPFDriver::Response::CreateCounterValues(builder, counter_values_fb, is_ingress, ts));
    UPFDriver::Response::ResponseBuilder builder2(builder);
    builder2.add_response(counter_value_reports.Union());
    builder2.add_response_type(UPFDriver::Response::ResponseUnion::ResponseUnion_CounterValues);
    auto resp(builder2.Finish());
    builder.Finish(resp);
    //println("[send_counter_values] send ", values_fb.size(), " entries size ", builder.GetSize(), " bytes");
    raw_if->send(std::bit_cast<std::byte const*>(builder.GetBufferPointer()), builder.GetSize());
}


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
) {
    bf_rt_id_t table_id;
    table->tableIdGet(&table_id);
    uint32_t entry_count = 0;
    table->tableUsageGet(*session, dev_tgt, bfrt::BfRtTable::BfRtTableGetFlag::GET_FROM_SW, std::addressof(entry_count));
    if (entry_count == 0 || !ctx->has_batch()) {
        return 0;
    }

    std::unique_ptr<bfrt::BfRtTableKey> key_ptr;
    std::unique_ptr<bfrt::BfRtTableData> data_ptr;
    table->keyAllocate(std::addressof(key_ptr));
    table->dataAllocate(std::addressof(data_ptr));

    usize to_sent_count(0);
    usize n(0);
    usize bs = 5000;
    auto flag = bfrt::BfRtTable::BfRtTableGetFlag::GET_FROM_SW;
    std::vector<flatbuffers::Offset<UPFDriver::Response::CounterValue>> values_fb;
    values_fb.reserve(bs);


    flatbuffers::FlatBufferBuilder builder(2 * 1024 * 1024);
    auto batch(ctx->get_batch());

    for (auto const& [ma_id, handle]: batch) {
        // table->keyReset(key_ptr.get());
        // table->dataReset(data_ptr.get());
        // key_ptr->setValue(id_key_ma_id, u64(key));
        table->tableEntryGet(*session, dev_tgt, 0, handle, key_ptr.get(), data_ptr.get());

        UPFDriver::Response::CounterValueT v2;
        my_assert(data_ptr->getValue(id_data_counter_bytes, std::addressof(v2.bytes)));
        my_assert(data_ptr->getValue(id_data_counter_pkts, std::addressof(v2.pkts)));
        //my_assert(key_ptr->getValue(id_key_ma_id, std::addressof(ma_id)));
        v2.ma_id = ma_id;
        values_fb.push_back(UPFDriver::Response::CounterValue::Pack(builder, &v2));
        // if (!is_ingress) {
        //     println("[debug] [maid=",ma_id,"] vol=", v2.bytes);
        // }

        ++to_sent_count;
        ++n;

        if (to_sent_count >= bs) {
            send_counter_values(builder, values_fb, raw_if, is_ingress, ts);
            to_sent_count = 0;
            builder.Clear();
            values_fb.clear();
        }
    }

    if (to_sent_count != 0) {
        send_counter_values(builder, values_fb, raw_if, is_ingress, ts);
    }
    ctx->advance_to_next_batch();

    return n;
}

