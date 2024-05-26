#pragma once

#ifndef _PDR_TABLES_H_
#define _PDR_TABLLES_H_

#include "common.hpp"
#include "models/requests_generated.h"

struct DL_N6_SimpleInsert {
    u32 key_ipv4    ;
    u32 data_ma_id  ;
    u32 data_teid   ;
    u16 data_qer_id ;
    u64 vol_thres   ;
};
struct DL_N6_SimpleUpdate {
    u32 key_ipv4       ;
    u32 data_ma_id     ;
    u32 data_old_ma_id ;
    u32 data_teid      ;
    u16 data_qer_id    ;
    u64 vol_thres      ;
};
struct DL_N6_SimpleRemove {
    u32 key_ipv4       ;
    u32 data_old_ma_id ;
};
struct UL_N6_SimpleInsert {
    u32 key_teid    ;
    u8  key_qfi     ;
    u32 data_ma_id  ;
    u16 data_qer_id ;
    u64 vol_thres   ;
};
struct UL_N6_SimpleUpdate {
    u32 key_teid       ;
    u8  key_qfi        ;
    u32 data_ma_id     ;
    u32 data_old_ma_id ;
    u16 data_qer_id    ;
    u64 vol_thres      ;
};
struct UL_N6_SimpleRemove {
    u32 key_teid       ;
    u8  key_qfi        ;
    u32 data_old_ma_id ;
};
enum class RequestTransactionType {
    DL_N6_SimpleInsertType,
    DL_N6_SimpleUpdateType,
    DL_N6_SimpleRemoveType,
    UL_N6_SimpleInsertType,
    UL_N6_SimpleUpdateType,
    UL_N6_SimpleRemoveType,
};
struct CopyablePDROperation {
    RequestTransactionType type;
    union {
        DL_N6_SimpleInsert DL_N6_SimpleInsertOp;
        DL_N6_SimpleUpdate DL_N6_SimpleUpdateOp;
        DL_N6_SimpleRemove DL_N6_SimpleRemoveOp;
        UL_N6_SimpleInsert UL_N6_SimpleInsertOp;
        UL_N6_SimpleUpdate UL_N6_SimpleUpdateOp;
        UL_N6_SimpleRemove UL_N6_SimpleRemoveOp;
    };
};
struct CopyableTransaction {
    u32 transaction_id;
    u64 deadline;
    std::vector<CopyablePDROperation> ops;
    CopyableTransaction(UPFDriver::Requests::Transaction const *const src) {
        transaction_id = src->transaction_id();
        deadline = src->deadline();
        auto size(src->ops()->size());
        ops.reserve(size);
        for (usize i(0) ; i != size; ++i) {
            auto item(src->ops()->Get(i));
            auto t(item->request_type());
            CopyablePDROperation op;
            switch (t) {
                case UPFDriver::Requests::RequestUnion_DL_N6_SimpleInsertRequest:
                {
                    op.type = RequestTransactionType::DL_N6_SimpleInsertType;
                    op.DL_N6_SimpleInsertOp.key_ipv4 = item->request_as_DL_N6_SimpleInsertRequest()->key_ipv4();
                    op.DL_N6_SimpleInsertOp.data_ma_id = item->request_as_DL_N6_SimpleInsertRequest()->data_ma_id();
                    op.DL_N6_SimpleInsertOp.data_qer_id = item->request_as_DL_N6_SimpleInsertRequest()->data_qer_id();
                    op.DL_N6_SimpleInsertOp.data_teid = item->request_as_DL_N6_SimpleInsertRequest()->data_teid();
                    op.DL_N6_SimpleInsertOp.vol_thres = item->request_as_DL_N6_SimpleInsertRequest()->vol_thres();
                }
                break;
                case UPFDriver::Requests::RequestUnion_DL_N6_SimpleUpdateRequest:
                {
                    op.type = RequestTransactionType::DL_N6_SimpleUpdateType;
                    op.DL_N6_SimpleUpdateOp.key_ipv4 = item->request_as_DL_N6_SimpleUpdateRequest()->key_ipv4();
                    op.DL_N6_SimpleUpdateOp.data_ma_id = item->request_as_DL_N6_SimpleUpdateRequest()->data_ma_id();
                    op.DL_N6_SimpleUpdateOp.data_old_ma_id = item->request_as_DL_N6_SimpleUpdateRequest()->data_old_ma_id();
                    op.DL_N6_SimpleUpdateOp.data_qer_id = item->request_as_DL_N6_SimpleUpdateRequest()->data_qer_id();
                    op.DL_N6_SimpleUpdateOp.data_teid = item->request_as_DL_N6_SimpleUpdateRequest()->data_teid();
                    op.DL_N6_SimpleUpdateOp.vol_thres = item->request_as_DL_N6_SimpleUpdateRequest()->vol_thres();
                }
                break;
                case UPFDriver::Requests::RequestUnion_DL_N6_SimpleRemoveRequest:
                {
                    op.type = RequestTransactionType::DL_N6_SimpleRemoveType;
                    op.DL_N6_SimpleRemoveOp.key_ipv4 = item->request_as_DL_N6_SimpleRemoveRequest()->key_ipv4();
                    op.DL_N6_SimpleRemoveOp.data_old_ma_id = item->request_as_DL_N6_SimpleRemoveRequest()->data_old_ma_id();
                }
                break;
                case UPFDriver::Requests::RequestUnion_UL_N6_SimpleInsertRequest:
                {
                    op.type = RequestTransactionType::UL_N6_SimpleInsertType;
                    op.UL_N6_SimpleInsertOp.key_qfi = item->request_as_UL_N6_SimpleInsertRequest()->key_qfi();
                    op.UL_N6_SimpleInsertOp.key_teid = item->request_as_UL_N6_SimpleInsertRequest()->key_teid();
                    op.UL_N6_SimpleInsertOp.data_ma_id = item->request_as_UL_N6_SimpleInsertRequest()->data_ma_id();
                    op.UL_N6_SimpleInsertOp.data_qer_id = item->request_as_UL_N6_SimpleInsertRequest()->data_qer_id();
                    op.UL_N6_SimpleInsertOp.vol_thres = item->request_as_UL_N6_SimpleInsertRequest()->vol_thres();
                }
                break;
                case UPFDriver::Requests::RequestUnion_UL_N6_SimpleUpdateRequest:
                {
                    op.type = RequestTransactionType::UL_N6_SimpleUpdateType;
                    op.UL_N6_SimpleUpdateOp.key_qfi = item->request_as_UL_N6_SimpleUpdateRequest()->key_qfi();
                    op.UL_N6_SimpleUpdateOp.key_teid = item->request_as_UL_N6_SimpleUpdateRequest()->key_teid();
                    op.UL_N6_SimpleUpdateOp.data_ma_id = item->request_as_UL_N6_SimpleUpdateRequest()->data_ma_id();
                    op.UL_N6_SimpleUpdateOp.data_old_ma_id = item->request_as_UL_N6_SimpleUpdateRequest()->data_old_ma_id();
                    op.UL_N6_SimpleUpdateOp.data_qer_id = item->request_as_UL_N6_SimpleUpdateRequest()->data_qer_id();
                    op.UL_N6_SimpleUpdateOp.vol_thres = item->request_as_UL_N6_SimpleUpdateRequest()->vol_thres();
                }
                break;
                case UPFDriver::Requests::RequestUnion_UL_N6_SimpleRemoveRequest:
                {
                    op.type = RequestTransactionType::UL_N6_SimpleRemoveType;
                    op.UL_N6_SimpleRemoveOp.key_qfi = item->request_as_UL_N6_SimpleRemoveRequest()->key_qfi();
                    op.UL_N6_SimpleRemoveOp.key_teid = item->request_as_UL_N6_SimpleRemoveRequest()->key_teid();
                    op.UL_N6_SimpleRemoveOp.data_old_ma_id = item->request_as_UL_N6_SimpleRemoveRequest()->data_old_ma_id();
                }
                break;
            }
            ops.push_back(op);
        }
    }
};

struct Table_UL_N6_Simple {
    bf_rt_id_t id_key_teid;
    bf_rt_id_t id_key_qfi;
    bf_rt_id_t id_data_ma_id;
    bf_rt_id_t id_data_qer_id;
    bf_rt_id_t id_action_set_ma_id;

    bfrt::BfRtTable const *table;

    Table_UL_N6_Simple(bfrt::BfRtInfo const *bf_rt_info): table(nullptr) {
        my_assert(bf_rt_info->bfrtTableFromNameGet("pipe.Ingress.pdr_all.ul_N6_simple_ipv4", std::addressof(table)));
        my_assert(table->keyFieldIdGet("hdr.gtpu.teid[23:0]", std::addressof(id_key_teid)));
        my_assert(table->keyFieldIdGet("hdr.gtpu_ext_psc.qfi", std::addressof(id_key_qfi)));
        my_assert(table->actionIdGet("Ingress.pdr_all.set_ma_id_ul_N6_simple_ipv4", std::addressof(id_action_set_ma_id)));
        my_assert(table->dataFieldIdGet("ma_id_v", id_action_set_ma_id, std::addressof(id_data_ma_id)));
        my_assert(table->dataFieldIdGet("qer_id", id_action_set_ma_id, std::addressof(id_data_qer_id)));
    }

    ~Table_UL_N6_Simple() noexcept {
        if (table) {
            table = nullptr;
        }
    }

    bf_status_t batch_update(
        std::shared_ptr<bfrt::BfRtSession> session,
        bf_rt_target_t dev_tgt,
        std::vector<CopyablePDROperation> const& ops,
        std::vector<std::tuple<std::uint32_t, u64>>& new_ma_ids,
        std::vector<std::uint32_t>& old_ma_ids
    ) {
        std::unique_ptr<bfrt::BfRtTableKey> key;
        my_assert(table->keyAllocate(std::addressof(key)));
        std::unique_ptr<bfrt::BfRtTableData> data;
        my_assert(table->dataAllocate(std::addressof(data)));

        u64 flags(0);
        for (auto const& op : ops) {
            switch (op.type) {
                case RequestTransactionType::UL_N6_SimpleInsertType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_teid, uint64_t(op.UL_N6_SimpleInsertOp.key_teid)));
                    my_assert(key->setValue(id_key_qfi, uint64_t(op.UL_N6_SimpleInsertOp.key_qfi)));
                    my_assert(data->setValue(id_data_ma_id, uint64_t(op.UL_N6_SimpleInsertOp.data_ma_id)));
                    my_assert(data->setValue(id_data_qer_id, uint64_t(op.UL_N6_SimpleInsertOp.data_qer_id)));
                    new_ma_ids.push_back({op.UL_N6_SimpleInsertOp.data_ma_id, op.UL_N6_SimpleInsertOp.vol_thres});
                    return_if_failed(table->tableEntryAdd(*session, dev_tgt, flags, *key, *data));
                }
                break;
                case RequestTransactionType::UL_N6_SimpleUpdateType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_teid, uint64_t(op.UL_N6_SimpleUpdateOp.key_teid)));
                    my_assert(key->setValue(id_key_qfi, uint64_t(op.UL_N6_SimpleUpdateOp.key_qfi)));
                    my_assert(data->setValue(id_data_ma_id, uint64_t(op.UL_N6_SimpleUpdateOp.data_ma_id)));
                    my_assert(data->setValue(id_data_qer_id, uint64_t(op.UL_N6_SimpleUpdateOp.data_old_ma_id)));
                    new_ma_ids.push_back({op.UL_N6_SimpleUpdateOp.data_ma_id, op.UL_N6_SimpleUpdateOp.vol_thres});
                    old_ma_ids.push_back(op.UL_N6_SimpleUpdateOp.data_old_ma_id);
                    return_if_failed(table->tableEntryMod(*session, dev_tgt, flags, *key, *data));
                }
                break;
                case RequestTransactionType::UL_N6_SimpleRemoveType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_teid, uint64_t(op.UL_N6_SimpleRemoveOp.key_teid)));
                    my_assert(key->setValue(id_key_qfi, uint64_t(op.UL_N6_SimpleRemoveOp.key_qfi)));
                    old_ma_ids.push_back(op.UL_N6_SimpleRemoveOp.data_old_ma_id);
                    return_if_failed(table->tableEntryDel(*session, dev_tgt, flags, *key));
                }
                break;
            }
        }
        return BF_SUCCESS;
    }
};


struct Table_DL_N6_Simple {
    bf_rt_id_t id_key_ipv4;
    bf_rt_id_t id_data_ma_id;
    bf_rt_id_t id_data_teid;
    bf_rt_id_t id_data_qer_id;
    bf_rt_id_t id_action_set_ma_id;

    bfrt::BfRtTable const *table;

    Table_DL_N6_Simple(bfrt::BfRtInfo const *bf_rt_info): table(nullptr) {
        my_assert(bf_rt_info->bfrtTableFromNameGet("pipe.Ingress.pdr_all.dl_N6_simple_ipv4", std::addressof(table)));
        my_assert(table->keyFieldIdGet("hdr.overlay_ipv4.dstAddr", std::addressof(id_key_ipv4)));
        my_assert(table->actionIdGet("Ingress.pdr_all.set_ma_id_and_tunnel_dl_N6_simple_ipv4", std::addressof(id_action_set_ma_id)));
        my_assert(table->dataFieldIdGet("ma_id_v", id_action_set_ma_id, std::addressof(id_data_ma_id)));
        my_assert(table->dataFieldIdGet("teid_v", id_action_set_ma_id, std::addressof(id_data_teid)));
        my_assert(table->dataFieldIdGet("qer_id", id_action_set_ma_id, std::addressof(id_data_qer_id)));
    }

    ~Table_DL_N6_Simple() noexcept {
        if (table) {
            table = nullptr;
        }
    }

    bf_status_t batch_update(
        std::shared_ptr<bfrt::BfRtSession> session,
        bf_rt_target_t dev_tgt,
        std::vector<CopyablePDROperation> const& ops,
        std::vector<std::tuple<std::uint32_t, u64>>& new_ma_ids,
        std::vector<std::uint32_t>& old_ma_ids
    ) {
        std::unique_ptr<bfrt::BfRtTableKey> key;
        my_assert(table->keyAllocate(std::addressof(key)));
        std::unique_ptr<bfrt::BfRtTableData> data;
        my_assert(table->dataAllocate(std::addressof(data)));

        u64 flags(0);
        for (auto const& op : ops) {
            switch (op.type) {
                case RequestTransactionType::DL_N6_SimpleInsertType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_ipv4, uint64_t(op.DL_N6_SimpleInsertOp.key_ipv4)));
                    my_assert(data->setValue(id_data_ma_id, uint64_t(op.DL_N6_SimpleInsertOp.data_ma_id)));
                    my_assert(data->setValue(id_data_teid, uint64_t(op.DL_N6_SimpleInsertOp.data_teid)));
                    my_assert(data->setValue(id_data_qer_id, uint64_t(op.DL_N6_SimpleInsertOp.data_qer_id)));
                    new_ma_ids.push_back({op.DL_N6_SimpleInsertOp.data_ma_id, op.DL_N6_SimpleInsertOp.vol_thres});
                    return_if_failed(table->tableEntryAdd(*session, dev_tgt, flags, *key, *data));
                }
                break;
                case RequestTransactionType::DL_N6_SimpleUpdateType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_ipv4, uint64_t(op.DL_N6_SimpleUpdateOp.key_ipv4)));
                    my_assert(data->setValue(id_data_ma_id, uint64_t(op.DL_N6_SimpleUpdateOp.data_ma_id)));
                    my_assert(data->setValue(id_data_teid, uint64_t(op.DL_N6_SimpleUpdateOp.data_teid)));
                    my_assert(data->setValue(id_data_qer_id, uint64_t(op.DL_N6_SimpleUpdateOp.data_qer_id)));
                    new_ma_ids.push_back({op.DL_N6_SimpleUpdateOp.data_ma_id, op.DL_N6_SimpleUpdateOp.vol_thres});
                    old_ma_ids.push_back(op.DL_N6_SimpleUpdateOp.data_old_ma_id);
                    return_if_failed(table->tableEntryMod(*session, dev_tgt, flags, *key, *data));
                }
                break;
                case RequestTransactionType::DL_N6_SimpleRemoveType:
                {
                    my_assert(table->keyReset(key.get()));
                    my_assert(table->dataReset(id_action_set_ma_id, data.get()));
                    my_assert(key->setValue(id_key_ipv4, uint64_t(op.DL_N6_SimpleRemoveOp.key_ipv4)));
                    old_ma_ids.push_back(op.DL_N6_SimpleRemoveOp.data_old_ma_id);
                    return_if_failed(table->tableEntryDel(*session, dev_tgt, flags, *key));
                }
                break;
            }
        }
        return BF_SUCCESS;
    }
};

struct PDRTables {
    Table_UL_N6_Simple ul_n6_simple;
    Table_DL_N6_Simple dl_n6_simple;

    PDRTables(bfrt::BfRtInfo const *bf_rt_info): ul_n6_simple(bf_rt_info), dl_n6_simple(bf_rt_info) {

    }

    bf_status_t batch_update(
        std::shared_ptr<bfrt::BfRtSession> session,
        bf_rt_target_t dev_tgt,
        std::vector<CopyablePDROperation> const& ops,
        std::vector<std::tuple<std::uint32_t, u64>>& new_ma_ids,
        std::vector<std::uint32_t>& old_ma_ids
    ) {
        return_if_failed(ul_n6_simple.batch_update(session, dev_tgt, ops, new_ma_ids, old_ma_ids));
        return_if_failed(dl_n6_simple.batch_update(session, dev_tgt, ops, new_ma_ids, old_ma_ids));
        return BF_SUCCESS;
    }
};

#endif
