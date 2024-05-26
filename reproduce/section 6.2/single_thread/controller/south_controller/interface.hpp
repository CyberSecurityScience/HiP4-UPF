#pragma once

#ifndef _INTERFACE_H_
#define _INTERFACE_H_

#include <optional>
#include <mutex>
#include <cstdint>

#include "common.hpp"
#include "domain_socket.hpp"
#include "models/requests_generated.h"
#include "models/response_generated.h"
#include "accounting_tables.hpp"
#include "pdr_tables.hpp"

constexpr std::size_t BUFFER_SIZE = 8 * 1024 * 1024;

struct UnixSocketInterface {
    UnixDomainSocket socket;
    std::byte *recv_buffer;
    ankerl::unordered_dense::map<u32, u8> ma_id_release_count;

    std::mutex interface_mutex;

    UnixSocketInterface(const std::string& send_endpoint, std::string const& recv_endpointh) : socket(send_endpoint, recv_endpointh), recv_buffer(nullptr), interface_mutex(), ma_id_release_count(4096) {
        recv_buffer = new std::byte[BUFFER_SIZE];
    }

    ~UnixSocketInterface() noexcept {
        if (recv_buffer) {
            delete[] recv_buffer ;
            recv_buffer = nullptr;
        }
        socket.~UnixDomainSocket();
    }

    void send_counter_values(std::vector<CounterResult> const& values, bool is_ingress, u64 ts) {
        usize bs = 10000;
        for (usize i(0); i < values.size(); i += bs) {
            flatbuffers::FlatBufferBuilder builder(2 * 1024 * 1024);
            std::vector<flatbuffers::Offset<UPFDriver::Response::CounterValue>> values_fb;
            values_fb.reserve(bs);
            for (usize j = 0; j < bs; ++j) {
                if (i + j >= values.size()) {
                    break;
                }
                UPFDriver::Response::CounterValueT v2;
                v2.bytes = values[i + j].bytes;
                v2.ma_id = values[i + j].ma_id;
                v2.pkts = values[i + j].pkts;
                values_fb.push_back(UPFDriver::Response::CounterValue::Pack(builder, &v2));
                //println("[send_counter_values] MAID ", v.ma_id);
            }
            auto counter_values_fb(builder.CreateVector(values_fb));
            auto counter_value_reports(UPFDriver::Response::CreateCounterValues(builder, counter_values_fb, is_ingress, ts));
            UPFDriver::Response::ResponseBuilder builder2(builder);
            builder2.add_response(counter_value_reports.Union());
            builder2.add_response_type(UPFDriver::Response::ResponseUnion::ResponseUnion_CounterValues);
            auto resp(builder2.Finish());
            builder.Finish(resp);
            //println("[send_counter_values] sending ", builder.GetSize()," bytes");
            socket.send(std::bit_cast<std::byte const*>(builder.GetBufferPointer()), builder.GetSize());
        }
    }

    void send_deleted_ma_ids(std::vector<std::uint32_t> const& ma_ids) {
        std::lock_guard<std::mutex> guard(interface_mutex);

        std::vector<u32> actually_reclaimed_ma_ids;
        actually_reclaimed_ma_ids.reserve(ma_id_release_count.size());

        for (u32 const ma_id : ma_ids) {
            ++ma_id_release_count[ma_id];
            //println(" -- [send_deleted_ma_ids] ma_id ", ma_id," count ", ma_id_release_count[ma_id]);
            if (ma_id_release_count[ma_id] >= 2) {
                ma_id_release_count.erase(ma_id);
                actually_reclaimed_ma_ids.push_back(ma_id);
            }
        }

        if (actually_reclaimed_ma_ids.size() == 0) {
            return;
        }
        // for (auto ma_id:actually_reclaimed_ma_ids) {
        //     println(" -- [send_deleted_ma_ids] reclaim ", ma_id);
        // }
        flatbuffers::FlatBufferBuilder builder(1024);
        auto actually_reclaimed_ma_ids_fb(builder.CreateVector(actually_reclaimed_ma_ids));
        auto table_op_complete(UPFDriver::Response::CreateMA_ID_Reclaimed(builder, actually_reclaimed_ma_ids_fb));
        UPFDriver::Response::ResponseBuilder builder2(builder);
        builder2.add_response(table_op_complete.Union());
        builder2.add_response_type(UPFDriver::Response::ResponseUnion::ResponseUnion_MA_ID_Reclaimed);
        auto resp(builder2.Finish());
        builder.Finish(resp);
        //println("[send_deleted_ma_ids] sending ", builder.GetSize()," bytes");
        socket.send(std::bit_cast<std::byte const*>(builder.GetBufferPointer()), builder.GetSize());
    }

    std::optional<std::vector<CopyableTransaction>> wait_for_requests(std::uint64_t timeout_us) {
        //println(" -- [wait_for_requests] timeout_us=", timeout_us);
        ssize_t ret_length(socket.receive(recv_buffer, BUFFER_SIZE, timeout_us));
        if (ret_length <= 0) {
            if (ret_length == -2) {
                // timed out
                return {};
            }
            // TODO: failed??
            perrln("[wait_for_requests] got error ", ret_length);
            return {};
        }
        //println("[wait_for_requests] successfully got ", ret_length," bytes");
        auto transaction(UPFDriver::Requests::GetTransaction(recv_buffer));
        //println("--  recv tid ", transaction->transaction_id());
        return {{CopyableTransaction(transaction)}};
    }

    void complete_requests(std::vector<CopyableTransaction> const& requests, int return_code) {
        flatbuffers::FlatBufferBuilder builder(1024);
        std::vector<u32> transaction_ids;
        transaction_ids.reserve(requests.size());
        for (auto const& req : requests) {
            transaction_ids.push_back(req.transaction_id);
            //println("--  send tid ", req.transaction_id);
        }
        auto transaction_ids_fb(builder.CreateVector(transaction_ids));
        auto table_op_complete(UPFDriver::Response::CreateTableOperationComplete(builder, return_code, transaction_ids_fb));
        UPFDriver::Response::ResponseBuilder builder2(builder);
        builder2.add_response(table_op_complete.Union());
        builder2.add_response_type(UPFDriver::Response::ResponseUnion::ResponseUnion_TableOperationComplete);
        auto resp(builder2.Finish());
        builder.Finish(resp);
        //println("[complete_requests] sending ", builder.GetSize()," bytes");
        socket.send(std::bit_cast<std::byte const*>(builder.GetBufferPointer()), builder.GetSize());
    }
};

#endif
