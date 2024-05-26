use etherparse::SlicedPacket;

pub fn parse_gtpu_layer<'a>(udp_payload: &'a [u8]) -> Option<&'a [u8]> {
	if udp_payload.len() < 16 {
		return None;
	}
	if udp_payload[0] & 0b11111100 != 0x34 {
		return None;
	}
	Some(&udp_payload[16..])
}

pub fn parse_gtpu(packet: &[u8]) -> Option<std::net::IpAddr> {
	match SlicedPacket::from_ip(packet) {
		Err(value) => { return None; },
		Ok(value) => {
			if value.ip.is_none() {
				return None;
			}
			if let Some(transport) = value.transport {
				match transport {
					etherparse::TransportSlice::Udp(udp) => {
						if udp.destination_port() == 2152 {
							if let Some(inner_ip_payload) = parse_gtpu_layer(value.payload) {
								match SlicedPacket::from_ip(inner_ip_payload) {
									Err(value) => { return None; },
									Ok(value) => {
										if value.ip.is_none() {
											return None;
										}
										match value.ip.unwrap() {
											etherparse::InternetSlice::Ipv4(hdr, _) => {
												return Some(std::net::IpAddr::V4(hdr.destination_addr()));
											},
											etherparse::InternetSlice::Ipv6(hdr, _) => {
												return Some(std::net::IpAddr::V6(hdr.destination_addr()));
											},
										}
									}
								}
							}
						} else {
							match value.ip.unwrap() {
								etherparse::InternetSlice::Ipv4(hdr, _) => {
									return Some(std::net::IpAddr::V4(hdr.destination_addr()));
								},
								etherparse::InternetSlice::Ipv6(hdr, _) => {
									return Some(std::net::IpAddr::V6(hdr.destination_addr()));
								},
							}
						}
					},
					_ => {
						match value.ip.unwrap() {
							etherparse::InternetSlice::Ipv4(hdr, _) => {
								return Some(std::net::IpAddr::V4(hdr.destination_addr()));
							},
							etherparse::InternetSlice::Ipv6(hdr, _) => {
								return Some(std::net::IpAddr::V6(hdr.destination_addr()));
							},
						}
					}
				}
			}
		}
	};
	None
}
