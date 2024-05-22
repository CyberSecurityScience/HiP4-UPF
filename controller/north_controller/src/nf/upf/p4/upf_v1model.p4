/* -*- P4_16 -*- */
#include <core.p4>
#include <v1model.p4>

error {
	GTP_VERSION_UNSUPPORTED,
	GTP_PRIME_UNSUPPORTED,
	GTP_EXTHEADER_UNSUPPORTED
}

typedef bit<48> MacAddress;

header ethernet_t {
	MacAddress dst;
	MacAddress src;
	bit<16> etherType;
}

header ipv4_t {
	bit<4>       version;
	bit<4>       ihl;
	bit<8>       diffserv;
	bit<16>      totalLen;
	bit<16>      identification;
	bit<3>       flags;
	bit<13>      fragOffset;
	bit<8>       ttl;
	bit<8>       protocol;
	bit<16>      hdrChecksum;
	bit<32>      srcAddr;
	bit<32>      dstAddr;
	varbit<320>  options;
}

header IPv4_up_to_ihl_only_h {
    bit<4> version;
    bit<4> ihl;
}

header ipv6_t {
	bit<4>   version;
	bit<8>   trafficClass;
	bit<20>  flowLabel;
	bit<16>  payloadLen;
	bit<8>   nextHdr;
	bit<8>   hopLimit;
	bit<128> srcAddr;
	bit<128> dstAddr;
}

header udp_t {
	bit<16> srcPort;
	bit<16> dstPort;
	bit<16> len;
	bit<16> checksum;
}

header tcp_t {
	bit<16>      srcPort;
	bit<16>      dstPort;
	bit<32>      seqNo;
	bit<32>      ackNo;
	bit<4>       dataOffset;
	bit<3>       res;
	bit<3>       ecn;
	bit<6>       ctrl;
	bit<16>      window;
	bit<16>      checksum;
	bit<16>      urgentPtr;
	varbit<320>  options;
}

header tcp_upto_data_offset_only_h {
    bit<16> srcPort;
    bit<16> dstPort;
    bit<32> seqNo;
    bit<32> ackNo;
    // dataOffset in TCP hdr uses 4 bits but needs padding.
    // If 4 bits are used for it, p4c-bm2-ss complains the header
    // is not a multiple of 8 bits.
    bit<4>  dataOffset;
    bit<4>  dontCare;
}

header padding_1_octet {
	bit<8> pad;
}

header padding_2_octet {
	bit<16> pad;
}

header padding_3_octet {
	bit<24> pad;
}

header gtpu_extheader_type_t {
	bit<8> value;
}

header gtpu_extheader_padding_t {
	varbit<24> value;
}

header gtpu_header_mandatory_t {
	bit<3>  version;
	bit<1>  pt;
	bit<1>  padding;
	bit<1>  e;
	bit<1>  s;
	bit<1>  pn;
	bit<8>  msgType;
	bit<16> len;
	bit<32> teid;
}

header gtpu_header_optional_t {
	bit<16> seqNum;
	bit<8>  nPduType;
	bit<8>  extHeaderType;
}

header gtpu_extheader_ul_pdusessioninfo_qmp_t {
	bit<64> dlSendingTimestampRepeated;
	bit<64> dlReceivedTimestamp;
	bit<64> ulSendingTimestamp;
}

header gtpu_extheader_ul_pdusessioninfo_dl_delay_result_t {
	bit<32> value;
}

header gtpu_extheader_ul_pdusessioninfo_ul_delay_result_t {
	bit<32> value;
}

header gtpu_extheader_ul_pdusessioninfo_qfi_seqnum_t {
	bit<24> value;
}

header gtpu_extheader_ul_pdusessioninfo_n3_n9_delay_result_t {
	bit<32> value;
}

header gtpu_extheader_ul_pdusessioninfo_mandatory_header_t {
	bit<8> lengthIn4octets;
	bit<4> pduType;
	bit<1> qmp;
	bit<1> dlDelayInd;
	bit<1> ulDelayInd;
	bit<1> snp;
	bit<1> n3n9DelayInd;
	bit<1> spare;
	bit<6> qfi;
}

struct gtpu_extheader_ul_pdusessioninfo_t {
	gtpu_extheader_ul_pdusessioninfo_mandatory_header_t		mandatoryHeader;
	gtpu_extheader_ul_pdusessioninfo_qmp_t					qmpFields;
	gtpu_extheader_ul_pdusessioninfo_dl_delay_result_t  	dlDelayResult;
	gtpu_extheader_ul_pdusessioninfo_ul_delay_result_t  	ulDelayResult;
	gtpu_extheader_ul_pdusessioninfo_qfi_seqnum_t			qfiSeqNum;
	gtpu_extheader_ul_pdusessioninfo_n3_n9_delay_result_t	n3n9DeplayResult;
	gtpu_extheader_padding_t								padding; // padding extracted
	padding_1_octet											outpad_1;// padding used for deparser
	padding_2_octet											outpad_2;
	padding_3_octet											outpad_3;
	gtpu_extheader_type_t									nextExtHeaderType;
}

header gtpu_extheader_dl_pdusessioninfo_qmp_t {
	bit<64> value;
}

header gtpu_extheader_dl_pdusessioninfo_qfi_seqnum_t {
	bit<24> value;
}

header gtpu_extheader_dl_pdusessioninfo_ppi_t {
	bit<3> ppi;
	bit<5> spare2;
}

header gtpu_extheader_dl_pdusessioninfo_mandatory_header_t {
	bit<8> lengthIn4octets;
	bit<4> pduType;
	bit<1> qmp;
	bit<1> snp;
	bit<2> spare1;
	bit<1> ppp;
	bit<1> rqi;
	bit<6> qfi;
}
struct gtpu_extheader_dl_pdusessioninfo_t {
	gtpu_extheader_dl_pdusessioninfo_mandatory_header_t	mandatoryHeader;
	gtpu_extheader_dl_pdusessioninfo_ppi_t				ppi;
	gtpu_extheader_dl_pdusessioninfo_qmp_t				qmpFields;
	gtpu_extheader_dl_pdusessioninfo_qfi_seqnum_t		qfiSeqNum;
	gtpu_extheader_padding_t							padding;  // padding extracted
	padding_1_octet										outpad_1;
	padding_2_octet										outpad_2;
	padding_3_octet										outpad_3;
	gtpu_extheader_type_t								nextExtHeaderType;
}

header gtpu_extheader_length_and_pdutype {
	bit<8> lengthIn4octets;
	bit<4> pduType;
	bit<4> unused;
}


struct gtpu_t {
	gtpu_header_mandatory_t				mandatoryHeader;
	gtpu_header_optional_t 				optionalHeader;
	gtpu_extheader_ul_pdusessioninfo_t 	extUL;
	gtpu_extheader_dl_pdusessioninfo_t 	extDL;
}

@controller_header("packet_in")
header packet_in_t {
    bit<9> ingress_port;
    bit<7> padding;
    bit<64> seid;
    bit<16> pdr_id;
	bit<8> flag; // 0 for Buffer, 1 for Send To CP
}

@controller_header("packet_out")
header packet_out_t {
    bit<9> ingress_port;
    bit<7> padding;
	bit<8> flag; // 0 for regular, 1 for End Marker
}

struct upf_headers_t {
	packet_in_t controller_in;
	packet_out_t controller_out;
	ethernet_t ethernet;
	ipv4_t underlay_ipv4;
	ipv6_t underlay_ipv6;
	udp_t underlay_udp;
	gtpu_t gtpu;
	ipv4_t overlay_ipv4;
	ipv6_t overlay_ipv6;
	udp_t overlay_udp;
	tcp_t overlay_tcp;
}

struct upf_metadata_t {
	bit<16>  underlay_ipv4_l4len;
	bit<16>  overlay_ipv4_l4len;
	bit<8>   tmpCounter; // used during parsing
	bit<1>   tunnel_valid; // if GTP-U tunnel is/shall be present
	bit<32>  tunnel_teid;  // GTP-U tunnel TEID
	bit<1>   tunnel_ipv4_or_ipv6; // 1 for ipv4, 0 for ipv6
	bit<32>  tunnel_ipv4_addr;
	bit<128> tunnel_ipv6_addr;
	bit<8>   tunnel_ipv6_prefix_len; // in bits
	bit<1>   tunnel_qfi_valid; // if GTP-U tunnel's QFI is/shall be present
	bit<6>   tunnel_qfi; // GTP-U tunnel QFIt
	bit<1>   ul_or_dl; // 1 fpr ul, 0 for dl
	bit<1>   to_n6; // used to send to DN on egress path
	bit<1>   to_cp; // send to CP
	bit<64>  seid; // SEID
	bit<16>  pdr_id; // PDR ID
	bit<1>   bypass; // bypass all subsequent stages?
	bit<2>   meter_result; // 0 for GREEN, 1 for YELLOW, 2 for RED
}

parser EthParser(packet_in                 packet,
				out upf_headers_t          hdr,
				inout upf_metadata_t       meta,
				inout standard_metadata_t  standard_metadata){
	state start {
		meta.underlay_ipv4_l4len = 0;
		meta.overlay_ipv4_l4len = 0;
		meta.tunnel_valid = 0;
		meta.tunnel_teid = 0;
		meta.tunnel_ipv4_addr = 0;
		meta.tunnel_ipv6_addr = 0;
		meta.tunnel_ipv6_prefix_len = 0;
		meta.tunnel_qfi = 0;
		meta.tunnel_qfi_valid = 0;
		meta.seid = 0;
		meta.to_n6 = 0;
		meta.to_cp = 0;
		meta.bypass = 0;
		meta.meter_result = 0;
		transition select(standard_metadata.ingress_port) {
			255: parse_packet_out; // for regular packets which goes through the entire pipeline
			default: parse_ethernet;
		}
	}

	state parse_packet_out {
		packet.extract(hdr.controller_out);
		standard_metadata.ingress_port = hdr.controller_out.ingress_port;
		transition select(hdr.controller_out.flag) {
			8w1: accept; // End Marker 
			default: parse_ethernet;
		}
	}

	state parse_ethernet {
		packet.extract(hdr.ethernet);
		transition select(standard_metadata.ingress_port) {
			9w1: parse_overlay; // physical port connecting to DN
			default: parse_underlay;
		}
	}

	/// --------------------------
	///    Parse GTP-U Tunnel
	/// --------------------------

	state parse_underlay {
		// parse ip
		transition select(hdr.ethernet.etherType) {
			0x0800: parse_underlay_ipv4;
			0x86dd: parse_underlay_ipv6;
			0x0806: accept; // ARP
			default: reject;
		}
	}

	state parse_underlay_ipv4 {
		meta.tunnel_ipv4_or_ipv6 = 1;
		packet.extract(hdr.underlay_ipv4,
                    (bit<32>)
                    (8 *
                     (4 * (bit<9>) (packet.lookahead<IPv4_up_to_ihl_only_h>().ihl)
                      - 20)));
		meta.tunnel_ipv4_addr = hdr.underlay_ipv4.dstAddr;
		meta.underlay_ipv4_l4len = hdr.underlay_ipv4.totalLen - (bit<16>)(hdr.underlay_ipv4.ihl) * 4;
		transition select(hdr.underlay_ipv4.protocol) {
			//6: parse_tcp;
			17: parse_underlay_udp;
			default: reject;
		}
	}

	state parse_underlay_ipv6 {
		meta.tunnel_ipv4_or_ipv6 = 0;
		packet.extract(hdr.underlay_ipv6);
		meta.tunnel_ipv6_addr = hdr.underlay_ipv6.dstAddr;
		meta.tunnel_ipv6_prefix_len = 128; // or 64
		transition select(hdr.underlay_ipv6.nextHdr) {
			//6: parse_tcp;
			17: parse_underlay_udp;
			default: reject;
		}
	}

	state parse_underlay_udp {
		packet.extract(hdr.underlay_udp);
		transition select(hdr.underlay_udp.dstPort) {
			2152: parse_gtpu;
			default: reject;
		}
	}

	state parse_gtpu {
		// work starts here
		meta.tunnel_valid = 1;
		packet.extract(hdr.gtpu.mandatoryHeader);
		meta.tunnel_teid = hdr.gtpu.mandatoryHeader.teid;
		verify(hdr.gtpu.mandatoryHeader.version == 1, error.GTP_VERSION_UNSUPPORTED);
		verify(hdr.gtpu.mandatoryHeader.pt == 1, error.GTP_PRIME_UNSUPPORTED);
		transition select(hdr.gtpu.mandatoryHeader.msgType, hdr.gtpu.mandatoryHeader.e, hdr.gtpu.mandatoryHeader.s, hdr.gtpu.mandatoryHeader.pn) {
			(255, 1, 0 &&& 0, 0 &&& 0): parse_gtpu_optional_header;
			(255, 0 &&& 0, 1, 0 &&& 0): parse_gtpu_optional_header;
			(255, 0 &&& 0, 0 &&& 0, 1): parse_gtpu_optional_header;
			(255, 0 &&& 0, 0 &&& 0, 0 &&& 0): parse_overlay;
			default: accept; // non G-PDU inside GTP-U
		}
	}

	state parse_gtpu_optional_header {
		packet.extract(hdr.gtpu.optionalHeader);
		verify(hdr.gtpu.optionalHeader.extHeaderType == 0 || hdr.gtpu.optionalHeader.extHeaderType == 8w0b10000101, error.GTP_EXTHEADER_UNSUPPORTED);
		transition select(hdr.gtpu.optionalHeader.extHeaderType) {
			8w0b10000101: parse_gtpu_extheader_pdusession_info;
			0: parse_overlay;
			default: reject;
		}
	}

	state parse_gtpu_extheader_pdusession_info {
		bit<4> pdu_type = packet.lookahead<gtpu_extheader_length_and_pdutype>().pduType;
		transition select(pdu_type) {
			0: parse_gtpu_extheader_pdusession_info_dl;
			1: parse_gtpu_extheader_pdusession_info_ul;
			default: reject;
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl {
		packet.extract(hdr.gtpu.extDL.mandatoryHeader);
		transition select(hdr.gtpu.extDL.mandatoryHeader.qmp, hdr.gtpu.extDL.mandatoryHeader.snp, hdr.gtpu.extDL.mandatoryHeader.ppp) {
			(0, 0, 0): parse_gtpu_extheader_pdusession_info_dl_000;
			(0, 0, 1): parse_gtpu_extheader_pdusession_info_dl_001;
			(0, 1, 0): parse_gtpu_extheader_pdusession_info_dl_010;
			(0, 1, 1): parse_gtpu_extheader_pdusession_info_dl_011;
			(1, 0, 0): parse_gtpu_extheader_pdusession_info_dl_100;
			(1, 0, 1): parse_gtpu_extheader_pdusession_info_dl_101;
			(1, 1, 0): parse_gtpu_extheader_pdusession_info_dl_110;
			(1, 1, 1): parse_gtpu_extheader_pdusession_info_dl_111;
			default: reject; // unreachable
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_000 {
		packet.extract(hdr.gtpu.extDL.padding, 32w0x2);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_001 {
		packet.extract(hdr.gtpu.extDL.ppi);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x1);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_010 {
		packet.extract(hdr.gtpu.extDL.qfiSeqNum);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x3);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_011 {
		packet.extract(hdr.gtpu.extDL.ppi);
		packet.extract(hdr.gtpu.extDL.qfiSeqNum);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x2);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_100 {
		packet.extract(hdr.gtpu.extDL.qmpFields);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x2);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_101 {
		packet.extract(hdr.gtpu.extDL.ppi);
		packet.extract(hdr.gtpu.extDL.qmpFields);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x1);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_110 {
		packet.extract(hdr.gtpu.extDL.qmpFields);
		packet.extract(hdr.gtpu.extDL.qfiSeqNum);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x3);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_111 {
		packet.extract(hdr.gtpu.extDL.ppi);
		packet.extract(hdr.gtpu.extDL.qmpFields);
		packet.extract(hdr.gtpu.extDL.qfiSeqNum);
		packet.extract(hdr.gtpu.extDL.padding, 32w0x2);
		packet.extract(hdr.gtpu.extDL.nextExtHeaderType);
		transition select(hdr.gtpu.extDL.nextExtHeaderType.value) {
			0: parse_gtpu_extheader_pdusession_info_dl_finsih;
			default: reject; // no more ext header supported!
		}
	}

	state parse_gtpu_extheader_pdusession_info_dl_finsih {
		meta.tunnel_qfi = hdr.gtpu.extDL.mandatoryHeader.qfi;
		meta.tunnel_qfi_valid = 1;
		meta.ul_or_dl = 0; // DL
		transition parse_overlay;
	}

	state parse_gtpu_extheader_pdusession_info_ul {
		packet.extract(hdr.gtpu.extUL.mandatoryHeader);
		meta.tmpCounter = 8w2;
		transition select(hdr.gtpu.extUL.mandatoryHeader.qmp, hdr.gtpu.extUL.mandatoryHeader.dlDelayInd, hdr.gtpu.extUL.mandatoryHeader.ulDelayInd, hdr.gtpu.extUL.mandatoryHeader.snp, hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd) {
			(1, 0 &&& 0, 0 &&& 0, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_qmp;
			(0, 1, 0 &&& 0, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_dlDelay;
			(0, 0, 1, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_ulDelay;
			(0, 0, 0, 1, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_snp;
			(0, 0, 0, 0, 1): parse_gtpu_extheader_pdusession_info_ul_n3n9Delay;
			(0, 0, 0, 0, 0): parse_gtpu_extheader_padding;
		}
	}

	state parse_gtpu_extheader_pdusession_info_ul_qmp {
		packet.extract(hdr.gtpu.extUL.qmpFields);
		meta.tmpCounter = meta.tmpCounter + 8w24;
		transition select(hdr.gtpu.extUL.mandatoryHeader.dlDelayInd, hdr.gtpu.extUL.mandatoryHeader.ulDelayInd, hdr.gtpu.extUL.mandatoryHeader.snp, hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd) {
			(1, 0 &&& 0, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_dlDelay;
			(0, 1, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_ulDelay;
			(0, 0, 1, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_snp;
			(0, 0, 0, 1): parse_gtpu_extheader_pdusession_info_ul_n3n9Delay;
			(0, 0, 0, 0): parse_gtpu_extheader_padding;
		}
	}

	state parse_gtpu_extheader_pdusession_info_ul_dlDelay {
		packet.extract(hdr.gtpu.extUL.dlDelayResult);
		meta.tmpCounter = meta.tmpCounter + 8w4;
		transition select(hdr.gtpu.extUL.mandatoryHeader.ulDelayInd, hdr.gtpu.extUL.mandatoryHeader.snp, hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd) {
			(1, 0 &&& 0, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_ulDelay;
			(0, 1, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_snp;
			(0, 0, 1): parse_gtpu_extheader_pdusession_info_ul_n3n9Delay;
			(0, 0, 0): parse_gtpu_extheader_padding;
		}
	}

	state parse_gtpu_extheader_pdusession_info_ul_ulDelay {
		packet.extract(hdr.gtpu.extUL.ulDelayResult);
		meta.tmpCounter = meta.tmpCounter + 8w4;
		transition select(hdr.gtpu.extUL.mandatoryHeader.snp, hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd) {
			(1, 0 &&& 0): parse_gtpu_extheader_pdusession_info_ul_snp;
			(0, 1): parse_gtpu_extheader_pdusession_info_ul_n3n9Delay;
			(0, 0): parse_gtpu_extheader_padding;
		}
	}

	state parse_gtpu_extheader_pdusession_info_ul_snp {
		packet.extract(hdr.gtpu.extUL.qfiSeqNum);
		meta.tmpCounter = meta.tmpCounter + 8w3;
		transition select(hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd) {
			1: parse_gtpu_extheader_pdusession_info_ul_n3n9Delay;
			0: parse_gtpu_extheader_padding;
		}
	}

	state parse_gtpu_extheader_pdusession_info_ul_n3n9Delay {
		packet.extract(hdr.gtpu.extUL.qfiSeqNum);
		meta.tmpCounter = meta.tmpCounter + 8w4;
		transition parse_gtpu_extheader_padding;
	}

	state parse_gtpu_extheader_padding {
		bit<32> paddingLengthInOctets = (bit<32>)((8w4 - ((meta.tmpCounter + 8w2) & 8w0b11)) & 8w0b11);
		bit<32> paddingLength = 32w8 * paddingLengthInOctets;
		meta.tmpCounter = 8w0;
		packet.extract(hdr.gtpu.extUL.padding, paddingLength);
		packet.extract(hdr.gtpu.extUL.nextExtHeaderType);
		meta.tunnel_qfi = hdr.gtpu.extUL.mandatoryHeader.qfi;
		meta.tunnel_qfi_valid = 1;
		meta.ul_or_dl = 1; // UL
		transition select(hdr.gtpu.extUL.nextExtHeaderType.value) {
			0: parse_overlay;
			default: reject; // no more ext header supported!
		}
	}

	/// ----------------------------
	///    Parse Overlay Packet
	/// ----------------------------

	state parse_overlay {
		// parse ip
		transition select(hdr.ethernet.etherType) {
			0x0800: parse_overlay_ipv4;
			0x86dd: parse_overlay_ipv6;
			// No ARP for DN port
			default: reject;
		}
	}

	state parse_overlay_ipv4 {
		packet.extract(hdr.overlay_ipv4,
                    (bit<32>)
                    (8 *
                     (4 * (bit<9>) (packet.lookahead<IPv4_up_to_ihl_only_h>().ihl)
                      - 20)));
        meta.overlay_ipv4_l4len = hdr.overlay_ipv4.totalLen - (bit<16>)(hdr.overlay_ipv4.ihl) * 4;
		transition select(hdr.overlay_ipv4.protocol) {
			6: parse_overlay_tcp;
			17: parse_overlay_udp;
			default: accept;
		}
	}

	state parse_overlay_ipv6 {
		packet.extract(hdr.overlay_ipv6);
		transition select(hdr.overlay_ipv6.nextHdr) {
			6: parse_overlay_tcp;
			17: parse_overlay_udp;
			default: accept;
		}
	}

	state parse_overlay_tcp {
		packet.extract(hdr.overlay_tcp,
                    (bit<32>)
                    (8 *
                     (4 * (bit<9>) (packet.lookahead<tcp_upto_data_offset_only_h>().dataOffset)
                      - 20)));
		transition accept;
	}

	state parse_overlay_udp {
		packet.extract(hdr.overlay_udp);
		transition accept;
	}
}





control UPFIngress(inout upf_headers_t hdr,
				 inout upf_metadata_t meta,
				 inout standard_metadata_t standard_metadata){
	direct_meter<bit<2>>(MeterType.bytes) dl_meter;
	direct_meter<bit<2>>(MeterType.bytes) ul_meter;

	action gtpu_echo_response() {
		// TODO: missing Recovery IE, need PSA model to work
		// step 1: swap underlay IP src/dst
		if (hdr.underlay_ipv4.isValid()) {
			bit<32> tmp = hdr.underlay_ipv4.srcAddr;
			hdr.underlay_ipv4.srcAddr = hdr.underlay_ipv4.dstAddr;
			hdr.underlay_ipv4.dstAddr = tmp;
		}
		if (hdr.underlay_ipv6.isValid()) {
			bit<128> tmp = hdr.underlay_ipv6.srcAddr;
			hdr.underlay_ipv6.srcAddr = hdr.underlay_ipv6.dstAddr;
			hdr.underlay_ipv6.dstAddr = tmp;
		}
		// step 2: add recovery IE
		// step 3: set msgType to 2
		hdr.gtpu.mandatoryHeader.msgType = 2;
	}

	action drop() {
		mark_to_drop(standard_metadata);
	}

	action send_to_control_plane(bit<64> seid, bit<16> pdr_id) {
		hdr.controller_out.setInvalid();
		hdr.controller_in.setValid();
		hdr.controller_in.ingress_port = standard_metadata.ingress_port;
		hdr.controller_in.padding = 0;
		hdr.controller_in.flag = 1;
		hdr.controller_in.seid = seid;
		hdr.controller_in.pdr_id = pdr_id;
        standard_metadata.egress_spec = 255;
		meta.to_cp = 1;
	}

	action buffer_and_notify_cp(bit<64> seid, bit<16> pdr_id) {
		hdr.controller_out.setInvalid();
		hdr.controller_in.setValid();
		hdr.controller_in.ingress_port = standard_metadata.ingress_port;
		hdr.controller_in.padding = 0;
		hdr.controller_in.flag = 0;
		hdr.controller_in.seid = seid;
		hdr.controller_in.pdr_id = pdr_id;
        standard_metadata.egress_spec = 255;
		meta.to_cp = 1;
	}

	action set_ipv4_udp(out ipv4_t dst, out udp_t dst_udp) {
		dst.setValid();
		dst.version = 4;
		dst.ihl = 5;
		dst.diffserv = 0;
		dst.totalLen = 20;
		dst.identification = 0;
		dst.flags = 3w0b010;
		dst.fragOffset = 0;
		dst.ttl = 64;
		dst.protocol = 17;
		dst_udp.setValid();
		// we randomly select a UDP source port, no guarantee it does not repeat
		bit<16> src_port;
		random(src_port, 16w49152, 16w65535);
		dst_udp.srcPort = src_port;
	}

	action set_ipv6_udp(out ipv6_t dst, out udp_t dst_udp) {
		dst.setValid();
		dst.version = 6;
		dst.trafficClass = 0;
		dst.flowLabel = 0;
		dst.nextHdr = 17;
		dst.hopLimit = 64;
		dst_udp.setValid();
		// we randomly select a UDP source port, no guarantee it does not repeat
		bit<16> src_port;
		random(src_port, 16w49152, 16w65535);
		dst_udp.srcPort = src_port;
	}

	action create_dl_tunnel_basic() {
		hdr.gtpu.mandatoryHeader.setValid();
		hdr.gtpu.mandatoryHeader.version = 1;
		hdr.gtpu.mandatoryHeader.pt = 1;
		hdr.gtpu.mandatoryHeader.padding = 0;
		hdr.gtpu.mandatoryHeader.e = 1;
		hdr.gtpu.mandatoryHeader.s = 0;
		hdr.gtpu.mandatoryHeader.pn = 0;
		hdr.gtpu.mandatoryHeader.msgType = 255;
		hdr.gtpu.optionalHeader.setValid();
		hdr.gtpu.optionalHeader.seqNum = 0;
		hdr.gtpu.optionalHeader.nPduType = 0;
	}

	action create_dl_tunnel_basic_with_extheader() {
		hdr.gtpu.mandatoryHeader.setValid();
		hdr.gtpu.mandatoryHeader.version = 1;
		hdr.gtpu.mandatoryHeader.pt = 1;
		hdr.gtpu.mandatoryHeader.padding = 0;
		hdr.gtpu.mandatoryHeader.e = 1;
		hdr.gtpu.mandatoryHeader.s = 0;
		hdr.gtpu.mandatoryHeader.pn = 0;
		hdr.gtpu.mandatoryHeader.msgType = 255;
		hdr.gtpu.optionalHeader.setValid();
		hdr.gtpu.optionalHeader.seqNum = 0;
		hdr.gtpu.optionalHeader.nPduType = 0;
		hdr.gtpu.optionalHeader.extHeaderType = 8w0b10000101;
		//if (!hdr.gtpu.extDL.mandatoryHeader.isValid()) { // TODO: "error: Conditional execution in actions unsupported on this target", use a different
			hdr.gtpu.extDL.mandatoryHeader.setValid();
			hdr.gtpu.extDL.mandatoryHeader.pduType = 0;
			hdr.gtpu.extDL.mandatoryHeader.qmp = 0;
			hdr.gtpu.extDL.mandatoryHeader.snp = 0;
			hdr.gtpu.extDL.mandatoryHeader.spare1 = 0;
			hdr.gtpu.extDL.mandatoryHeader.ppp = 0; // TODO: maybe support paging policy diff
			hdr.gtpu.extDL.mandatoryHeader.rqi = 0; // TODO: maybe support Reflective QoS
			hdr.gtpu.extDL.nextExtHeaderType.setValid();
			hdr.gtpu.extDL.nextExtHeaderType.value = 0;
		//}
	}

	action create_ul_tunnel_basic() {
		hdr.gtpu.mandatoryHeader.setValid();
		hdr.gtpu.mandatoryHeader.version = 1;
		hdr.gtpu.mandatoryHeader.pt = 1;
		hdr.gtpu.mandatoryHeader.padding = 0;
		hdr.gtpu.mandatoryHeader.e = 1;
		hdr.gtpu.mandatoryHeader.s = 0;
		hdr.gtpu.mandatoryHeader.pn = 0;
		hdr.gtpu.mandatoryHeader.msgType = 255;
		hdr.gtpu.optionalHeader.setValid();
		hdr.gtpu.optionalHeader.seqNum = 0;
		hdr.gtpu.optionalHeader.nPduType = 0;
	}

	action create_ul_tunnel_basic_with_extheader() {
		hdr.gtpu.mandatoryHeader.setValid();
		hdr.gtpu.mandatoryHeader.version = 1;
		hdr.gtpu.mandatoryHeader.pt = 1;
		hdr.gtpu.mandatoryHeader.padding = 0;
		hdr.gtpu.mandatoryHeader.e = 1;
		hdr.gtpu.mandatoryHeader.s = 0;
		hdr.gtpu.mandatoryHeader.pn = 0;
		hdr.gtpu.mandatoryHeader.msgType = 255;
		hdr.gtpu.optionalHeader.setValid();
		hdr.gtpu.optionalHeader.seqNum = 0;
		hdr.gtpu.optionalHeader.nPduType = 0;
		hdr.gtpu.optionalHeader.extHeaderType = 8w0b10000101;
		//if (!hdr.gtpu.extUL.mandatoryHeader.isValid()) { // TODO: "error: Conditional execution in actions unsupported on this target", use a different
			hdr.gtpu.extUL.mandatoryHeader.setValid();
			hdr.gtpu.extUL.mandatoryHeader.pduType = 1;
			hdr.gtpu.extUL.mandatoryHeader.qmp = 0;
			hdr.gtpu.extUL.mandatoryHeader.dlDelayInd = 0;
			hdr.gtpu.extUL.mandatoryHeader.ulDelayInd = 0;
			hdr.gtpu.extUL.mandatoryHeader.snp = 0;
			hdr.gtpu.extUL.mandatoryHeader.n3n9DelayInd = 0;
			hdr.gtpu.extUL.mandatoryHeader.spare = 0;
			hdr.gtpu.extUL.nextExtHeaderType.setValid();
			hdr.gtpu.extUL.nextExtHeaderType.value = 0;
		//}
	}

	/// ----------------------------------
	///        GTP-U Path(N3/N9)
	/// ----------------------------------

	action remove_tunnel() {
		hdr.underlay_ipv4.setInvalid();
		hdr.underlay_ipv6.setInvalid();
		hdr.underlay_udp.setInvalid();
		hdr.gtpu.mandatoryHeader.setInvalid();
		hdr.gtpu.optionalHeader.setInvalid();
	}

	action remove_tunnel_extheaders_ul(out bit<6> qfi, out bool header_removed) {
		qfi = 0;
		header_removed = false;
		if (hdr.gtpu.extUL.mandatoryHeader.isValid()) {
			qfi = hdr.gtpu.extUL.mandatoryHeader.qfi;
			header_removed = true;
		}
		hdr.gtpu.extUL.mandatoryHeader.setInvalid();
		hdr.gtpu.extUL.qmpFields.setInvalid();
		hdr.gtpu.extUL.dlDelayResult.setInvalid();
		hdr.gtpu.extUL.ulDelayResult.setInvalid();
		hdr.gtpu.extUL.qfiSeqNum.setInvalid();
		hdr.gtpu.extUL.n3n9DeplayResult.setInvalid();
		hdr.gtpu.extUL.padding.setInvalid();
		hdr.gtpu.extUL.nextExtHeaderType.setInvalid();
	}

	action remove_tunnel_extheaders_dl(out bit<6> qfi, out bool header_removed) {
		qfi = 0;
		header_removed = false;
		if (hdr.gtpu.extDL.mandatoryHeader.isValid()) {
			qfi = hdr.gtpu.extDL.mandatoryHeader.qfi;
			header_removed = true;
		}
		hdr.gtpu.extDL.mandatoryHeader.setInvalid();
		hdr.gtpu.extDL.ppi.setInvalid();
		hdr.gtpu.extDL.qmpFields.setInvalid();
		hdr.gtpu.extDL.qfiSeqNum.setInvalid();
		hdr.gtpu.extDL.padding.setInvalid();
		hdr.gtpu.extDL.nextExtHeaderType.setInvalid();
	}

	action remove_tunnel_extheaders() {
		bit<6> qfi;
		bool hr;
		remove_tunnel_extheaders_ul(qfi, hr);
		remove_tunnel_extheaders_dl(qfi, hr);
		hdr.gtpu.optionalHeader.extHeaderType = 0;
	}

	action set_seid(bit<64> seid) {
		meta.seid = seid;
	}

	/// Set PDR_ID without OuterHeaderRemoval
	action set_pdr_id(bit<16> pdr_id) {
		meta.pdr_id = pdr_id;
	}

	/// Set PDR_ID with OuterHeaderRemoval
	action set_pdr_id_remove_tunnel(bit<16> pdr_id) {
		remove_tunnel();
		meta.pdr_id = pdr_id;
	}

	/// Set PDR_ID with OuterHeaderRemoval and Extheader Removal
	action set_pdr_id_remove_tunnel_and_extheader(bit<16> pdr_id) {
		remove_tunnel();
		remove_tunnel_extheaders();
		meta.pdr_id = pdr_id;
	}

	action send_to_access_create_gtpu_tunnel_ipv4(
		bit<1>  mark_tos,
		bit<8>  tos,
		bit<32> upf_ip,
		bit<32> gtpu_teid,
		bit<32> gtpu_ip,
		bit<1>  set_qfi,
		bit<6>  gtpu_qfi
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 0; // DL direction
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
		// get old QFI in case no manual QFI set
		bit<6> old_qfi;
		bool old_qfi_set = false;
		remove_tunnel_extheaders_ul(old_qfi, old_qfi_set);
		create_dl_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		if ((bool)set_qfi)
			hdr.gtpu.extDL.mandatoryHeader.qfi = gtpu_qfi;
		else if (old_qfi_set)
			hdr.gtpu.extDL.mandatoryHeader.qfi = old_qfi;
		set_ipv4_udp(hdr.underlay_ipv4, hdr.underlay_udp);
		hdr.underlay_ipv4.srcAddr = upf_ip;
		hdr.underlay_ipv4.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	action send_to_access_create_gtpu_tunnel_ipv6(
		bit<1>    mark_tos,
		bit<8>    tos,
		bit<128>  upf_ip,
		bit<32>   gtpu_teid,
		bit<128>  gtpu_ip,
		bit<1>    set_qfi,
		bit<6>    gtpu_qfi
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 0; // DL direction
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
		bit<6> old_qfi;
		bool old_qfi_set = false;
		remove_tunnel_extheaders_ul(old_qfi, old_qfi_set);
		create_dl_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		if ((bool)set_qfi)
			hdr.gtpu.extDL.mandatoryHeader.qfi = gtpu_qfi;
		else if (old_qfi_set)
			hdr.gtpu.extDL.mandatoryHeader.qfi = old_qfi;
		set_ipv6_udp(hdr.underlay_ipv6, hdr.underlay_udp);
		hdr.underlay_ipv6.srcAddr = upf_ip;
		hdr.underlay_ipv6.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	action send_to_core_create_gtpu_tunnel_ipv4(
		bit<1>  mark_tos,
		bit<8>  tos,
		bit<32> upf_ip,
		bit<32> gtpu_teid,
		bit<32> gtpu_ip,
		bit<1>  set_qfi,
		bit<6>  gtpu_qfi
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 1; // UL direction
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
		bit<6> old_qfi;
		bool old_qfi_set = false;
		remove_tunnel_extheaders_dl(old_qfi, old_qfi_set);
		create_ul_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		if ((bool)set_qfi)
			hdr.gtpu.extUL.mandatoryHeader.qfi = gtpu_qfi;
		else if (old_qfi_set)
			hdr.gtpu.extUL.mandatoryHeader.qfi = old_qfi;
		set_ipv4_udp(hdr.underlay_ipv4, hdr.underlay_udp);
		hdr.underlay_ipv4.srcAddr = upf_ip;
		hdr.underlay_ipv4.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	action send_to_core_create_gtpu_tunnel_ipv6(
		bit<1>   mark_tos,
		bit<8>   tos,
		bit<128> upf_ip,
		bit<32>  gtpu_teid,
		bit<128> gtpu_ip,
		bit<1>   set_qfi,
		bit<6>   gtpu_qfi
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 1; // UL direction
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
		bit<6> old_qfi;
		bool old_qfi_set = false;
		remove_tunnel_extheaders_dl(old_qfi, old_qfi_set);
		create_ul_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		if ((bool)set_qfi)
			hdr.gtpu.extUL.mandatoryHeader.qfi = gtpu_qfi;
		else if (old_qfi_set)
			hdr.gtpu.extUL.mandatoryHeader.qfi = old_qfi;
		set_ipv6_udp(hdr.underlay_ipv6, hdr.underlay_udp);
		hdr.underlay_ipv6.srcAddr = upf_ip;
		hdr.underlay_ipv6.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	action send_to_access(
		bit<1> mark_tos,
		bit<8> tos
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 0; // DL direction
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
	}

	action send_to_core(
		bit<1> mark_tos,
		bit<8> tos
	) {
		ul_meter.read(meta.meter_result);
		meta.ul_or_dl = 1; // UL direction
		meta.to_n6 = 1; // to DN
		remove_tunnel_extheaders();
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos;
			}
		}
	}

	table GTP_Action {
		key = {
			// I don't know why but if you use exact match here it will always fail to match anything, so I have to use ternary with all ones mask
			meta.seid   : ternary;
			meta.pdr_id : ternary;
		}
		actions = {
			send_to_access;
			send_to_access_create_gtpu_tunnel_ipv4;
			send_to_core_create_gtpu_tunnel_ipv4;
			send_to_access_create_gtpu_tunnel_ipv6;
			send_to_core_create_gtpu_tunnel_ipv6;
			send_to_core;
			buffer_and_notify_cp;
			send_to_control_plane;
			drop;
		}
		const default_action = drop;
		meters = ul_meter;
		size = 1024;
	}

	table GTP_Overlay_IPv4_UDP {
		key = {
			meta.tunnel_teid          : ternary;
			meta.tunnel_qfi           : ternary;
			meta.seid                 : exact;
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_udp.srcPort   : range;
			hdr.overlay_udp.dstPort   : range;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table GTP_Overlay_IPv4_TCP {
		key = {
			meta.tunnel_teid          : ternary;
			meta.tunnel_qfi           : ternary;
			meta.seid                 : exact;
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_tcp.srcPort   : range;
			hdr.overlay_tcp.dstPort   : range;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table GTP_Overlay_IPv4_Default {
		key = {
			meta.tunnel_teid          : ternary;
			meta.tunnel_qfi           : ternary;
			meta.seid                 : exact;
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_ipv4.protocol : ternary;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}


	table GTP_Overlay_IPv6_UDP {
		key = {
			meta.tunnel_teid              : ternary;
			meta.tunnel_qfi               : ternary;
			meta.seid                     : exact;
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_udp.srcPort       : range;
			hdr.overlay_udp.dstPort       : range;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table GTP_Overlay_IPv6_TCP {
		key = {
			meta.tunnel_teid              : ternary;
			meta.tunnel_qfi               : ternary;
			meta.seid                     : exact;
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_tcp.srcPort       : range;
			hdr.overlay_tcp.dstPort       : range;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table GTP_Overlay_IPv6_Default {
		key = {
			meta.tunnel_teid              : ternary;
			meta.tunnel_qfi               : ternary;
			meta.seid                     : exact;
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_ipv6.nextHdr      : ternary;
		}
		actions = {
			set_pdr_id;
			set_pdr_id_remove_tunnel;
			set_pdr_id_remove_tunnel_and_extheader;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}


	table GTP_IPv4 {
		key = {
			meta.tunnel_ipv4_addr : ternary;
			meta.tunnel_teid      : exact;
		}
		actions = {
			set_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table GTP_IPv6 {
		key = {
			meta.tunnel_ipv6_addr : ternary;
			meta.tunnel_teid      : exact;
		}
		actions = {
			set_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	/// ----------------------------------
	///           DN Path(N6)
	/// ----------------------------------

	action create_gtpu_header_dl_ipv4(
		bit<1>  mark_tos,
		bit<8>  tos_value,
		bit<32> upf_ip,
		bit<32> gtpu_teid,
		bit<32> gtpu_ip,
		bit<6>  gtpu_qfi
	) {
		dl_meter.read(meta.meter_result);
		create_dl_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		hdr.gtpu.extDL.mandatoryHeader.qfi = gtpu_qfi;
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos_value;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos_value;
			}
		}
		set_ipv4_udp(hdr.underlay_ipv4, hdr.underlay_udp);
		hdr.underlay_ipv4.srcAddr = upf_ip;
		hdr.underlay_ipv4.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	action create_gtpu_header_dl_ipv6(
		bit<1>   mark_tos,
		bit<8>   tos_value,
		bit<128> upf_ip,
		bit<32>  gtpu_teid,
		bit<128> gtpu_ip,
		bit<6>   gtpu_qfi
	) {
		dl_meter.read(meta.meter_result);
		create_dl_tunnel_basic_with_extheader();
		hdr.gtpu.mandatoryHeader.teid = gtpu_teid;
		hdr.gtpu.extDL.mandatoryHeader.qfi = gtpu_qfi;
		if ((bool)mark_tos) {
			if (hdr.overlay_ipv4.isValid()) {
				hdr.overlay_ipv4.diffserv = tos_value;
			} else if (hdr.overlay_ipv6.isValid()) {
				hdr.overlay_ipv6.trafficClass = tos_value;
			}
		}
		set_ipv6_udp(hdr.underlay_ipv6, hdr.underlay_udp);
		hdr.underlay_ipv6.srcAddr = upf_ip;
		hdr.underlay_ipv6.dstAddr = gtpu_ip;
		hdr.underlay_udp.dstPort = 2152;
	}

	/// Set both SEID and PDR_ID for later to be matched to N6 Action
	action set_pdr_id_and_seid(bit<64> seid, bit<16> pdr_id) {
		meta.seid = seid;
		meta.pdr_id = pdr_id;
	}

	table N6_Action {
		key = {
			// I don't know why but if you use exact match here it will always fail to match anything, so I have to use ternary with all ones mask
			meta.seid   : ternary;
			meta.pdr_id : ternary;
		}
		actions = {
			create_gtpu_header_dl_ipv4;
			create_gtpu_header_dl_ipv6;
			buffer_and_notify_cp;
			send_to_control_plane;
			drop;
		}
		const default_action = drop;
		meters = dl_meter;
		size = 1024;
	}

	table N6_IPv4_UDP {
		key = {
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_udp.srcPort   : range;
			hdr.overlay_udp.dstPort   : range;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table N6_IPv4_TCP {
		key = {
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_tcp.srcPort   : range;
			hdr.overlay_tcp.dstPort   : range;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table N6_IPv4_Default {
		key = {
			hdr.overlay_ipv4.diffserv : ternary;
			hdr.overlay_ipv4.srcAddr  : ternary;
			hdr.overlay_ipv4.dstAddr  : ternary;
			hdr.overlay_ipv4.protocol : ternary;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table N6_IPv6_UDP {
		key = {
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_udp.srcPort       : range;
			hdr.overlay_udp.dstPort       : range;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table N6_IPv6_TCP {
		key = {
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_tcp.srcPort       : range;
			hdr.overlay_tcp.dstPort       : range;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	table N6_IPv6_Default {
		key = {
			hdr.overlay_ipv6.trafficClass : ternary;
			hdr.overlay_ipv6.srcAddr      : ternary;
			hdr.overlay_ipv6.dstAddr      : ternary;
			hdr.overlay_ipv6.nextHdr      : ternary;
		}
		actions = {
			set_pdr_id_and_seid;
			drop;
		}
		const default_action = drop;
		size = 1024;
	}

	apply {
		// TODO: learn MAC first
		if (hdr.controller_out.isValid()) {
			if (hdr.controller_out.flag == 8w1) {
				// End Marker
				meta.bypass = 1;
				hdr.controller_out.setInvalid();
				standard_metadata.egress_spec = 0;
				return;
			}
			hdr.controller_out.setInvalid();
		}
		if ((bool)meta.tunnel_valid) {
			// if (hdr.gtpu.mandatoryHeader.msgType == 255) {
			// 	standard_metadata.egress_spec = 1;
			// 	return;
			// }
			if (hdr.gtpu.mandatoryHeader.msgType == 1) {
				// Echo Request
				// send response
				gtpu_echo_response();
				return;
			}
			if (hdr.gtpu.mandatoryHeader.msgType == 2) {
				// Echo Response
				// do nothing
				return;
			}
			if (hdr.gtpu.mandatoryHeader.msgType == 253) {
				// Tunnel Status
				// do nothing
				return;
			}
			if (hdr.gtpu.mandatoryHeader.msgType == 254) {
				// End Marker
				// do nothing
				return;
			}
			// handle N3/N9
			if (hdr.underlay_ipv4.isValid()) {
				// match and set SEID
				if (GTP_IPv4.apply().hit) {
					bool overlay_match_found = false;
					// match and set PDR_ID
					if (hdr.overlay_udp.isValid()) {
						// IPv4_UDP
						overlay_match_found = GTP_Overlay_IPv4_UDP.apply().hit;
					} else if (hdr.overlay_tcp.isValid()) {
						// IPv4_TCP
						overlay_match_found = GTP_Overlay_IPv4_TCP.apply().hit;
					} else {
						// IPv4_Default
						overlay_match_found = GTP_Overlay_IPv4_Default.apply().hit;
					}
					if (overlay_match_found) {
						// match SEID and PDR_ID to actions
						GTP_Action.apply();
					}
				}
			} else if (hdr.underlay_ipv6.isValid()) {
				// match and set SEID
				if (GTP_IPv6.apply().hit) {
					bool overlay_match_found = false;
					// match and set PDR_ID
					if (hdr.overlay_udp.isValid()) {
						// IPv6_UDP
						overlay_match_found = GTP_Overlay_IPv6_UDP.apply().hit;
					} else if (hdr.overlay_tcp.isValid()) {
						// IPv6_TCP
						overlay_match_found = GTP_Overlay_IPv6_TCP.apply().hit;
					} else {
						// IPv6_Default
						overlay_match_found = GTP_Overlay_IPv6_Default.apply().hit;
					}
					if (overlay_match_found) {
						// match SEID and PDR_ID to actions
						GTP_Action.apply();
					}
				}
			}
		} else {
			bool n6hit = false;
			// handle N6
			if (hdr.overlay_ipv4.isValid()) {
				// dispatch to IPv4
				if (hdr.overlay_udp.isValid()) {
					// IPv4_UDP
					n6hit = n6hit || N6_IPv4_UDP.apply().hit;
				} else if (hdr.overlay_tcp.isValid()) {
					// IPv4_TCP
					n6hit = n6hit || N6_IPv4_TCP.apply().hit;
				} else {
					// IPv4_Default
					n6hit = n6hit || N6_IPv4_Default.apply().hit;
				}
			} else {
				// dispatch to IPv6
				if (hdr.overlay_udp.isValid()) {
					// IPv6_UDP
					n6hit = n6hit || N6_IPv6_UDP.apply().hit;
				} else if (hdr.overlay_tcp.isValid()) {
					// IPv6_TCP
					n6hit = n6hit || N6_IPv6_TCP.apply().hit;
				} else {
					// IPv6_Default
					n6hit = n6hit || N6_IPv6_Default.apply().hit;
				}
			}
			if (n6hit) {
				N6_Action.apply();
			}
		}
		if (meta.meter_result != 0) {
			drop();
			return;
		}
		if (!((bool)meta.to_cp)) {
			if ((bool)meta.to_n6) {
				standard_metadata.egress_spec = 1;
			} else {
				standard_metadata.egress_spec = 0;
			}
		}
	}
}

control UPFEgress(inout upf_headers_t hdr,
				 inout upf_metadata_t meta,
				 inout standard_metadata_t standard_metadata){
	// NOTE: any packets marked for drop should not reach here
	apply {
		if ((bool)meta.bypass) {
			return;
		}
		// calculate header lengths and set padding
		bit<16> overlay_length = 0; // total length of overlay packet, including IP packet header and content
		if (hdr.overlay_ipv4.isValid()) {
			overlay_length = hdr.overlay_ipv4.totalLen;
		} else if (hdr.overlay_ipv6.isValid()) {
			overlay_length = hdr.overlay_ipv6.payloadLen + 16 + 16 + 2;
		}
		bit<16> gtpu_length = 0; // length of GTP-U header, including mandatory and extheaders
		if (hdr.gtpu.mandatoryHeader.isValid()) {
			gtpu_length = 8;
			if (hdr.gtpu.optionalHeader.isValid()) {
				gtpu_length = gtpu_length + 4;
			}
			if (hdr.gtpu.extUL.mandatoryHeader.isValid()) {
				hdr.gtpu.extUL.padding.setInvalid();
				bit<16> ext_ul_len = 4; // mandatory header(3), next ext header(1)
				if (hdr.gtpu.extUL.qmpFields.isValid()) {
					ext_ul_len = ext_ul_len + 24;
				}
				if (hdr.gtpu.extUL.dlDelayResult.isValid()) {
					ext_ul_len = ext_ul_len + 4;
				}
				if (hdr.gtpu.extUL.ulDelayResult.isValid()) {
					ext_ul_len = ext_ul_len + 4;
				}
				if (hdr.gtpu.extUL.qfiSeqNum.isValid()) {
					ext_ul_len = ext_ul_len + 3;
				}
				if (hdr.gtpu.extUL.n3n9DeplayResult.isValid()) {
					ext_ul_len = ext_ul_len + 4;
				}
				bit<16> div = ext_ul_len >> 2; // divide by 4
				bit<2>  rem = (bit<2>)(ext_ul_len & 16w0b11); // remainder of divide by 4
				hdr.gtpu.extUL.mandatoryHeader.lengthIn4octets = (bit<8>)div;
				if (rem == 1) {
					hdr.gtpu.extUL.outpad_3.setValid();
				} else if (rem == 2) {
					hdr.gtpu.extUL.outpad_2.setValid();
				} else if (rem == 3) {
					hdr.gtpu.extUL.outpad_1.setValid();
				}
				gtpu_length = gtpu_length + ext_ul_len;
			}
			if (hdr.gtpu.extDL.mandatoryHeader.isValid()) {
				hdr.gtpu.extDL.padding.setInvalid();
				bit<16> ext_dl_len = 4; // mandatory header(3), next ext header(1)
				if (hdr.gtpu.extDL.ppi.isValid()) {
					ext_dl_len = ext_dl_len + 1;
				}
				if (hdr.gtpu.extDL.qmpFields.isValid()) {
					ext_dl_len = ext_dl_len + 8;
				}
				if (hdr.gtpu.extDL.qfiSeqNum.isValid()) {
					ext_dl_len = ext_dl_len + 3;
				}
				bit<16> div = ext_dl_len >> 2; // divide by 4
				bit<2>  rem = (bit<2>)(ext_dl_len & 16w0b11); // remainder of divide by 4
				hdr.gtpu.extDL.mandatoryHeader.lengthIn4octets = (bit<8>)div;
				if (rem == 1) {
					hdr.gtpu.extDL.outpad_3.setValid();
				} else if (rem == 2) {
					hdr.gtpu.extDL.outpad_2.setValid();
				} else if (rem == 3) {
					hdr.gtpu.extDL.outpad_1.setValid();
				}
				gtpu_length = gtpu_length + ext_dl_len;
			}
			bit<16> gtpu_payload_length = gtpu_length - 8;
			hdr.gtpu.mandatoryHeader.len = gtpu_payload_length + overlay_length;
		}
		if (hdr.underlay_udp.isValid()) {
			hdr.underlay_udp.len = 8 + overlay_length + gtpu_length;
		}
		if (hdr.underlay_ipv4.isValid()) {
			hdr.underlay_ipv4.totalLen = 20 + 8 + overlay_length + gtpu_length;
		} else if (hdr.overlay_ipv6.isValid()) {
			hdr.underlay_ipv6.payloadLen = 8 + overlay_length + gtpu_length;
		}
		// TODO: set dst MAC
	}
}

control EthDeparser(packet_out packet, in upf_headers_t hdr){
	apply{
		packet.emit(hdr);
	}
}


control UPFComputeChecksum(inout upf_headers_t hdr, inout upf_metadata_t meta){
	
	apply {
		// step 1: overlay IPv4 checksum (IPv6 does not have checksum)
		update_checksum(hdr.overlay_ipv4.isValid(),
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
				hdr.overlay_ipv4.dstAddr,
				hdr.overlay_ipv4.options
			},
			hdr.overlay_ipv4.hdrChecksum, HashAlgorithm.csum16
		);
		// step 2: overlay TCP/UDP checksum
		// not necessary since we did not change thier data
		// step 3: underlay udp checksum
		// this contains so many combinations of GTP-U tunnel data, and since
		// it is optional for IPv4, we will just leave it as all zeros
		// meaning incorrect checksum when using IPv6 underlay
		
#if 0 // do not destroy a potential valid checksum
		update_checksum(hdr.underlay_udp.isValid(), { 16w0 }, hdr.underlay_udp.checksum, HashAlgorithm.csum16);
#endif

		// step 4: underlay IPv4 checksum (IPv6 does not have checksum)
		update_checksum(hdr.underlay_ipv4.isValid(),
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
				hdr.underlay_ipv4.dstAddr,
				hdr.underlay_ipv4.options
			},
			hdr.underlay_ipv4.hdrChecksum, HashAlgorithm.csum16
		);
	}
}

control UPFVerifyChecksum(inout upf_headers_t hdr, inout upf_metadata_t meta){
	apply{
	}
}


V1Switch(
	EthParser(),
	UPFVerifyChecksum(),
	UPFIngress(),
	UPFEgress(),
	UPFComputeChecksum(),
	EthDeparser()
) main;
