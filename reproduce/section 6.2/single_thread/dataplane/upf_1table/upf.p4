#include <core.p4>
#include <tna.p4>

#include "headers.p4"
#include "settings.p4"
#include "pdr.p4"

parser TofinoIngressParser(
			packet_in                                   pkt,
	inout   ig_metadata_t                               meta,
	out     ingress_intrinsic_metadata_t                ig_intr_md
) {
	state start {
		pkt.extract(ig_intr_md);
		transition select(ig_intr_md.resubmit_flag) {
			1 : parse_resubmit;
			0 : parse_port_metadata;
		}
	}

	state parse_resubmit {
		// Parse resubmitted packet here.
		pkt.advance(64);
		transition accept;
	}

	state parse_port_metadata {
		pkt.advance(64);  //tofino 1 port metadata size
		transition accept;
	}
}

parser IngressParser(
			packet_in                                   pkt,
	/* User */
	out     ig_header_t                                 hdr,
	out     ig_metadata_t                               meta,
	out     ingress_intrinsic_metadata_t                ig_intr_md
) {
	TofinoIngressParser() tofino_parser;

	state start {
		tofino_parser.apply(pkt, meta, ig_intr_md);
		transition init_metadata;
	}

	state init_metadata {
		meta.uplink = false;
		//meta.l4_protocol = 0;
		//meta.l4_src_port = 0;
		//meta.l4_dst_port = 0;	
		meta.individual_gbr_meter_color = MeterColor_t.GREEN;
		//meta.always_one = 1;
		//meta.ma_id = 24w0xffffff;
		//meta.gbr_meter_color = MeterColor_t.GREEN;
		transition parse_ethernet;
	}

	state parse_ethernet {
		pkt.extract(hdr.ethernet);
		transition select(hdr.ethernet.etherType) {
			0x0806: parse_arp;
			default: parse_ip_gateway;
		}
	}

	state parse_ip_gateway {
		transition select(ig_intr_md.ingress_port) {
			N6_PORT_MAPPING // physical port connecting to DN
			// port groups:
			// 1. connected to gNBs
			// meta.uplink = true;
			// 2. connected to UPFs within premises
			// 3. connected to UPFs via open Internet to other PLMNs
			// 4. connected to the Internet
			// 5. connected to "To CP"
			// 6. connected to "buffering"
			default: parse_underlay;
		}
	}

	state parse_underlay {
		transition select(hdr.ethernet.etherType) {
			0x0800: parse_underlay_ipv4;
			0x86dd: parse_underlay_ipv6;
			default: reject;
		}
	}

	state parse_underlay_ipv4 {
		pkt.extract(hdr.underlay_ipv4);
		transition select(hdr.underlay_ipv4.ihl) {
				  5 : parse_underlay_ipv4_no_options;
			6 .. 15 : parse_underlay_ipv4_options;
			default : reject;
		}
	}

	state parse_underlay_ipv4_options {
		pkt.extract(hdr.underlay_ipv4_options, (bit<32>)(hdr.underlay_ipv4.ihl - 5) * 32);
		transition parse_underlay_ipv4_no_options;
	}

	state parse_underlay_ipv4_no_options {
		transition select(hdr.underlay_ipv4.protocol) {
			17: parse_underlay_udp;
			default: reject;
		}
	}

	state parse_underlay_ipv6 {
		pkt.extract(hdr.underlay_ipv6);
		transition select(hdr.underlay_ipv6.nextHdr) {
			17: parse_underlay_udp;
			default: reject;
		}
	}

	state parse_arp {
		pkt.extract(hdr.arp);
		transition accept;
	}

	state parse_underlay_udp {
		pkt.extract(hdr.underlay_udp);
		transition select(hdr.underlay_udp.dstPort) {
			2152: parse_gtpu;
			default: reject;
		}
	}

	state parse_gtpu {
		pkt.extract(hdr.gtpu);
		meta.extracted_teid = hdr.gtpu.teid[(MA_ID_BITS  - COMPRESSED_QFI_BITS - 1):0];
		transition select(hdr.gtpu.extensionHeaderFlag, hdr.gtpu.sequenceNumberFlag, hdr.gtpu.npduNumberFlag) {
			(0, 0, 0): parse_overlay_gateway;
			default: parse_gtpu_optional;
		}
	}

	state parse_gtpu_optional {
		pkt.extract(hdr.gtpu_optional);
		transition select(hdr.gtpu_optional.nextExtensionHeaderType) {
			8w0b10000101: parse_gtpu_psc;
			default: reject; // not handled we can only reject
		}
	}

	state parse_gtpu_psc {
		pkt.extract(hdr.gtpu_ext_psc);
		meta.tunnel_qfi = hdr.gtpu_ext_psc.qfi;
		meta.uplink = (bool)hdr.gtpu_ext_psc.pduType[0:0];
		transition select(hdr.gtpu_ext_psc.extHdrLength) {
			0: reject;
			1: parse_gtpu_psc_optional_1;
			2: parse_gtpu_psc_optional_2;
			3: parse_gtpu_psc_optional_3;
			4: parse_gtpu_psc_optional_4;
			5: parse_gtpu_psc_optional_5;
			6: parse_gtpu_psc_optional_6;
			7: parse_gtpu_psc_optional_7;
			8: parse_gtpu_psc_optional_8;
			9: parse_gtpu_psc_optional_9;
			10: parse_gtpu_psc_optional_10;
			11: parse_gtpu_psc_optional_11;
			12: parse_gtpu_psc_optional_12;
			default: reject;
		}
	}

	state parse_gtpu_psc_optional_1 {
		// skip 8 bit nextHdr
		pkt.advance(8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_2 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (2 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_3 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (3 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_4 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (4 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_5 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (5 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_6 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (6 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_7 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (7 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_8 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (8 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_9 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (9 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_10 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (10 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_11 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (11 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}
	state parse_gtpu_psc_optional_12 {
		pkt.extract(hdr.gtpu_ext_psc_optional, (12 * 4 - 3) * 8);
		transition parse_overlay_gateway;
	}

	state parse_overlay_gateway {
		// handle cases where no more data after GTP-U mandatory header
		transition select(hdr.gtpu.messageType) {
			1: parse_gtpu_control_echo_request;
			2: parse_gtpu_control_echo_response;
			26: parse_gtpu_control_error_indication;
			31: parse_gtpu_control_supported_extension_headers_notification;
			253: parse_gtpu_control_tunnel_status;
			254: parse_gtpu_control_end_marker;
			255: parse_overlay; // G-PDU type with overlay data
			default: accept; // other GTP-U control messages
		}
	}

	state parse_gtpu_control_echo_request {
		transition accept;
	}

	state parse_gtpu_control_echo_response {
		// Skip 29.281:8.2 Recovery IE
		transition accept;
	}

	state parse_gtpu_control_error_indication {
		// Skip 29.281:8.3 Tunnel Endpoint Identifier Data I
		// Skip 29.281:8.4 Extension Header Type List
		transition accept;
	}

	state parse_gtpu_control_supported_extension_headers_notification {
		// Skip 29.281:8.5 Extension Header Type List
		transition accept;
	}

	state parse_gtpu_control_tunnel_status {
		// Skip 29.281:8.7 GTP-U Tunnel Status Information
		transition accept;
	}

	state parse_gtpu_control_end_marker {
		transition accept;
	}

	state parse_overlay {
		bit<4> ip_ver = pkt.lookahead<bit<4>>();
		transition select(ip_ver) {
			4w4: parse_overlay_ipv4;
			4w6: parse_overlay_ipv6;
			default: reject; // other L3 protocol not supported
		}
	}

	state parse_overlay_ipv4 {
		pkt.extract(hdr.overlay_ipv4);
		meta.l4_protocol = hdr.overlay_ipv4.protocol;
		meta.overlay_length = hdr.overlay_ipv4.totalLen;
		transition select(hdr.overlay_ipv4.ihl) {
				  5 : parse_overlay_ipv4_no_options;
			6 .. 15 : parse_overlay_ipv4_options;
			default : reject; // invalid IHL value
		}
	}

	state parse_overlay_ipv4_options {
		pkt.extract(hdr.overlay_ipv4_options, (bit<32>)(hdr.overlay_ipv4.ihl - 5) * 32);
		transition parse_overlay_ipv4_no_options;
	}

	state parse_overlay_ipv4_no_options {
		transition select(hdr.overlay_ipv4.protocol) {
			6: parse_overlay_tcp_udp;
			17: parse_overlay_tcp_udp;
			default: accept; // ICMP and other L4 protocols
		}
	}

	state parse_overlay_ipv6 {
		pkt.extract(hdr.overlay_ipv6);
		meta.l4_protocol = hdr.overlay_ipv6.nextHdr;
		transition select(hdr.overlay_ipv6.nextHdr) {
			6: parse_overlay_tcp_udp;
			17: parse_overlay_tcp_udp;
			default: accept; // ICMP and other L4 protocols
		}
	}

	state parse_overlay_tcp_udp {
		pkt.extract(hdr.overlay_tcp_udp);
		transition accept; // Done
	}
}

control ACL(
	inout   ig_header_t                                 hdr,
	inout   ig_metadata_t                               meta,
	/* Intrinsic */
	in      ingress_intrinsic_metadata_t                ig_intr_md,
	in      ingress_intrinsic_metadata_from_parser_t    ig_prsr_md,
	inout   ingress_intrinsic_metadata_for_deparser_t   ig_dprsr_md,
	inout   ingress_intrinsic_metadata_for_tm_t         ig_tm_md
) {
	
	apply {

	}
}

control AccountingIngress(
	/* Flow Identifiers */
	in      ma_id_t                               ma_id
)(bit<32> adj) {
	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters1;
	action inc_counter1() {
		usage_counters1.count(adj);
	}

	table accounting_exact1 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter1;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters1;
		const size = TABLE_SIZE_ACCOUNTING; // >> 1;
	}


	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters2;
	action inc_counter2() {
		usage_counters2.count(adj);
	}

	table accounting_exact2 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter2;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters2;
		const size = TABLE_SIZE_ACCOUNTING >> 1;
	}

	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters3;
	action inc_counter3() {
		usage_counters3.count(adj);
	}

	table accounting_exact3 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter3;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters3;
		const size = TABLE_SIZE_ACCOUNTING >> 2;
	}


	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters4;
	action inc_counter4() {
		usage_counters4.count(adj);
	}

	table accounting_exact4 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters4;
		const size = TABLE_SIZE_ACCOUNTING >> 2;
	}
	apply {
		accounting_exact1.apply();
		// accounting_exact2.apply();
		// accounting_exact3.apply();
		// accounting_exact4.apply();
	}
}

control AccountingEgress(
	/* Flow Identifiers */
	in      ma_id_t                               ma_id
)(bit<32> adj) {
	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters1;
	action inc_counter1() {
		usage_counters1.count(adj);
	}

	table accounting_exact1 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter1;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters1;
		const size = TABLE_SIZE_ACCOUNTING; // >> 1;
	}


	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters2;
	action inc_counter2() {
		usage_counters2.count(adj);
	}

	table accounting_exact2 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter2;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters2;
		const size = TABLE_SIZE_ACCOUNTING >> 1;
	}

	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters3;
	action inc_counter3() {
		usage_counters3.count(adj);
	}

	table accounting_exact3 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter3;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters3;
		const size = TABLE_SIZE_ACCOUNTING >> 2;
	}


	DirectCounter<bit<36>>(CounterType_t.PACKETS_AND_BYTES) usage_counters4;
	action inc_counter4() {
		usage_counters4.count(adj);
	}

	table accounting_exact4 {
		key = {
			ma_id : exact;
		}
		actions = {
			inc_counter4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		counters = usage_counters4;
		const size = TABLE_SIZE_ACCOUNTING >> 2;
	}
	apply {
		accounting_exact1.apply();
		// accounting_exact2.apply();
		// accounting_exact3.apply();
		// accounting_exact4.apply();
	}
}

control RoutingIPv4(
	/* User */
	inout   ig_header_t                                 hdr,
	in      ipv4_addr_t                                 dst_ip,
	/* Intrinsic */
	inout   ingress_intrinsic_metadata_for_tm_t         ig_tm_md
) {
	// -----------------------------------------------------------------
	// step 7 : routing

	action send(PortId_t port, mac_addr_t src_mac, mac_addr_t dst_mac) {
		ig_tm_md.ucast_egress_port = port;
		hdr.ethernet.dst = dst_mac;
		hdr.ethernet.src = src_mac;
	}

	table ipv4_exact {
		key = {
			dst_ip           : exact;
		}
		actions = {
			send;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_IPV4_EXACT;
	}

	table ipv4_lpm {
		key = {
			dst_ip           : lpm;
		}
		actions = {
			send;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_IPV4_LPM;
	}

	apply {
		if (ipv4_exact.apply().miss) {
			ipv4_lpm.apply();
		}
	}
}

control Ingress(
	/* User */
	inout   ig_header_t                                 hdr,
	inout   ig_metadata_t                               meta,
	/* Intrinsic */
	in      ingress_intrinsic_metadata_t                ig_intr_md,
	in      ingress_intrinsic_metadata_from_parser_t    ig_prsr_md,
	inout   ingress_intrinsic_metadata_for_deparser_t   ig_dprsr_md,
	inout   ingress_intrinsic_metadata_for_tm_t         ig_tm_md
) {
	// -----------------------------------------------------------------
	//                         Common actions

	action mark_for_drop() {
		ig_dprsr_md.drop_ctl = ig_dprsr_md.drop_ctl | 0b001;
	}

	action actual_drop() {
		mark_for_drop();
		exit;
	}

	action arp_reply(mac_addr_t request_mac) {
		//update operation code from request to reply
		hdr.arp.op_code = ARP_REPLY;
		
		hdr.arp.dst_mac = hdr.arp.src_mac;
		
		hdr.arp.src_mac = request_mac;

		ipv4_addr_t tmp = hdr.arp.src_ip;
		hdr.arp.src_ip = hdr.arp.dst_ip;
		hdr.arp.dst_ip = tmp;

		//update ethernet header
		hdr.ethernet.dst = hdr.ethernet.src;
		hdr.ethernet.src = request_mac;

		//send it back to the same port
		ig_tm_md.ucast_egress_port = ig_intr_md.ingress_port;
		ig_tm_md.bypass_egress = 1;
	}

	action gtpu_echo_response_ipv4() {
		ipv4_addr_t tmp = hdr.underlay_ipv4.srcAddr;
		hdr.underlay_ipv4.srcAddr = hdr.underlay_ipv4.dstAddr;
		hdr.underlay_ipv4.dstAddr = tmp;
	}

	table handle_gtpu_control_msg_table {
		key = {
			hdr.gtpu.isValid()          : exact;
			hdr.gtpu.messageType        : exact;
			hdr.underlay_ipv4.isValid() : exact;
		}
		actions = {
			gtpu_echo_response_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const entries = {
			(true, 1, true) : gtpu_echo_response_ipv4();
		}
		const size = 1;
	}

	// UPF constants
	ipv4_addr_t             underlay_src_ipv4 = 0;

	action set_upf_ip_table_set_ip(ipv4_addr_t upf_ipv4) {
		underlay_src_ipv4 = upf_ipv4;
	}

	table set_upf_ip_table {
		key = {
			//meta.always_one : exact;
		}
		actions = {
			//@defaultonly NoAction;
			set_upf_ip_table_set_ip;
		}
		//const default_action = NoAction();
		const size = 1;
	}

	// common actions

	bool do_nocp = false;
	bool do_buffer = false;

	action decap() { // used by UL_DECAP_TO_DN
		hdr.underlay_ipv4.setInvalid();
		hdr.underlay_ipv4_options.setInvalid();
		hdr.underlay_ipv6.setInvalid();
		hdr.underlay_udp.setInvalid();
		hdr.gtpu.setInvalid();
		hdr.gtpu_optional.setInvalid();
		hdr.gtpu_ext_psc.setInvalid();
		hdr.gtpu_ext_psc_optional.setInvalid();
		hdr.gtpu_ext_psc_next_header_type.setInvalid();
	}

	PDR_single_stage()   pdr_all;

	// post match tables

	// UL
	action ul_no_mark_tos_table_forward(bool nocp) {
		decap();
		do_nocp = nocp;
	}

	action ul_no_mark_tos_table_drop() {
		mark_for_drop();
	}

	table ul_no_mark_tos_table {
		key = {
			meta.ma_id : ternary;
		}
		actions = {
			@defaultonly NoAction;
			ul_no_mark_tos_table_forward;
			ul_no_mark_tos_table_drop;
		}
		const default_action = NoAction();
		const size = 512 * 2;
	}

	bool do_drop = false;

	action ul_mark_tos_table_forward_v4(bool mark_tos, bit<8> tos_value, bool nocp, bool drop) {
		decap();
		do_drop = drop;
		if (mark_tos)
			hdr.overlay_ipv4.diffserv = tos_value;
		do_nocp = nocp;
	}

	table ul_mark_tos_table {
		key = {
			meta.ma_id : ternary;
		}
		actions = {
			@defaultonly NoAction;
			ul_mark_tos_table_forward_v4;
			mark_for_drop;
		}
		const default_action = NoAction();
		const size = 512 * 4;
	}

	action ul_to_N3N9_table_table_v4(ipv4_addr_t gnb_ipv4, qfi_t qfi_v, bool nocp) {
		do_nocp = nocp;
		hdr.underlay_ipv4.setValid();
		hdr.underlay_ipv4.srcAddr = underlay_src_ipv4;
		hdr.underlay_ipv4.dstAddr = gnb_ipv4;
		// other IPv4 fields
		hdr.underlay_ipv4.version = 4;
		hdr.underlay_ipv4.ihl = 5;
		hdr.underlay_ipv4.diffserv = 0;
		hdr.underlay_ipv4.identification = 16w0x1145;
		hdr.underlay_ipv4.flags = 3w0b010;
		hdr.underlay_ipv4.fragOffset = 0;
		hdr.underlay_ipv4.ttl = 65;
		hdr.underlay_ipv4.protocol = 17;

		hdr.underlay_udp.setValid();
		hdr.underlay_udp.srcPort = 2152;
		hdr.underlay_udp.dstPort = 2152;
		hdr.underlay_udp.len =
			8 + // UDP length
			8 + // GTP-U header
			4 + // GTP-U optional header
			4 + // GTP-U PSC ext header
			meta.overlay_length;
		// TODO: UDP checksum
		hdr.underlay_udp.checksum = 0;


		hdr.underlay_ipv4.totalLen =
			20 + // IP header
			8 +  // UDP length
			8 +  // GTP-U header
			4 +  // GTP-U optional header
			4 +  // GTP-U PSC ext header
			meta.overlay_length;

		hdr.gtpu.setValid();
		hdr.gtpu.version = 3w1;
		hdr.gtpu.protocolType = 1;
		hdr.gtpu.spare = 0;
		hdr.gtpu.extensionHeaderFlag = 1;
		hdr.gtpu.sequenceNumberFlag = 0;
		hdr.gtpu.npduNumberFlag = 0;
		hdr.gtpu.messageType = 255;
		hdr.gtpu.payloadLength =
			4 + // GTP-U optional header
			4 + // GTP-U PSC ext header
			meta.overlay_length;
		hdr.gtpu.teid = meta.teid;

		hdr.gtpu_optional.setValid();
		hdr.gtpu_optional.sequenceNumber = 0;
		hdr.gtpu_optional.npduNumber = 0;
		hdr.gtpu_optional.nextExtensionHeaderType = 8w0b10000101;

		hdr.gtpu_ext_psc.setValid();
		hdr.gtpu_ext_psc.extHdrLength = 1;
		hdr.gtpu_ext_psc.pduType = 0;
		hdr.gtpu_ext_psc.dontCare = 0;
		hdr.gtpu_ext_psc.qfi = qfi_v;
		meta.tunnel_qfi = qfi_v;

		hdr.gtpu_ext_psc_next_header_type.setValid();
		hdr.gtpu_ext_psc_next_header_type.content = 0;
	}

	table ul_to_N3N9_table {
		key = {
			meta.ma_id : ternary;
		}
		actions = {
			@defaultonly NoAction;
			ul_to_N3N9_table_table_v4;
		}
		const default_action = NoAction();
		const size = 512 * 4;
	}

	action dl_to_N3N9_table_v4(ipv4_addr_t gnb_ipv4, qfi_t qfi_v, bool nocp, bool buf, bool drop) {
		do_drop = drop;
		do_nocp = nocp;
		do_buffer = buf;
		hdr.underlay_ipv4.setValid();
		hdr.underlay_ipv4.srcAddr = underlay_src_ipv4;
		hdr.underlay_ipv4.dstAddr = gnb_ipv4;
		// other IPv4 fields
		hdr.underlay_ipv4.version = 4;
		hdr.underlay_ipv4.ihl = 5;
		hdr.underlay_ipv4.diffserv = 0;
		hdr.underlay_ipv4.identification = 16w0x1145;
		hdr.underlay_ipv4.flags = 3w0b010;
		hdr.underlay_ipv4.fragOffset = 0;
		hdr.underlay_ipv4.ttl = 65;
		hdr.underlay_ipv4.protocol = 17;

		hdr.underlay_udp.setValid();
		hdr.underlay_udp.srcPort = 2152;
		hdr.underlay_udp.dstPort = 2152;
		hdr.underlay_udp.len =
			8 + // UDP length
			8 + // GTP-U header
			4 + // GTP-U optional header
			4 + // GTP-U PSC ext header
			meta.overlay_length;
		// TODO: UDP checksum
		hdr.underlay_udp.checksum = 0;


		hdr.underlay_ipv4.totalLen =
			20 + // IP header
			8 +  // UDP length
			8 +  // GTP-U header
			4 +  // GTP-U optional header
			4 +  // GTP-U PSC ext header
			meta.overlay_length;

		hdr.gtpu.setValid();
		hdr.gtpu.version = 3w1;
		hdr.gtpu.protocolType = 1;
		hdr.gtpu.spare = 0;
		hdr.gtpu.extensionHeaderFlag = 1;
		hdr.gtpu.sequenceNumberFlag = 0;
		hdr.gtpu.npduNumberFlag = 0;
		hdr.gtpu.messageType = 255;
		hdr.gtpu.payloadLength =
			4 + // GTP-U optional header
			4 + // GTP-U PSC ext header
			meta.overlay_length;
		hdr.gtpu.teid = meta.teid;

		hdr.gtpu_optional.setValid();
		hdr.gtpu_optional.sequenceNumber = 0;
		hdr.gtpu_optional.npduNumber = 0;
		hdr.gtpu_optional.nextExtensionHeaderType = 8w0b10000101;

		hdr.gtpu_ext_psc.setValid();
		hdr.gtpu_ext_psc.extHdrLength = 1;
		hdr.gtpu_ext_psc.pduType = 0;
		hdr.gtpu_ext_psc.dontCare = 0;
		hdr.gtpu_ext_psc.qfi = qfi_v;
		meta.tunnel_qfi = qfi_v;

		hdr.gtpu_ext_psc_next_header_type.setValid();
		hdr.gtpu_ext_psc_next_header_type.content = 0;
	}

	// DL
	table dl_to_N3N9_table {
		key = {
			meta.ma_id : ternary;
		}
		actions = {
			@defaultonly NoAction;
			dl_to_N3N9_table_v4;
			mark_for_drop;
		}
		const default_action = NoAction();
		const size = 512 * 14;
	}

	// step 2:

	action set_bridge_header() {
		hdr.bridge.header_type = HEADER_TYPE_BRIDGE;
		hdr.bridge.header_info = 0;
		hdr.bridge.ingress_ts = ig_prsr_md.global_tstamp;
		hdr.bridge.ma_id = meta.ma_id;
	}

	ACL() acl;

	DirectMeter(MeterType_t.BYTES) bitrate_enforce_meters;

	action set_meter_color() {
		meta.individual_gbr_meter_color = bitrate_enforce_meters.execute();
	}

	table bitrate_enforce_table {
		key = {
			meta.qer_id     : exact;
		}
		actions = {
			@defaultonly NoAction;
			set_meter_color;
		}
		const default_action = NoAction();
		meters = bitrate_enforce_meters;
		size = TABLE_SIZE_ACCOUNTING >> 3;
	}

	action put_in_queue(QueueId_t qid) {
		ig_tm_md.qid = qid;
	}

	table qfi_to_queue_table {
		key = {
			meta.qfi : exact;
		}
		actions = {
			put_in_queue;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = 64;
	}

	// step 3:

	action send_to_cpu_table_action(bit<8> flags, bool copy_to_cpu) {
		if (copy_to_cpu) {
			ig_tm_md.copy_to_cpu = 1;
		}
		hdr.cpu_header.setValid();
		hdr.cpu_header.cpu_header_magic = CPU_HEADER_MAGIC;
		hdr.cpu_header.flags = flags;
		hdr.cpu_header.ma_id = (bit<32>)meta.ma_id;
	}

	table send_to_cpu_table {
		key = {
			do_buffer : exact;
			do_nocp   : exact;
		}
		actions = {
			@defaultonly NoAction;
			send_to_cpu_table_action;
		}
		const default_action = NoAction();
		const entries = {
			(true, false) : send_to_cpu_table_action(8w0b00000010, false);
			(true,  true) : send_to_cpu_table_action(8w0b00000011, true);
			(false, true) : send_to_cpu_table_action(8w0b00000001, true);
		}
		const size = 3;
	}

	// step 3: routing
	RoutingIPv4() ipv4_routing_overlay;
	RoutingIPv4() ipv4_routing_underlay;

	AccountingIngress(0) accounting;

	apply {
		// step 0: common ops
		if (hdr.ethernet.etherType == TYPE_ARP) {
			arp_reply(UPF_MAC);
		}
		set_upf_ip_table.apply();
		hdr.bridge.setValid();
		handle_gtpu_control_msg_table.apply();
		if ((hdr.gtpu.messageType == 255 && hdr.gtpu.isValid()) || !hdr.gtpu.isValid()) { // only do PDR match and action if not GTP-U control messages
			// step 1: match
			pdr_all.apply(hdr, meta, meta.extracted_teid);
			// step 2: match MA_ID to do actions
			if (meta.uplink) {
				//if (ul_no_mark_tos_table.apply().miss) {
					if (ul_mark_tos_table.apply().miss) {
						ul_to_N3N9_table.apply();
					}
				//}
			} else {
				if (dl_to_N3N9_table.apply().miss) {
					//dl_to_drop_table.apply();
				}
			}
		}
		// step 2: accounting
		accounting.apply(meta.ma_id);
		// step 2: AMBR enforcement
		bitrate_enforce_table.apply();
		if (meta.individual_gbr_meter_color == MeterColor_t.RED || do_drop) {
			mark_for_drop();
		} else if (meta.individual_gbr_meter_color == MeterColor_t.GREEN || hdr.gtpu.messageType != 255) {
			ig_tm_md.qid = 0;
		} else {
			qfi_to_queue_table.apply();
		}
		// step 2: set_bridge_header
		set_bridge_header();
		// step 2: ACL
		acl.apply(hdr, meta, ig_intr_md, ig_prsr_md, ig_dprsr_md, ig_tm_md);
		// step 3: NoCP and Buffer (send to CPU)
		send_to_cpu_table.apply();
		if (do_buffer) {
			ig_tm_md.ucast_egress_port = PORT_BUFFER;
		} else {
			// step 3: IP routing
			if (hdr.underlay_ipv4.isValid()) {
				ipv4_routing_underlay.apply(hdr, hdr.underlay_ipv4.dstAddr, ig_tm_md);
			} else if (hdr.overlay_ipv4.isValid()) {
				ipv4_routing_overlay.apply(hdr, hdr.overlay_ipv4.dstAddr, ig_tm_md);
			}
		}
	}
}

control IngressDeparser(
			packet_out                                  pkt,
	/* User */
	inout   ig_header_t                                 hdr,
	in      ig_metadata_t                               meta,
	/* Intrinsic */
	in      ingress_intrinsic_metadata_for_deparser_t   ig_intr_dprsr_md
) {
	Checksum() underlay_ip_checksum;
	Checksum() overlay_ip_checksum;
	apply {
		if (hdr.underlay_ipv4.isValid()) {
			hdr.underlay_ipv4.hdrChecksum = underlay_ip_checksum.update(
				{
					hdr.underlay_ipv4.version,
					hdr.underlay_ipv4.ihl,
					hdr.underlay_ipv4.diffserv,
					hdr.underlay_ipv4.totalLen,
					hdr.underlay_ipv4.identification,
					hdr.underlay_ipv4.flags,
					hdr.underlay_ipv4.fragOffset,
					hdr.underlay_ipv4.ttl,
					hdr.underlay_ipv4.protocol,
					hdr.underlay_ipv4.srcAddr,
					hdr.underlay_ipv4.dstAddr
				}
			);
		}
		if (hdr.overlay_ipv4.isValid()) {
			hdr.overlay_ipv4.hdrChecksum = overlay_ip_checksum.update(
				{
					hdr.overlay_ipv4.version,
					hdr.overlay_ipv4.ihl,
					hdr.overlay_ipv4.diffserv,
					hdr.overlay_ipv4.totalLen,
					hdr.overlay_ipv4.identification,
					hdr.overlay_ipv4.flags,
					hdr.overlay_ipv4.fragOffset,
					hdr.overlay_ipv4.ttl,
					hdr.overlay_ipv4.protocol,
					hdr.overlay_ipv4.srcAddr,
					hdr.overlay_ipv4.dstAddr
				}
			);
		}
		pkt.emit(hdr);
	}
}

// ------------------------------------------------------------------------------------------
//                                      EGRESS STAGE
// ------------------------------------------------------------------------------------------

parser EgressParser(
			packet_in                                   pkt,
	/* User */
	out     eg_headers_t                                hdr,
	out     eg_metadata_t                               meta,
	/* Intrinsic */
	out     egress_intrinsic_metadata_t                 eg_intr_md
)
{
	internal_header_h inthdr;
	
	/* This is a mandatory state, required by Tofino Architecture */
	state start {
		meta.flow_key_reporting_packet = false;
		pkt.extract(eg_intr_md);
		// inthdr = pkt.lookahead<internal_header_h>();
		// transition select(inthdr.header_type, inthdr.header_info) {
		// 	(HEADER_TYPE_BRIDGE   ,                              _) : parse_bridge;
		// 	(HEADER_TYPE_EG_MIRROR, HEADER_INFO_FLOW_KEY_REPORTING) : extract_eg_mirror_flow_key_reporting;
		// 	default: reject;
		// }
		transition parse_bridge;
	}

	state parse_bridge {
		pkt.extract(meta.bridge);
		transition parse_ethernet;
	}

	state parse_ethernet {
		pkt.extract(hdr.ethernet);
		bit<8> nextHdr = pkt.lookahead<bit<8>>();
		transition select(nextHdr) {
			CPU_HEADER_MAGIC: parse_cpu_header;
			default: accept;
		}
	}

	state extract_eg_mirror_flow_key_reporting {
		pkt.extract<eg_mirror_header_flow_key_reporting_h>(_);
		meta.flow_key_reporting_packet = true;
		transition parse_ethernet_no_bridge;
	}

	state parse_ethernet_no_bridge {
		pkt.extract(hdr.ethernet);
		transition accept;
	}

	state parse_cpu_header {
		pkt.extract(hdr.cpu_header);
		transition accept;
	}
}


// --------------------------------------------------
//                  Egress Control
// --------------------------------------------------
control Egress(
	/* User */
	inout   eg_headers_t                                hdr,
	inout   eg_metadata_t                               meta,
	/* Intrinsic */
	in      egress_intrinsic_metadata_t                 eg_intr_md,
	in      egress_intrinsic_metadata_from_parser_t     eg_intr_md_from_prsr,
	inout   egress_intrinsic_metadata_for_deparser_t    eg_intr_dprs_md,
	inout   egress_intrinsic_metadata_for_output_port_t eg_intr_oport_md
) {
	AccountingEgress(15) accounting;

	// step 3: stats tracking
	port_queue_id_t stats_index = 0;
	Lpf<bit<32>, port_queue_id_t>(size = 1024) lpf_per_queue_rate;
	Register<bit<32>, port_queue_id_t>(size = 1024) reg_lpf_per_queue_rate;
	Lpf<bit<16>, port_queue_id_t>(size = 1024) lpf_per_queue_depth;
	Register<bit<16>, port_queue_id_t>(size = 1024) reg_lpf_per_queue_depth;
	bit<32> lpf_per_queue_rate_result;
	bit<16> lpf_per_queue_depth_result;

	Lpf<bit<32>, port_queue_id_t>(size = 1024) lpf_per_queue_delay;
	Register<bit<32>, port_queue_id_t>(size = 1024) reg_lpf_per_queue_delay_result;
	bit<32> lpf_per_queue_delay_result;

	Counter<bit<32>, port_queue_id_t>(1024, CounterType_t.PACKETS) qos_reach_eg;

	action set_index(port_queue_id_t idx) {
		stats_index = idx;
	}
	table eg_stats_set_index_table {
		key = {
			eg_intr_md.egress_port[8:3] : exact;
			eg_intr_md.egress_qid       : exact;
		}
		actions = {
			@defaultonly NoAction;
			set_index;
		}
		const size = 1024;
	}

	bit<32> time_diff;

	action set_time_diff() {
		time_diff = eg_intr_md_from_prsr.global_tstamp[31:0] - meta.bridge.ingress_ts[31:0];
	}

	apply {
		set_time_diff();
		if (eg_intr_md.egress_port != PORT_CPU && eg_intr_md.egress_port != PORT_BUFFER) {
			// for packets not going to CPU
			hdr.cpu_header.setInvalid();
			// -----------------------------------------------------------------
			// step 2 : Post QoS Accounting
			accounting.apply(meta.bridge.ma_id);
		}
	}
}


control EgressDeparser(
			packet_out                                  pkt,
	/* User */
	inout   eg_headers_t                                hdr,
	in      eg_metadata_t                               meta,
	/* Intrinsic */
	in      egress_intrinsic_metadata_for_deparser_t    eg_dprsr_md
) {
	apply {
		pkt.emit(hdr);
	}
}


Pipeline(
	IngressParser(),
	Ingress(),
	IngressDeparser(),
	EgressParser(),
	Egress(),
	EgressDeparser()
) pipe;

Switch(pipe) main;
