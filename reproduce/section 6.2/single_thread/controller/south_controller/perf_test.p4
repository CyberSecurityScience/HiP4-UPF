#include <core.p4>
#include <tna.p4>

typedef bit<48>                       mac_addr_t;

header ethernet_h {
	mac_addr_t dst;
	mac_addr_t src;
	bit<16> etherType;
}

struct ig_header_t {
    ethernet_h ethernet;
}

struct ig_metadata_t {
}

struct eg_header_t {
    ethernet_h ethernet;
}

struct eg_metadata_t {
}

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
		transition parse_ethernet;
	}

	state parse_ethernet {
		pkt.extract(hdr.ethernet);
		transition accept;
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
    Counter<bit<36>, bit<24>>(32, CounterType_t.PACKETS_AND_BYTES) counter1;
    Counter<bit<36>, bit<24>>(32, CounterType_t.PACKETS_AND_BYTES) counter2;
    Counter<bit<36>, bit<24>>(32, CounterType_t.PACKETS_AND_BYTES) counter3;
    Counter<bit<36>, bit<24>>(32, CounterType_t.PACKETS_AND_BYTES) counter4;

    bit<24> idx;
    action set_idx() {
        idx = hdr.ethernet.dst[23:0];
    }

	apply {
        if (hdr.ethernet.isValid()) {
            set_idx();
            counter1.count(idx);
            counter2.count(idx);
            counter3.count(idx);
            counter4.count(idx);
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
	apply {
		pkt.emit(hdr);
	}
}

// ------------------------------------------------------------------------------------------
//                                      EGRESS STAGE
// ------------------------------------------------------------------------------------------

parser EgressParser(
			packet_in                                   pkt,
	/* User */
	out     eg_header_t                                 hdr,
	out     eg_metadata_t                               meta,
	/* Intrinsic */
	out     egress_intrinsic_metadata_t                 eg_intr_md
)
{	
	/* This is a mandatory state, required by Tofino Architecture */
	state start {
		pkt.extract(eg_intr_md);
		transition parse_ethernet;
	}

	state parse_ethernet {
		pkt.extract(hdr.ethernet);
		transition accept;
	}
}


// --------------------------------------------------
//                  Egress Control
// --------------------------------------------------
control Egress(
	/* User */
	inout   eg_header_t                                 hdr,
	inout   eg_metadata_t                               meta,
	/* Intrinsic */
	in      egress_intrinsic_metadata_t                 eg_intr_md,
	in      egress_intrinsic_metadata_from_parser_t     eg_intr_md_from_prsr,
	inout   egress_intrinsic_metadata_for_deparser_t    eg_intr_dprs_md,
	inout   egress_intrinsic_metadata_for_output_port_t eg_intr_oport_md
) {

	apply {
	}
}


control EgressDeparser(
			packet_out                                  pkt,
	/* User */
	inout   eg_header_t                                 hdr,
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
