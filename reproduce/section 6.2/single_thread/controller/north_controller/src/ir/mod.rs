use std::{thread::JoinHandle, time::Duration};

use itertools::Itertools;
use libpfcp::{messages::{PFCPSessionReportRequest, DownlinkDataReport}, PFCPModel, models::ReportType};
use log::{warn, info, error};
use tokio::sync::mpsc::Receiver;

use crate::datapath::{FlattenedPacketPipeline, PacketPipeline, FlattenedPDI, Report, BackendReport};

async fn send_reports(reports: &Vec<BackendReport>) {
	let sessions = reports
		.iter()
		.group_by(|item| item.seid)
		.into_iter()
		.map(|(seid, group)| (seid, group.cloned().collect()))
		.collect::<Vec<(u64, Vec<BackendReport>)>>();
	info!("Sending {} usage reports across {} PFCP sessions", reports.len(), sessions.len());
	// println!("{:?}", reports);
	let global_pfcp_guard = crate::n4::GLOBAL_PFCP_CONTEXT.read().await;
	let all_pfcp = global_pfcp_guard.as_ref().unwrap();
	for (seid, reports) in sessions {
		if let Some(session_up_handle) = all_pfcp.PfcpSessions.get(&seid) {
			let mut session_up = session_up_handle.write().await;
			let mut report_request = PFCPSessionReportRequest {
				report_type: ReportType(0),
				downlink_data_report: None,
				usage_report: vec![],
				cause: None,
			};
			for r in reports.iter() {
				match &r.report {
					Report::BufferNoCP => todo!(),
					Report::DropNoCP => todo!(),
					Report::NoCP => todo!(),
					Report::DlArrival(pdr_id) => {
						report_request.downlink_data_report = Some(DownlinkDataReport {
							pdr_id: *pdr_id,
							downlink_data_service_information: None,
							data_status: None,
						});
						report_request.report_type.setDLDR(1);
					},
					Report::Error => todo!(),
					Report::UsageReports(rp) => {
						report_request.usage_report.append(&mut rp.clone());
						report_request.report_type.setUSAR(1);
					},
				}
			}
			report_request.usage_report[0].volume_measurement.as_mut().unwrap().downlink_packets = Some(seid);
			match session_up.SendSessionReportRequest(seid, report_request.encode()).await {
				Ok(_) => {

				},
				Err(e) => {
					error!("Timed out while sending SessionReportRequest for [SEID={}]", seid);
				},
			}
		}
	}
	//info!("Done sending {} usgae reports", reports.len());
}

fn create_usage_reporting_receiver_task(mut ur_receiver: Receiver<BackendReport>) -> JoinHandle<()> {
	// accumulate reports spanning 1/10 second
	let handle = std::thread::spawn(|| {
		let runtime = tokio::runtime::Builder::new_current_thread().thread_name("Tofino UPF usage reporting loop thread").enable_time().build().unwrap();
		runtime.block_on(async move {
			let start_timestamp = crate::context::BACKEND_TIME_REFERENCE.get().unwrap().clone();
			let mut cur_reports = vec![];
			loop {
				if cur_reports.len() != 0 {
					send_reports(&cur_reports.clone()).await;
				}
				cur_reports.clear();
				let epoch_timestamp_ms = start_timestamp.elapsed().as_millis() as u64;
				while (start_timestamp.elapsed().as_millis() as i64) - (epoch_timestamp_ms as i64) <= 100 {
					if let Ok(report) = ur_receiver.try_recv() {
						cur_reports.push(report);
					}
				}
			}
		});
	});
	handle
}

pub fn create_n4_usage_reporting_threads(ur_receiver: Receiver<BackendReport>) -> JoinHandle<()> {
	create_usage_reporting_receiver_task(ur_receiver)
}
