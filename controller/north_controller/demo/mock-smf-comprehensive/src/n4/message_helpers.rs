use crate::context;
use messages::{session_establishment, session_modification};

/// Create a PDR and FAR for detecting uplink packet and send them to N6 interface
pub fn build_CNTunnelEstablishmentForDefaultSDF(
    ctx: &mut crate::sm_context::SmContext,
) -> session_establishment::SessionEstablishmentRequest {
    let default_far_id_uplink = ctx.pfcp_sessions.next_far_id(); // unique within a PFCP session
    let default_pdr_id_uplink = ctx.pfcp_sessions.next_pdr_id();
    let node_ip = context::GLOBAL_CONTEXT.Parameters.read().unwrap().NodeId;
    let extra_ip_len = match node_ip {
        std::net::IpAddr::V4(_) => 0,
        std::net::IpAddr::V6(_) => 12,
    };
    let mut request = messages::session_establishment::SessionEstablishmentRequest {
		header: messages::header::Header {
			version: 1,
			mp: false,
			s: false,
			msg_t: 50, // Session Establishment	Request
			msg_len: 9 + extra_ip_len + 17 + extra_ip_len + 5 + 63 + 23 + 4 + 4,
			seid: None,
			sequence: 0,
			priority: None
		},
		node_id: messages::elements::node_id::NodeID {
			ie_type: messages::elements::ie_type::NODE_ID,
			ie_len: 5 + extra_ip_len,
			node_id_type: messages::elements::node_id::node_id_type::IPV4,
			node_id: match node_ip {
				std::net::IpAddr::V4(v4) => v4.octets().to_vec(),
				std::net::IpAddr::V6(v6) => v6.octets().to_vec()
			},
		},
		cp_f_seid: messages::elements::f_seid::FSEID {
			ie_type: messages::elements::ie_type::F_SEID,
			ie_len: 13 + extra_ip_len,
			mask: 0b10,
			seid: ctx.local_seid.to_be_bytes().to_vec(),
			ipv4_addr: match node_ip {
				std::net::IpAddr::V4(v4) => Some(v4.octets().to_vec()),
				std::net::IpAddr::V6(_) => None
			},
			ipv6_addr: match node_ip {
				std::net::IpAddr::V4(_) => None,
				std::net::IpAddr::V6(v6) => Some(v6.octets().to_vec())
			},
		},
		pdn_type: messages::elements::pdn_type::PDNType { // 29244-g50.pdf:248
			ie_type: messages::elements::ie_type::PDN_TYPE,
			ie_len: 1,
			pdn_type: 1 // IPv4
		},
		create_pdrs: vec![messages::elements::create_pdr::CreatePDR { // Uplink PDR
			ie_type: messages::elements::ie_type::CREATE_PDR,
			ie_len: 59,
			pdi: messages::elements::pdi::PDI {
				ie_type: messages::elements::ie_type::PDI,
				ie_len: 28,
				source_interface: messages::elements::source_interface::SourceInterface { // 29244-g50.pdf:204
					ie_type: messages::elements::ie_type::SOURCE_INTERFACE,
					ie_len: 1,
					interface: 0 // Access
				},
				f_teid: Some(messages::elements::f_teid::FTEID {
					ie_type: messages::elements::ie_type::F_TEID,
					ie_len: 1 + 4 + 4 + 1,
					mask: 0b1101,
					teid: Some(0u32.to_be_bytes().to_vec()),      // empty TEID, let UPF assign one
					ipv4_addr: Some(0u32.to_be_bytes().to_vec()), // empty IPv4, useless here because the tunnel endpoint is always going to be UPF's IP
					ipv6_addr: None,
					choose_id: Some(0)
				}),
				network_instance: None,
				ue_ip_address: Some(messages::elements::ue_ip_address::UEIPAddress::from_ipv4(*ctx.ue_ip_ipv4.as_ref().unwrap())),
				sdf_filter: None,
				qfi: None,
				_3gpp_interface_type: None
			},
			precedence: messages::elements::precedence::Precedence {
				ie_type: messages::elements::ie_type::PRECEDENCE,
				ie_len: 4,
				precedence: 65534u32.to_be_bytes().to_vec() // lowest precedence for default PDR
			},
			pdr_id: messages::elements::pdr_id::PDRID {
				ie_type: messages::elements::ie_type::PDR_ID,
				ie_len: 2,
				rule_id: default_pdr_id_uplink.to_be_bytes().to_vec()
			},
			outer_header_removal: Some(messages::elements::outer_header_removal::OuterHeaderRemoval::new(messages::elements::outer_header_removal::OuterHeaderRemovalDescription::GTP_U_UDP_IP)),
			far_id: Some(messages::elements::far_id::FARID {
				ie_type: messages::elements::ie_type::FAR_ID,
				ie_len: 4,
				far_id: default_far_id_uplink.to_be_bytes().to_vec()
			}),
			urr_ids: None,
			qer_ids: None
		}],
		create_fars: vec![messages::elements::create_far::CreateFAR { // Uplink FAR
			ie_type: messages::elements::ie_type::CREATE_FAR,
			ie_len: 4 + 4 + 2 + 4 + 5 + 4,
			far_id: messages::elements::far_id::FARID {
				ie_type: messages::elements::ie_type::FAR_ID,
				ie_len: 4,
				far_id: default_far_id_uplink.to_be_bytes().to_vec()
			},
			apply_action: messages::elements::apply_action::ApplyAction { // 29244-g50.pdf:218
				ie_type: messages::elements::ie_type::APPLY_ACTION,
				ie_len: 2,
				action: 0b10, // FORW
				mask: 0
			},
			forwarding_parameters: Some(messages::elements::forwarding_parameters::ForwardingParameters {
				ie_type: messages::elements::ie_type::FORWARDING_PARAMETERS,
				ie_len: 5,
				destination_interface: messages::elements::destination_interface::DestinationInterface { // 29244-g50.pdf:215
					ie_type: messages::elements::ie_type::DESTINATION_INTERFACE,
					ie_len: 1,
					interface: 1 // Core(Uplink)
				},
				network_instance: None,
				redirect_information: None,
				outer_head_creation: None,
				transport_level_marking: None,
				forwarding_policy: None,
				header_enrichment: None,
				linked_traffic_endpoint_id: None,
				proxying: None,
				destination_interface_type: None,
				data_network_access_identifier: None
			})
		}],
		create_qers: vec![],
		create_urrs: vec![]
    };
    request
}

/// Update the existing sm context to include downlink PDR and FAR for packet from N6 interface to AN
pub fn build_ANTunnelUpdateForDefaultSDF(
    ctx: &mut crate::sm_context::SmContext,
) -> session_modification::SessionModificationRequest {
    let default_far_id_downlink = ctx.pfcp_sessions.next_far_id(); // unique within a PFCP session
    let default_pdr_id_downlink = ctx.pfcp_sessions.next_pdr_id();
    let node_ip = context::GLOBAL_CONTEXT.Parameters.read().unwrap().NodeId;
    let extra_ip_len = match node_ip {
        std::net::IpAddr::V4(_) => 0,
        std::net::IpAddr::V6(_) => 12,
    };
    let mut request = session_modification::SessionModificationRequest {
        header: messages::header::Header {
            version: 1,
            mp: false,
            s: false,
            msg_t: 52, // Session Modification	Request
            msg_len: 9 + extra_ip_len + 17 + extra_ip_len + 5 + 44 + 39 + 4 + 4,
            seid: None,
            sequence: 0,
            priority: None,
        },
        node_id: messages::elements::node_id::NodeID {
            ie_type: messages::elements::ie_type::NODE_ID,
            ie_len: 5 + extra_ip_len,
            node_id_type: messages::elements::node_id::node_id_type::IPV4,
            node_id: match node_ip {
                std::net::IpAddr::V4(v4) => v4.octets().to_vec(),
                std::net::IpAddr::V6(v6) => v6.octets().to_vec(),
            },
        },
        cp_f_seid: messages::elements::f_seid::FSEID {
            ie_type: messages::elements::ie_type::F_SEID,
            ie_len: 13 + extra_ip_len,
            mask: 0b10,
            seid: ctx.local_seid.to_be_bytes().to_vec(),
            ipv4_addr: match node_ip {
                std::net::IpAddr::V4(v4) => Some(v4.octets().to_vec()),
                std::net::IpAddr::V6(_) => None,
            },
            ipv6_addr: match node_ip {
                std::net::IpAddr::V4(_) => None,
                std::net::IpAddr::V6(v6) => Some(v6.octets().to_vec()),
            },
        },
        create_pdrs: vec![messages::elements::create_pdr::CreatePDR {
            // Downlink PDR
            ie_type: messages::elements::ie_type::CREATE_PDR,
            ie_len: 40,
            pdi: messages::elements::pdi::PDI {
                ie_type: messages::elements::ie_type::PDI,
                ie_len: 14,
                source_interface: messages::elements::source_interface::SourceInterface {
                    // 29244-g50.pdf:204
                    ie_type: messages::elements::ie_type::SOURCE_INTERFACE,
                    ie_len: 1,
                    interface: 1, // Core
                },
                f_teid: None,
                network_instance: None,
                ue_ip_address: Some(messages::elements::ue_ip_address::UEIPAddress::from_ipv4(
                    *ctx.ue_ip_ipv4.as_ref().unwrap(),
                )),
                sdf_filter: None,
                qfi: None,
                _3gpp_interface_type: None,
            },
            precedence: messages::elements::precedence::Precedence {
                ie_type: messages::elements::ie_type::PRECEDENCE,
                ie_len: 4,
                precedence: 65534u32.to_be_bytes().to_vec(), // lowest precedence for default PDR
            },
            pdr_id: messages::elements::pdr_id::PDRID {
                ie_type: messages::elements::ie_type::PDR_ID,
                ie_len: 2,
                rule_id: default_pdr_id_downlink.to_be_bytes().to_vec(),
            },
            outer_header_removal: None,
            far_id: Some(messages::elements::far_id::FARID {
                ie_type: messages::elements::ie_type::FAR_ID,
                ie_len: 4,
                far_id: default_far_id_downlink.to_be_bytes().to_vec(),
            }),
            urr_ids: None,
            qer_ids: None,
        }],
        create_fars: vec![messages::elements::create_far::CreateFAR {
            // Downlink FAR
            ie_type: messages::elements::ie_type::CREATE_FAR,
            ie_len: 4 + 4 + 2 + 4 + 21 + 4,
            far_id: messages::elements::far_id::FARID {
                ie_type: messages::elements::ie_type::FAR_ID,
                ie_len: 4,
                far_id: default_far_id_downlink.to_be_bytes().to_vec(),
            },
            apply_action: messages::elements::apply_action::ApplyAction {
                // 29244-g50.pdf:218
                ie_type: messages::elements::ie_type::APPLY_ACTION,
                ie_len: 2,
                action: 0b10, // FORW
                mask: 0,
            },
            forwarding_parameters: Some(
                messages::elements::forwarding_parameters::ForwardingParameters {
                    ie_type: messages::elements::ie_type::FORWARDING_PARAMETERS,
                    ie_len: 21,
                    destination_interface:
                        messages::elements::destination_interface::DestinationInterface {
                            // 29244-g50.pdf:215
                            ie_type: messages::elements::ie_type::DESTINATION_INTERFACE,
                            ie_len: 1,
                            interface: 0, // Access(Downlink)
                        },
                    network_instance: None,
                    redirect_information: None,
                    outer_head_creation: Some(
                        messages::elements::outer_header_creation::OuterHeaderCreation {
                            ie_type: messages::elements::ie_type::OUTER_HEADER_CREATION,
                            ie_len: 12,
                            description: vec![0b1, 0], // GTP/UDP/IPv4
                            teid: Some(ctx.tunnel.an_teid.unwrap().to_be_bytes().to_vec()),
                            ipv6_addr: None,
                            ipv4_addr: Some(match ctx.tunnel.an_ip.unwrap() {
                                std::net::IpAddr::V4(ip) => ip.octets().to_vec(),
                                std::net::IpAddr::V6(ip) => unimplemented!(), // TODO: support ipv6
                            }), // NG-RAN IP
                            port_number: Some(10001u16.to_be_bytes().to_vec()), // 127.0.0.1:10001 TODO: 127.0.0.1:8805 but taken
                            c_tag: None,
                            s_tag: None,
                        },
                    ),
                    transport_level_marking: None,
                    forwarding_policy: None,
                    header_enrichment: None,
                    linked_traffic_endpoint_id: None,
                    proxying: None,
                    destination_interface_type: None,
                    data_network_access_identifier: None,
                },
            ),
        }],
        create_qers: vec![],
        create_urrs: vec![],
    };
    request
}
