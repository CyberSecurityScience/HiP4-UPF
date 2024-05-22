


use std::net::{UdpSocket, IpAddr, SocketAddr};

use crate::UE;

pub struct GtpClient {
    pub socket: UdpSocket,
    pub upf_ip: IpAddr
}

impl GtpClient {
    pub fn new(self_ip: IpAddr, upf_ip: IpAddr) -> Self {
        Self {
            socket: UdpSocket::bind(SocketAddr::new(self_ip, 2152)).unwrap(),
            upf_ip
        }
    }
    pub fn craft_and_send(&self, ue: &UE, payload_size: usize) {
        let overlay_udp_payload_size = payload_size - (
            14 + // Ethernet
            20 + // underlay IP
            8 + // underlay UDP
            16 + // GTP
            20 + // overlay IP
            8 // overlay UDP
        );
        let overlay_size = payload_size - (14 + 20 + 8);
        let overlay_ip_size = payload_size - (14 + 20 + 8 + 16);
        let overlay_ip_gtp = payload_size - (14 + 20 + 8 + 8);
        let mut payload = [0u8; 1500];
        payload[..16].clone_from_slice(&[0x34u8, 0xff, 0x00, 0x5c, 0x9a, 0xcb, 0x04, 0x42, 0x00, 0x00, 0x00, 0x85, 0x01, 0x10, 0x01, 0x00]);
        payload[2..2 + 2].clone_from_slice(&(overlay_ip_gtp as u16).to_be_bytes());
        payload[4..8].clone_from_slice(&ue.teid.unwrap().to_be_bytes());
        payload[16..16 + 20].clone_from_slice(&[0x45, 0x00, 0x00, 0x54, 0x01, 0x39, 0x40, 0x00, 0x3f, 0x11, 0x26, 0x2e, 0xc0, 0xa8, 0x46, 0x87, 0x0c, 0x01, 0x01, 0x02]);
        payload[16 + 2..16 + 2 + 2].clone_from_slice(&(overlay_ip_size as u16).to_be_bytes());
        payload[16 + 20 + 4..16 + 20 + 4 + 2].clone_from_slice(&(overlay_udp_payload_size as u16).to_be_bytes());
        self.socket.send_to(&payload[..overlay_size], SocketAddr::new(self.upf_ip, 2152)).unwrap();
    }

}
