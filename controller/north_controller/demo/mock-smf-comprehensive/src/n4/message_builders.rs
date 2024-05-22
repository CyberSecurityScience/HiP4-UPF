use super::message_helpers;
use crate::context;

pub fn build_default_pdu_session() -> Vec<u8> {
    Vec::new()
}

pub fn build_SessionEstablishmentRequest(ctx: &mut crate::sm_context::SmContext) -> Vec<u8> {
    message_helpers::build_CNTunnelEstablishmentForDefaultSDF(ctx).pack()
}

pub fn build_SessionModificationRequest(ctx: &mut crate::sm_context::SmContext) -> Vec<u8> {
    message_helpers::build_ANTunnelUpdateForDefaultSDF(ctx).pack()
}
