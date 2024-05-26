
#include <core.p4>
#include <v1model.p4>

#define MAX_CLIENTS 1024
#define MAX_CONN_PER_CLIENT 256

typedef bit<48>  EthernetAddress;
typedef bit<32>  IPv4Address;

header ethernet_t {
    EthernetAddress dstAddr;
    EthernetAddress srcAddr;
    bit<16>         etherType;
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
    IPv4Address  srcAddr;
    IPv4Address  dstAddr;
    varbit<320>  options;
}

header IPv4_up_to_ihl_only_h {
    bit<4> version;
    bit<4> ihl;
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

header udp_t {
    bit<16> srcPort;
    bit<16> dstPort;
    bit<16> length_;
    bit<16> checksum;
}

header icmp_t {
    bit<16> typeCode;
    bit<16> hdrChecksum;
    bit<16> identifier;
    bit<16> seqNo;
}

@controller_header("packet_in")
header packet_in_t {
    IPv4Address srcAddr;
    bit<16>     srcPort;
    IPv4Address dstAddr;
    bit<16>     dstPort;
    bit<16>     icmpId;
    bit<8>      protocol; // 0x6 for TCP, 0x11 for UDP, 0x1 for ICMP
}

@controller_header("packet_out")
header packet_out_t {
}

struct headers {
	packet_in_t      controller_in;
	packet_out_t     controller_out;

    ethernet_t       ethernet;
    ipv4_t           ipv4;
    tcp_t            tcp;
    udp_t            udp;
    icmp_t           icmp;
}

struct metadata {
    bit<16>     l4Len;
    bit<1>      direction; // 0 for DL, 1 for UL
    bit<16>     srcPort;
    bit<16>     dstPort;
    bit<16>     icmpId;
}

parser ParserImpl(
    packet_in packet,
    out headers hdr,
    inout metadata meta,
    inout standard_metadata_t standard_metadata
) {
    const bit<16> ETHERTYPE_IPV4 = 0x0800;

    state start {
        meta.direction = 0; // DL
        if (standard_metadata.ingress_port == 0 || standard_metadata.ingress_port == 255) {
            meta.direction = 1; // UL
        }
        transition parse_ethernet;
    }
    state parse_ethernet {
        packet.extract(hdr.ethernet);
        transition select(hdr.ethernet.etherType) {
            ETHERTYPE_IPV4: parse_ipv4;
            default: accept;
        }
    }
    state parse_ipv4 {
        packet.extract(hdr.ipv4,
                    (bit<32>)
                    (8 *
                     (4 * (bit<9>) (packet.lookahead<IPv4_up_to_ihl_only_h>().ihl)
                      - 20)));
        meta.l4Len = hdr.ipv4.totalLen - (bit<16>)(hdr.ipv4.ihl) * 4;
        transition select(hdr.ipv4.protocol) {
            8w0x1:   parse_icmp;
            8w0x6:   parse_tcp;
            8w0x11:  parse_udp;
            default: accept;
        }
    }
    state parse_icmp {
        packet.extract<icmp_t>(hdr.icmp);
        meta.icmpId = hdr.icmp.identifier;
        transition accept;
    }
    state parse_tcp {
        packet.extract(hdr.tcp,
                    (bit<32>)
                    (8 *
                     (4 * (bit<9>) (packet.lookahead<tcp_upto_data_offset_only_h>().dataOffset)
                      - 20)));
        meta.srcPort = hdr.tcp.srcPort;
        meta.dstPort = hdr.tcp.dstPort;
        transition accept;
    }
    state parse_udp {
        packet.extract<udp_t>(hdr.udp);
        meta.srcPort = hdr.udp.srcPort;
        meta.dstPort = hdr.udp.dstPort;
        transition accept;
    }
}

bit<16> cadd(in bit<16> a, in bit<16> b) {
    bit<32> aa = (bit<32>)a;
    bit<32> bb = (bit<32>)b;
    bit<32> cc = aa + bb;
    bit<32> ret = (cc & 0xffff) + (cc >> 16);
    return (bit<16>)ret;
}

bit<16> uhc(in bit<16> ohc, in bit<16> om, in bit<16> nm) {
    return ~(cadd(cadd((~ohc), (~om)), nm));
}

control ingress(
    inout headers hdr,
    inout metadata meta,
    inout standard_metadata_t standard_metadata
) {

    direct_counter(CounterType.packets) outgoing_flow_counter;
    direct_counter(CounterType.packets) incoming_flow_counter;


    action drop() {
        mark_to_drop(standard_metadata);
    }

    action rewrite_tcp(IPv4Address newSrcAddr, bit<16> newSrcPort) {
        outgoing_flow_counter.count();
        hdr.ipv4.srcAddr = newSrcAddr;
        hdr.tcp.srcPort = newSrcPort;
        //hdr.tcp.checksum = 16w0;
    }

    action rewrite_udp(IPv4Address newSrcAddr, bit<16> newSrcPort) {
        outgoing_flow_counter.count();
        hdr.ipv4.srcAddr = newSrcAddr;
        hdr.udp.srcPort = newSrcPort;
        //hdr.udp.checksum = 16w0;
    }

    action rewrite_icmp(IPv4Address newSrcAddr, bit<16> newIcmpId) {
        outgoing_flow_counter.count();
        hdr.ipv4.srcAddr = newSrcAddr;
        hdr.icmp.identifier = newIcmpId;
        //hdr.icmp.hdrChecksum = 16w0;
    }

    action rewrite_tcp_dst(IPv4Address newDstAddr, bit<16> newDstPort) {
        incoming_flow_counter.count();
        bit<16> om1 = hdr.ipv4.dstAddr[15:0];
        bit<16> om2 = hdr.ipv4.dstAddr[31:16];
        bit<16> om3 = hdr.tcp.dstPort;
        bit<16> nm1 = newDstAddr[15:0];
        bit<16> nm2 = newDstAddr[31:16];
        bit<16> nm3 = newDstPort;
        hdr.ipv4.dstAddr = newDstAddr;
        hdr.tcp.dstPort = newDstPort;
        hdr.tcp.checksum = uhc(hdr.tcp.checksum, om1, nm1);
        hdr.tcp.checksum = uhc(hdr.tcp.checksum, om2, nm2);
        hdr.tcp.checksum = uhc(hdr.tcp.checksum, om3, nm3);
    }

    action rewrite_udp_dst(IPv4Address newDstAddr, bit<16> newDstPort) {
        incoming_flow_counter.count();
        bit<16> om1 = hdr.ipv4.dstAddr[15:0];
        bit<16> om2 = hdr.ipv4.dstAddr[31:16];
        bit<16> om3 = hdr.udp.dstPort;
        bit<16> nm1 = newDstAddr[15:0];
        bit<16> nm2 = newDstAddr[31:16];
        bit<16> nm3 = newDstPort;
        hdr.ipv4.dstAddr = newDstAddr;
        hdr.udp.dstPort = newDstPort;
        hdr.udp.checksum = uhc(hdr.udp.checksum, om1, nm1);
        hdr.udp.checksum = uhc(hdr.udp.checksum, om2, nm2);
        hdr.udp.checksum = uhc(hdr.udp.checksum, om3, nm3);
    }

    action rewrite_icmp_dst(IPv4Address newDstAddr, bit<16> newIcmpId) {
        incoming_flow_counter.count();
        bit<16> om3 = hdr.icmp.identifier;
        bit<16> nm3 = newIcmpId;
        hdr.ipv4.dstAddr = newDstAddr;
        hdr.icmp.identifier = newIcmpId;
        hdr.icmp.hdrChecksum = uhc(hdr.icmp.hdrChecksum, om3, nm3);
    }

    table outgoing_flow {
        key = {
            hdr.ipv4.srcAddr   :  ternary;
            hdr.ipv4.dstAddr   :  ternary;
            meta.srcPort       :  ternary;
            meta.dstPort       :  ternary;
            meta.icmpId        :  ternary;
            hdr.ipv4.protocol  :  exact;
        }
        actions = {
            NoAction;
            rewrite_tcp;
            rewrite_udp;
            rewrite_icmp;
        }
        default_action = NoAction();
        counters = outgoing_flow_counter;
    }

    table incoming_flow {
        key = {
            hdr.ipv4.srcAddr   :  ternary;
            hdr.ipv4.dstAddr   :  ternary;
            meta.srcPort       :  ternary;
            meta.dstPort       :  ternary;
            meta.icmpId        :  ternary;
            hdr.ipv4.protocol  :  exact;
        }
        actions = {
            NoAction;
            rewrite_tcp_dst;
            rewrite_udp_dst;
            rewrite_icmp_dst;
        }
        default_action = NoAction();
        counters = incoming_flow_counter;
    }

    apply {
        if (!hdr.ipv4.isValid()) {
            return;
        }
        if (meta.direction == 1) {
            standard_metadata.egress_spec = 1;
            // outgoing
            if (outgoing_flow.apply().hit) {
                // send to port 1(public)
                standard_metadata.egress_spec = 1;
            } else {
                // send to CP for new flow allocation
                standard_metadata.egress_spec = 255;
                hdr.controller_in.setValid();
                hdr.controller_in.srcAddr = hdr.ipv4.srcAddr;
                hdr.controller_in.dstAddr = hdr.ipv4.dstAddr;
                if (hdr.tcp.isValid()) {
                    hdr.controller_in.srcPort = hdr.tcp.srcPort;
                    hdr.controller_in.dstPort = hdr.tcp.dstPort;
                    hdr.controller_in.protocol = 0x6;
                } else if (hdr.udp.isValid()) {
                    hdr.controller_in.srcPort = hdr.udp.srcPort;
                    hdr.controller_in.dstPort = hdr.udp.dstPort;
                    hdr.controller_in.protocol = 0x11;
                } else if (hdr.icmp.isValid()) {
                    hdr.controller_in.icmpId = hdr.icmp.identifier;
                    hdr.controller_in.protocol = 0x1;
                } else {
                    drop();
                }
            }
        } else {
            hdr.controller_in.setInvalid();
            hdr.controller_out.setInvalid();
            standard_metadata.egress_spec = 0;
            // incoming
            if (incoming_flow.apply().hit) {
                // send to port 0 (private)
                ;
            } else {
                // flow initiated from outside
                drop();
            }
        }
    }
}


control egress(inout headers hdr,
               inout metadata meta,
               inout standard_metadata_t standard_metadata)
{
    apply {
    }
}

control DeparserImpl(packet_out packet, in headers hdr) {
    apply {
        packet.emit(hdr);
    }
}

control verifyChecksum(inout headers hdr, inout metadata meta) {
    apply {
    }
}

control computeChecksum(inout headers hdr, inout metadata meta) {
    apply {
        update_checksum(
            hdr.ipv4.isValid(),
            {
                hdr.ipv4.version,
                hdr.ipv4.ihl,
                hdr.ipv4.diffserv,
                hdr.ipv4.totalLen,
                hdr.ipv4.identification,
                hdr.ipv4.flags,
                hdr.ipv4.fragOffset,
                hdr.ipv4.ttl,
                hdr.ipv4.protocol,
                hdr.ipv4.srcAddr,
                hdr.ipv4.dstAddr,
                hdr.ipv4.options
            },
            hdr.ipv4.hdrChecksum,
            HashAlgorithm.csum16
        );
        update_checksum_with_payload(
            hdr.tcp.isValid() && hdr.ipv4.isValid() && meta.direction == 1, // non IPv4 packets are not modified
            {
                hdr.ipv4.srcAddr,
                hdr.ipv4.dstAddr,
                8w0,
                hdr.ipv4.protocol,
                meta.l4Len,
                hdr.tcp.srcPort,
                hdr.tcp.dstPort,
                hdr.tcp.seqNo,
                hdr.tcp.ackNo,
                hdr.tcp.dataOffset,
                hdr.tcp.res,
                hdr.tcp.ecn,
                hdr.tcp.ctrl,
                hdr.tcp.window,
                hdr.tcp.urgentPtr,
                hdr.tcp.options
            },
            hdr.tcp.checksum,
            HashAlgorithm.csum16
        );
        update_checksum_with_payload(
            hdr.udp.isValid() && hdr.ipv4.isValid() && meta.direction == 1, // non IPv4 packets are not modified
            {
                hdr.ipv4.srcAddr,
                hdr.ipv4.dstAddr,
                8w0,
                hdr.ipv4.protocol,
                meta.l4Len,
                hdr.udp.srcPort,
                hdr.udp.dstPort,
                hdr.udp.length_
            },
            hdr.udp.checksum,
            HashAlgorithm.csum16
        );
        update_checksum_with_payload(
            hdr.icmp.isValid() && hdr.ipv4.isValid() && meta.direction == 1, // non IPv4 packets are not modified
            {
                hdr.icmp.typeCode,
                hdr.icmp.identifier,
                hdr.icmp.seqNo
            },
            hdr.icmp.hdrChecksum,
            HashAlgorithm.csum16
        );
    }
}

V1Switch(
    ParserImpl(),
    verifyChecksum(),
    ingress(),
    egress(),
    computeChecksum(),
    DeparserImpl()
) main;
