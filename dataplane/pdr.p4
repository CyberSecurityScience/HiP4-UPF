

control PDR_single_stage(
	/* User */
	inout   ig_header_t                                 hdr,
	inout   ig_metadata_t                               meta,
	in bit<(MA_ID_BITS - COMPRESSED_QFI_BITS)>          extracted_teid
) {
	bit<COMPRESSED_QFI_BITS> compressed_qfi = 0;

    action set_ma_id_and_tunnel_dl_N6_simple_ipv4(ma_id_t ma_id_v, teid_t teid_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
        meta.teid = (teid_t)teid_v;
		meta.qer_id = qer_id;
	}
	
	table dl_N6_simple_ipv4 {
		key = {
			hdr.overlay_ipv4.dstAddr : exact;
		}
		actions = {
			set_ma_id_and_tunnel_dl_N6_simple_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_DL_N6_SIMPLE_IPV4;
	}

    action set_ma_id_and_tunnel_dl_N6_complex_ipv4(ma_id_t ma_id_v, teid_t teid_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
        meta.teid = (teid_t)teid_v;
		meta.qer_id = qer_id;
	}
	
	table dl_N6_complex_ipv4 { // require 5 TCAM join
		key = {
			hdr.overlay_ipv4.dstAddr    : ternary;
			hdr.overlay_ipv4.srcAddr    : ternary;
			hdr.overlay_ipv4.protocol   : ternary;
			hdr.overlay_ipv4.diffserv   : ternary;
			hdr.overlay_tcp_udp.srcPort : ternary;
			hdr.overlay_tcp_udp.dstPort : ternary;
		}
		actions = {
			set_ma_id_and_tunnel_dl_N6_complex_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_DL_N6_COMPLEX_IPV4;
	}

    action set_ma_id_and_tunnel_dl_N9_simple_ipv4(ma_id_t ma_id_v, teid_t teid_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
        meta.teid = (teid_t)teid_v;
		meta.qer_id = qer_id;
	}
	
	table dl_N9_simple_ipv4 {
		key = {
			hdr.gtpu.teid[23:0]  : exact;
			hdr.gtpu_ext_psc.qfi : exact;
		}
		actions = {
			set_ma_id_and_tunnel_dl_N9_simple_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_DL_N9_SIMPLE_IPV4;
	}

    action set_ma_id_ul_N6_simple_ipv4(ma_id_t ma_id_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
		meta.qer_id = qer_id;
	}
	
	table ul_N6_simple_ipv4 {
		key = {
			hdr.gtpu.teid[23:0]  : exact;
			hdr.gtpu_ext_psc.qfi : exact;
		}
		actions = {
			set_ma_id_ul_N6_simple_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_UL_N6_SIMPLE_IPV4;
	}

    action set_ma_id_ul_N6_complex_ipv4(ma_id_t ma_id_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
		meta.qer_id = qer_id;
	}
	
	table ul_N6_complex_ipv4 { // require 5 TCAM join
		key = {
			hdr.gtpu.teid[23:0]         : exact;
			hdr.gtpu_ext_psc.qfi        : ternary;
			hdr.overlay_ipv4.dstAddr    : ternary;
			hdr.overlay_ipv4.srcAddr    : ternary;
			hdr.overlay_ipv4.protocol   : ternary;
			hdr.overlay_ipv4.diffserv   : ternary;
			hdr.overlay_tcp_udp.srcPort : ternary;
			hdr.overlay_tcp_udp.dstPort : ternary;
		}
		actions = {
			set_ma_id_ul_N6_complex_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_UL_N6_COMPLEX_IPV4;
	}

    action set_ma_id_and_tunnel_ul_N9_simple_ipv4(ma_id_t ma_id_v, teid_t teid_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
        meta.teid = (teid_t)teid_v;
		meta.qer_id = qer_id;
	}
	
	table ul_N9_simple_ipv4 {
		key = {
			hdr.gtpu.teid[23:0]  : exact;
			hdr.gtpu_ext_psc.qfi : exact;
		}
		actions = {
			set_ma_id_and_tunnel_ul_N9_simple_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_UL_N9_SIMPLE_IPV4;
	}

    action set_ma_id_and_tunnel_ul_N9_complex_ipv4(ma_id_t ma_id_v, teid_t teid_v, qer_id_t qer_id) {
		meta.ma_id = ma_id_v;
        meta.teid = (teid_t)teid_v;
		meta.qer_id = qer_id;
	}
	
	table ul_N9_complex_ipv4 { // require 5 TCAM join
		key = {
			hdr.gtpu.teid[23:0]         : exact;
			hdr.gtpu_ext_psc.qfi        : ternary;
			hdr.overlay_ipv4.dstAddr    : ternary;
			hdr.overlay_ipv4.srcAddr    : ternary;
			hdr.overlay_ipv4.protocol   : ternary;
			hdr.overlay_ipv4.diffserv   : ternary;
			hdr.overlay_tcp_udp.srcPort : ternary;
			hdr.overlay_tcp_udp.dstPort : ternary;
		}
		actions = {
			set_ma_id_and_tunnel_ul_N9_complex_ipv4;
			@defaultonly NoAction;
		}
		const default_action = NoAction();
		const size = TABLE_SIZE_UL_N9_COMPLEX_IPV4;
	}

    apply {
        if (meta.uplink) {
			// UL
			if (hdr.overlay_ipv4.isValid()) {
				if (ul_N6_complex_ipv4.apply().miss) {
					if (ul_N6_simple_ipv4.apply().miss) {
						if (ul_N9_complex_ipv4.apply().miss) {
							if (ul_N9_simple_ipv4.apply().miss) {
							}
						}
					}
				}
			}
		} else {
			// DL
			if (hdr.underlay_ipv4.isValid()) {
				// from N9
				if (hdr.underlay_ipv4.isValid()) {
					dl_N9_simple_ipv4.apply();
				}
			} else {
				// from N6
				if (hdr.overlay_ipv4.isValid()) {
					if (dl_N6_complex_ipv4.apply().miss) {
						if (dl_N6_simple_ipv4.apply().miss) {
						}
					}
				}
			}
		}
    }
}
