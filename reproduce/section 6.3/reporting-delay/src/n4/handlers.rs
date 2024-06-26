#![allow(unused)]

use async_trait::async_trait;
use libpfcp::{
    handlers::{NodeRequestHandlers, SessionRequestHandlers},
    messages::{HeartbeatResponse, PFCPSessionReportResponse, PFCPSessionReportRequest},
    models::{RecoveryTimeStamp, Cause},
    PFCPModel,
};
use log::{info, error};
use std::{net::IpAddr, time::{SystemTime, UNIX_EPOCH}};

use crate::{N4_REPORT_TIME, Report};//, UE_TYPE, UEType};

#[derive(Debug, Clone)]
pub struct N4Handlers;

#[async_trait]
impl NodeRequestHandlers for N4Handlers {
    async fn handle_association_setup(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_heartbeat(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        let response = HeartbeatResponse {
            recovery_time_stamp: RecoveryTimeStamp::new(chrono::Utc::now()),
        };
        response.encode()
    }

    async fn handle_pfd_management(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_association_update(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_association_release(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_node_report(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_session_set_deletion(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }
}

#[async_trait]
impl SessionRequestHandlers for N4Handlers {
    async fn handle_session_establishment(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_session_modification(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_session_deletion(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        todo!()
    }

    async fn handle_session_report(
        &self,
        header: libpfcp::models::PFCPHeader,
        body: Vec<u8>,
        src_ip: IpAddr,
    ) -> Vec<u8> {
        // step 1: session lookup
        // step 2:
        // if is downlink data
        //     retrive FAR ID
        //
        //info!("handle_session_report");
        let mut response = PFCPSessionReportResponse {
            cause: libpfcp::models::Cause::RequestAccepted,
            offending_ie: None,
            update_bar: None,
            pfcpsr_rsp_flags: None,
            cp_f_seid: None,
            n4u_f_teid: None,
        };
        let request = match PFCPSessionReportRequest::decode(body.as_slice()) {
			Ok(r) => r,
			Err(e) => {
                error!("{:?}", e);
				response.cause = Cause::RequestRejectedUnspecified; // TODO: replace with correct error handling
				return response.encode();
			}
		};
        let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        for report in request.usage_report.iter() {
            let seid = header.seid.unwrap();
            //println!("volume_measurement: {:?}", report.volume_measurement);
            // info!("UE [SEID={}] done at {}, UL={} bytes", seid, time_now, report.volume_measurement.as_ref().unwrap().uplink_volume.unwrap());
            // write to file
			if N4_REPORT_TIME.contains_key(&seid) {//&& *UE_TYPE.get(&seid).unwrap() != UEType::Period {
                continue; // skip re-applied thresholds
            }
            // if N4_REPORT_TIME.contains_key(&seid) {
            //     // must be periodical
            //     let mut report = N4_REPORT_TIME.get_mut(&seid).unwrap();
            //     report.periodical_reports.push(time_now);
            // } else {
            //     match *UE_TYPE.get(&seid).unwrap() {
            //         UEType::Vol => {
            //             N4_REPORT_TIME.insert(seid, Report { vol_thres_reach_time_ms: time_now, total_bytes: report.volume_measurement.as_ref().unwrap().total_volume.unwrap(), time_thres_reach_time_ms: 0, total_time: 0, periodical_reports: vec![] });
            //         },
            //         UEType::Time => {
            //             N4_REPORT_TIME.insert(seid, Report { vol_thres_reach_time_ms: 0, total_bytes: 0, time_thres_reach_time_ms: time_now, total_time: report.duration_measurement.as_ref().unwrap().seconds as _, periodical_reports: vec![] });
            //         },
            //         UEType::Period => {
            //             N4_REPORT_TIME.insert(seid, Report { vol_thres_reach_time_ms: 0, total_bytes: 0, time_thres_reach_time_ms: 0, total_time: 0, periodical_reports: vec![time_now] });
            //         },
            //     }
            // }
            //N4_REPORT_TIME.insert(seid, Report { traffic_change_time_ms: time_now, vol_thres_reach_time_ms: time_now, total_bytes: report.volume_measurement.as_ref().unwrap().total_volume.unwrap() });
            //N4_REPORT_TIME.insert(seid, Report { traffic_change_time_ms: 0, vol_thres_reach_time_ms: time_now, total_bytes: report.duration_measurement.as_ref().unwrap().seconds as _ });


			N4_REPORT_TIME.insert(seid, Report { vol_thres_reach_time_ms: time_now, total_bytes: report.volume_measurement.as_ref().unwrap().total_volume.unwrap() });
        }
        response.encode()
    }
}
