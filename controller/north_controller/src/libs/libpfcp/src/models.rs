#![allow(unused_mut, unused_variables, non_snake_case, non_camel_case_types)]

use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    u8,
};

use bitfield::bitfield;

use super::{PFCPError, PFCPModel};

macro_rules! decode_primitive_u8 {
    ($t:ident, $u:expr) => {
        match num::FromPrimitive::from_u8($u) as Option<$t> {
            Some(a) => a,
            None => {
                return Err(PFCPError::new(&format!(
                    "FromPrimitive error {} from {}",
                    stringify!($t),
                    $u
                )));
            }
        }
    };
}

bitfield! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct PFCPHeaderFlags(u8);
    u8;
    pub getVersion, setVersion: 7, 5;
    pub getFO, setFO: 2, 2; // Follow On
    pub getMP, setMP: 1, 1; // Presence of Message Priority
    pub getSEID, setSEID: 0, 0;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PFCPHeader {
    pub flags: PFCPHeaderFlags,
    pub msg_type: u8,
    pub length: u16,
    pub seid: Option<u64>,
    /// Sequence number, big endian, lower 24 bits are used
    pub seq: u32,
    pub priority: Option<u8>,
}

impl PFCPHeader {
    pub fn encode(&self) -> Vec<u8> {
        let mut ret = vec![self.flags.0, self.msg_type];
        ret.append(&mut self.length.to_be_bytes().to_vec());
        if let Some(seid) = self.seid {
            assert!(self.flags.getSEID() != 0);
            ret.append(&mut seid.to_be_bytes().to_vec());
        }
        let seq_priority = (self.seq << 8) | ((self.priority.map_or(0, |f| f << 4)) as u32);
        ret.append(&mut seq_priority.to_be_bytes().to_vec());
        ret
    }
    pub fn decode(stream: &[u8]) -> Result<(Vec<u8>, &[u8], PFCPHeader), PFCPError> {
        let mut stream = stream;
        if stream.len() < 4 {
            return Err(PFCPError::new(&format!(
                "Expect at 4 octets for PFCP header, got {}",
                stream.len()
            )));
        }
        let flags = PFCPHeaderFlags(stream[0]);
        let msg_type = stream[1];
        let length = u16::from_be_bytes(stream[2..4].try_into().unwrap());
        stream = &stream[4..];
        if stream.len() < length as usize {
            return Err(PFCPError::new(&format!(
                "Message is of length {}, but remaining octects is {}",
                length,
                stream.len()
            )));
        }
        let mut body_length = length as usize;
        let seid = if flags.getSEID() != 0 {
            let r = u64::from_be_bytes(stream[0..8].try_into().unwrap());
            stream = &stream[8..];
            body_length -= 8;
            Some(r)
        } else {
            None
        };
        let seq_priority = u32::from_be_bytes(stream[0..4].try_into().unwrap());
        stream = &stream[4..];
        body_length -= 4;
        let priority = if flags.getMP() != 0 {
            // decode priority
            Some(((seq_priority & 0xF0) as u8) >> 4)
        } else {
            None
        };
        Ok((
            stream[..body_length].to_vec(),
            &stream[body_length..],
            PFCPHeader {
                flags: flags,
                msg_type: msg_type,
                length: length,
                seid: seid,
                seq: (seq_priority >> 8),
                priority: priority,
            },
        ))
    }
    /// Is this message a request or a response to a request
    pub fn is_request(&self) -> bool {
        match self.msg_type {
            1 | 3 | 5 | 7 | 9 | 12 | 14 | 50 | 52 | 54 | 56 => true,
            _ => false,
        }
    }
}

#[test]
pub fn test_pfcp_header() {
    let msg = PFCPHeader {
        flags: PFCPHeaderFlags(0b00100001),
        msg_type: 123,
        length: 32 as u16,
        seid: Some(123456),
        seq: 1,
        priority: None,
    };
    let mut encoded = msg.encode();
    encoded.append(&mut vec![123u8; 20]);
    let mut ptr = encoded.as_mut_slice();
    let decoded = PFCPHeader::decode(&mut ptr).unwrap().2;
    assert_eq!(msg, decoded);
}

bitfield! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SourceIPAddressFlags(u8);
    u8;
    pub getMPL, setMPL: 2, 2;
    pub getV4, setV4: 1, 1;
    pub getV6, setV6: 0, 0;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceIPAddress {
    pub flags: SourceIPAddressFlags,
    pub ipv4: Option<std::net::Ipv4Addr>,
    pub ipv6: Option<std::net::Ipv6Addr>,
    pub prefix_length: Option<u8>,
}
impl PFCPModel for SourceIPAddress {
    const ID: u16 = 192;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.ipv4
            .as_ref()
            .map(|o| result.append(&mut o.octets().to_vec()));
        self.ipv6
            .as_ref()
            .map(|o| result.append(&mut o.octets().to_vec()));
        self.prefix_length.as_ref().map(|o| result.push(*o));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = SourceIPAddressFlags(stream[0]);
        stream = &stream[1..];
        let ipv4 = if flags.getV4() != 0 {
            if stream.len() < 4 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 4, got {}",
                    length
                )));
            }
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(std::net::Ipv4Addr::from(tmp))
        } else {
            None
        };
        let ipv6 = if flags.getV6() != 0 {
            if stream.len() < 16 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 16, got {}",
                    length
                )));
            }
            let tmp: [u8; 16] = stream[..16].try_into().unwrap();
            stream = &stream[16..];
            Some(std::net::Ipv6Addr::from(tmp))
        } else {
            None
        };
        let prefix_length = if flags.getMPL() != 0 {
            if stream.len() < 1 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 1, got {}",
                    length
                )));
            }
            let ret = stream[0];
            stream = &stream[1..];
            Some(ret)
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            ipv4: ipv4,
            ipv6: ipv6,
            prefix_length: prefix_length,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct UPFunctionFeatures(u64);
    u8;
    /// Downlink Data Buffering in CP function is supported by the UP function.
    pub getBUCP, setBUCP: 56, 56;
    /// The buffering parameter 'Downlink Data Notification Delay' is supported by the UP function.
    pub getDDND, setDDND: 57, 57;
    /// The buffering parameter 'DL Buffering Duration' is supported by the UP function.
    pub getDLBD, setDLBD: 58, 58;
    /// Traffic Steering is supported by the UP function.
    pub getTRST, setTRST: 59, 59;
    /// F-TEID allocation / release in the UP function is supported by the UP function.
    pub getFTUP, setFTUP: 60, 60;
    /// The PFD Management procedure is supported by the UP function.
    pub getPFDM, setPFDM: 61, 61;
    /// Header Enrichment of Uplink traffic is supported by the UP function.
    pub getHEEU, setHEEU: 62, 62;
    /// Traffic Redirection Enforcement in the UP function is supported by the UP function.
    pub getTREU, setTREU: 63, 63;
    /// Sending of End Marker packets supported by the UP function.
    pub getEMPU, setEMPU: 48, 48;
    /// Support of PDI optimised signalling in UP function (see clause 5.2.1A.2).
    pub getPDIU, setPDIU: 49, 49;
    /// Support of UL/DL Buffering Control
    pub getUDBC, setUDBC: 50, 50;
    /// The UP function supports being provisioned with the Quota Action to apply when reaching quotas.
    pub getQUOAC, setQUOAC: 51, 51;
    /// The UP function supports Trace (see clause 5.15).
    pub getTRACE, setTRACE: 52, 52;
    /// The UP function supports Framed Routing (see IETF RFC 2865 [37] and IETF RFC 3162 [38]).
    pub getFRRT, setFRRT: 53, 53;
    /// The UP function supports a PFD Contents including a property with multiple values.
    pub getPFDE, setPFDE: 54, 54;
    /// The UP function supports the Enhanced PFCP Association Release feature (see clause 5.18).
    pub getEPFAR, setEPFAR: 55, 55;
    /// The UP function supports Deferred PDR Activation or Deactivation.
    pub getDPDRA, setDPDRA: 40, 40;
    /// The UP function supports the Activation and Deactivation of Pre-defined PDRs (see clause 5.19).
    pub getADPDP, setADPDP: 41, 41;
    /// The UP function supports allocating UE IP addresses or prefixes (see clause 5.21).
    pub getUEIP, setUEIP: 42, 42;
    /// UPF support of PFCP sessions successively controlled by different SMFs of a same SMF Set (see clause 5.22).
    pub getSSET, setSSET: 43, 43;
    /// The UP function supports measurement of number of packets which is instructed with the flag 'Measurement of Number of Packets' in a URR. See also clause 5.2.2.2.1.
    pub getMNOP, setMNOP: 44, 44;
    /// UPF supports multiple instances of Traffic Endpoint IDs in a PDI.
    pub getMTE, setMTE: 45, 45;
    /// PFCP messages bundling (see clause 6.5) is supported by the UP function.
    pub getBUNDL, setBUNDL: 46, 46;
    /// UPF support of 5G VN Group Communication. (See clause 5.23)
    pub getGCOM, setGCOM: 47, 47;
    /// UPF support for multiple PFCP associations to the SMFs in an SMF set (see clause 5.22.3).
    pub getMPAS, setMPAS: 32, 32;
    /// UPF supports redundant transmission at transport layer.
    pub getRTTL, setRTTL: 33, 33;
    /// UP function support of quota validity time feature.
    pub getVTIME, setVTIME: 34, 34;
    /// UP function support of Number of Reports as specified in clause 5.2.2.2.
    pub getNORP, setNORP: 35, 35;
    /// UPF support of IPTV service (see clause 5.25) 3GPP Release 16 222 3GPP TS 29.244 V16.6.0 (2020-12)
    pub getIPTV, setIPTV: 36, 36;
    /// UPF supports:
    /// - UE IPv6 address(es) allocation with IPv6 prefix length other than default /64 (including allocating /128 individual IPv6 addresses), as specified in clause 4.6.2.2 of of 3GPP TS 23.316 [57]; and
    /// - multiple UE IPv6 addresses allocation using multiple instances of the UE IP Address IE in a same PDI or Traffic Endpoint, or using multiple PDIs or Traffic Endpoints with a different UE IP Address as specified in clause 5.21.1.
    pub getIP6PL, setIP6PL: 37, 37;
    /// Time Sensitive Communication is supported by the UPF (see clause 5.26).
    pub getTSCU, setTSCU: 38, 38;
    /// UPF support of MPTCP Proxy functionality (see clause 5.20)
    pub getMPTCP, setMPTCP: 39, 39;
    /// UPF support of ATSSS-LLL steering functionality (see clause 5.20)
    pub getATSSSLL, setATSSSLL: 24, 24;
    /// UPF support of per QoS flow per UE QoS monitoring (see clause 5.24.4).
    pub getQFQM, setQFQM: 25, 25;
    /// UPF support of per GTP-U Path QoS monitoring (see clause 5.24.5).
    pub getGPQM, setGPQM: 26, 26;
    /// SGW-U support of reporting the size of DL Data Packets. (see clause 5.2.4.1).
    pub getMTEDT, setMTEDT: 27, 27;
    /// UP function support of CIoT feature, e.g. small data packet rate enforcement. (see 5.4.15)
    pub getCIOT, setCIOT: 28, 28;
    /// UPF support of Ethernet PDU Session Anchor Relocation (see clause 5.13.6).
    pub getETHAR, setETHAR: 29, 29;
    /// UPF support of reporting the first buffered / first discarded downlink data after buffering / directly dropped downlink data for downlink data delivery status notification.
    pub getDDDS, setDDDS: 30, 30;
    /// UP function support of Reliable Data Service (see clause 5.29).
    pub getRDS, setRDS: 31, 31;
    /// UPF support of RTT measurements towards the UE Without PMF.
    pub getRTTWP, setRTTWP: 16, 16;
    /// The UP function supports being provisioned in a URR with an Exempted Application ID for Quota Action or an Exempted SDF Filter for Quota Action which is to be used when the quota is exhausted. See also clauses 5.2.2.2.1 and 5.2.2.3.1.
    pub getQUASF, setQUASF: 17, 17;
    /// UP function supports notifying start of Pause of Charging via user plane.
    pub getNSPOC, setNSPOC: 18, 18;
    /// UP function supports the L2TP feature as described in clause 5.31.
    pub getL2TP, setL2TP: 19, 19;
    /// UP function supports the uplink packets buffering during EAS relocation.
    pub getUPBER, setUPBER: 20, 20;
    /// UP function supports Restoration of PFCP Sessions associated with one or more PGW-C/SMF FQ-CSID(s), Group Id(s) or CP IP address(es) (see clause 5.22.4)
    pub getRESPS, setRESPS: 21, 21;
    /// UP function supports IP Address and Port number replacement (see clause 5.33.3).
    pub getIPREP, setIPREP: 22, 22;
    /// UP function support DNS Traffic Steering based on FQDN in the DNS Query message (see clause 5.33.4)
    pub getDNSTS, setDNSTS: 23, 23;
    /// UP function supports Direct Reporting of QoS monitoring events to Local NEF or AF (see clause 5.33.5).
    pub getDRQOS, setDRQOS: 8, 8;
    /// UPF supports sending MBS multicast session data to associated PDU sessions using 5GC individual delivery.
    pub getMBSN4, setMBSN4: 9, 9;
    /// UP function supports Per Slice UP Resource Management (see clause 5.35).
    pub getPSUPRM, setPSUPRM: 10, 10;
}
impl PFCPModel for UPFunctionFeatures {
    const ID: u16 = 43;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 6u16.to_be_bytes().to_vec());
        result.append(&mut self.0.to_be_bytes()[..6].to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        match length {
            4 => {
                let mut bytes = [0u8; 8];
                let bytes_read: [u8; 4] = stream[..4].try_into().unwrap();
                stream = &stream[4..];
                bytes[..4].copy_from_slice(&bytes_read);
                Ok(Self {
                    0: u64::from_be_bytes(bytes),
                })
            }
            6 => {
                let mut bytes = [0u8; 8];
                let bytes_read: [u8; 6] = stream[..6].try_into().unwrap();
                stream = &stream[6..];
                bytes[..6].copy_from_slice(&bytes_read);
                Ok(Self {
                    0: u64::from_be_bytes(bytes),
                })
            }
            x => {
                return Err(PFCPError::new(&format!("Expect length 4 or 6, got {}", x)));
            }
        }        
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct CPFunctionFeatures(u32);
    u8;
    /// Load Control is supported by the CP function.
    pub getLOAD, setLOAD: 24, 24;
    /// Overload Control is supported by the CP function.
    pub getOVRL, setOVRL: 25, 25;
    /// The CP function supports the Enhanced PFCP Association Release feature (see clause 5.18).
    pub getEPFAR, setEPFAR: 26, 26;
    /// SMF support of PFCP sessions successively controlled by different SMFs of a same SMF Set (see clause 5.22).
    pub getSSET, setSSET: 27, 27;
    /// PFCP messages bundling (see clause 6.5) is supported by the CP function.
    pub getBUNDL, setBUNDL: 28, 28;
    /// SMF support for multiple PFCP associations from an SMF set to a single UPF (see clause 5.22.3).
    pub getMPAS, setMPAS: 29, 29;
    /// CP function supports Additional Usage Reports in the PFCP Session Deletion Response (see clause 5.2.2.3.1).
    pub getARDR, setARDR: 30, 30;
    /// CP function supports the UE IP Address Usage Reporting feature, i.e. receiving and handling of UE IP Address Usage Information IE (see clause 5.21.3.2). 
    pub getUIAUR, setUIAUR: 31, 31;
    /// CP function supports PFCP session establishment or modification with Partial Success, i.e. with UP function reporting rules that cannot be activated. See clause 5.2.9.
    pub getPSUCC, setPSUCC: 16, 16;
}
impl PFCPModel for CPFunctionFeatures {
    const ID: u16 = 89;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        if self.0 & 0x00ffff00 != 0 {
            result.append(&mut 3u16.to_be_bytes().to_vec());
            result.append(&mut self.0.to_be_bytes()[..3].to_vec());
        } else {
            result.append(&mut 1u16.to_be_bytes().to_vec());
            result.append(&mut self.0.to_be_bytes()[..1].to_vec());
        };
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 3 && length != 1 {
            return Err(PFCPError::new(&format!("Expect length 3 or 1, got {}", length)));
        }
        let mut bytes = [0u8; 4];
        let bytes_read: [u8; 3] = if length == 3 {
            stream[..3].try_into().unwrap()
        } else {
            let mut ret = [0u8; 3];
            ret[0] = stream[0];
            ret
        };
        stream = &stream[length..];
        bytes[..3].copy_from_slice(&bytes_read);
        Ok(Self {
            0: u32::from_be_bytes(bytes),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum NodeIdType {
    IPV4 = 0,
    IPV6 = 1,
    FQDN = 2,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NodeID {
    /// Node ID Type
    pub node_id_type: NodeIdType,

    /// Node ID Value
    pub node_id: Vec<u8>,
    // FQDN encoding shall be identical to the encoding of a FQDN within a DNS message of clause 3.1 of
    //   IETF RFC 1035 [27] but excluding the trailing zero byte.
    // NOTE: The FQDN field in the IE is not encoded as a dotted string as commonly used in DNS master zone files.
}
impl NodeID {
    pub fn from_ip(ip: std::net::IpAddr) -> NodeID {
        match ip {
            std::net::IpAddr::V4(v4) => NodeID {
                node_id_type: NodeIdType::IPV4,
                node_id: v4.octets().to_vec(),
            },
            std::net::IpAddr::V6(v6) => NodeID {
                node_id_type: NodeIdType::IPV6,
                node_id: v6.octets().to_vec(),
            },
        }
    }
    pub fn to_ip(&self) -> std::net::IpAddr {
        match self.node_id_type {
            NodeIdType::IPV4 => {
                let tmp: [u8; 4] = self.node_id.as_slice().try_into().unwrap();
                std::net::IpAddr::V4(std::net::Ipv4Addr::from(tmp))
            }
            NodeIdType::IPV6 => {
                let tmp: [u8; 16] = self.node_id.as_slice().try_into().unwrap();
                std::net::IpAddr::V6(std::net::Ipv6Addr::from(tmp))
            }
            NodeIdType::FQDN => {
                unimplemented!()
            }
        }
    }
}
impl PFCPModel for NodeID {
    const ID: u16 = 60;
    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut ((self.node_id.len() + 1) as u16).to_be_bytes().to_vec());
        result.push(self.node_id_type as u8);
        result.append(&mut self.node_id.clone());
        result
    }
    fn decode(stream: &[u8]) -> Result<NodeID, PFCPError> {
        let mut length = stream.len();
        let mut stream = stream;
        let id_type = decode_primitive_u8!(NodeIdType, stream[0]);
        stream = &stream[1..];
        length -= 1;
        match id_type {
            NodeIdType::IPV4 => {
                if length != 4 {
                    return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
                }
            }
            NodeIdType::IPV6 => {
                if length != 16 {
                    return Err(PFCPError::new(&format!("Expect length 16, got {}", length)));
                }
            }
            NodeIdType::FQDN => {}
        };
        let content = stream[..length as usize].to_vec();
        stream = &stream[length as usize..];
        Ok(NodeID {
            node_id_type: id_type,
            node_id: content,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryTimeStamp {
    pub timestamp: u32,
}
impl RecoveryTimeStamp {
    pub fn new(startup_time: chrono::DateTime<chrono::offset::Utc>) -> RecoveryTimeStamp {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (startup_time - start).num_seconds() as u32;
        RecoveryTimeStamp { timestamp: diff }
    }
}
impl PFCPModel for RecoveryTimeStamp {
    const ID: u16 = 96;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<RecoveryTimeStamp, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(RecoveryTimeStamp {
            timestamp: timestamp,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Cause {
    Reserved = 0,
    RequestAccepted = 1,
    RequestRejectedUnspecified = 64,
    SessionContextNotFound = 65,
    MandatoryIEMissing = 66,
    ConditionalIEMissing = 67,
    InvalidLength = 68,
    MandatoryIEIncorrect = 69,
}
impl PFCPModel for Cause {
    const ID: u16 = 19;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(*self as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(decode_primitive_u8!(Self, val))
    }
}

#[derive(Debug, Clone, Hash)]
pub struct F_SEID {
    pub ipv4: Option<std::net::Ipv4Addr>,
    pub ipv6: Option<std::net::Ipv6Addr>,
    pub seid: u64,
}
impl F_SEID {
    pub fn new(ip: std::net::IpAddr, seid: u64) -> F_SEID {
        match ip {
            std::net::IpAddr::V4(v4) => F_SEID {
                ipv4: Some(v4),
                ipv6: None,
                seid: seid,
            },
            std::net::IpAddr::V6(v6) => F_SEID {
                ipv4: None,
                ipv6: Some(v6),
                seid: seid,
            },
        }
    }
    pub fn to_single_ip(&self) -> Option<IpAddr> {
        self.ipv6.map_or_else(
            || self.ipv4.map_or(None, |f| Some(IpAddr::V4(f))),
            |f| Some(IpAddr::V6(f)),
        )
    }
}
impl PFCPModel for F_SEID {
    const ID: u16 = 57;

    fn encode(&self) -> Vec<u8> {
        let mut flag = 0u8;
        let mut length: u16 = 1 + 8;
        if self.ipv4.is_some() {
            flag |= 0b10;
            length += 4;
        };
        if self.ipv6.is_some() {
            flag |= 0b01;
            length += 16;
        };
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut length.to_be_bytes().to_vec());
        result.push(flag);
        result.append(&mut self.seid.to_be_bytes().to_vec());
        self.ipv4.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.ipv6.map(|ip| result.append(&mut ip.octets().to_vec()));
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if stream.len() < 9 {
            return Err(PFCPError::new(&format!("insufficient length")));
        }
        let flag = stream[0];
        stream = &stream[1..];
        let seid = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        let v4 = if flag & 0b10 != 0 {
            if stream.len() < 4 {
                return Err(PFCPError::new(&format!("insufficient length")));
            }
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            let ret = std::net::Ipv4Addr::from(tmp);
            Some(ret)
        } else {
            None
        };
        let v6 = if flag & 0b01 != 0 {
            if stream.len() < 16 {
                return Err(PFCPError::new(&format!("insufficient length")));
            }
            let tmp: [u8; 16] = stream[..16].try_into().unwrap();
            stream = &stream[16..];
            let ret = std::net::Ipv6Addr::from(tmp);
            Some(ret)
        } else {
            None
        };
        Ok(F_SEID {
            ipv4: v4,
            ipv6: v6,
            seid: seid,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct PDR_ID {
    pub rule_id: u16,
}
impl PFCPModel for PDR_ID {
    const ID: u16 = 56;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(PDR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Precedence {
    /// Higher value lower precedence, must be greater than 0 and less than 2^31-1
    pub precedence: i32,
}
impl Precedence {
    pub fn default_precedence() -> Precedence {
        Precedence { precedence: 65534 }
    }
}
impl PFCPModel for Precedence {
    const ID: u16 = 29;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.precedence.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = i32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Precedence { precedence: std::cmp::max(0, val) })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum OuterHeaderRemovalDescription {
    GTP_U_UDP_IPv4 = 0,
    GTP_U_UDP_IPv6 = 1,
    UDP_IPv4 = 2,
    UDP_IPv6 = 3,
    IPv4 = 4,
    IPv6 = 5,
    GTP_U_UDP_IP = 6,
    VLAN_S_TAG = 7,
    S_TAG_AND_C_TAG = 8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OuterHeaderRemoval {
    pub desc: OuterHeaderRemovalDescription,
    pub ext_header_deletion: Option<u8>,
}
impl PFCPModel for OuterHeaderRemoval {
    const ID: u16 = 95;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(
            &mut if self.ext_header_deletion.is_some() {
                2u16
            } else {
                1u16
            }
            .to_be_bytes()
            .to_vec(),
        );
        result.push(self.desc as u8);
        self.ext_header_deletion.map(|e| result.push(e));
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 && length != 2 {
            return Err(PFCPError::new(&format!(
                "Expect length 1 or 2, got {}",
                length
            )));
        }
        let desc = decode_primitive_u8!(OuterHeaderRemovalDescription, stream[0]);
        stream = &stream[1..];
        let ext_rm = if length == 2 {
            let ret = stream[0];
            stream = &stream[1..];
            Some(ret)
        } else {
            None
        };
        Ok(OuterHeaderRemoval {
            desc: desc,
            ext_header_deletion: ext_rm,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
pub struct FAR_ID {
    pub rule_id: u32,
}
impl FAR_ID {
    pub fn is_predefined(&self) -> bool {
        self.rule_id & 0b1000_0000 != 0
    }
}
impl PFCPModel for FAR_ID {
    const ID: u16 = 108;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(FAR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct URR_ID {
    pub rule_id: u32,
}
impl URR_ID {
    pub fn is_predefined(&self) -> bool {
        self.rule_id & 0b1000_0000 != 0
    }
}
impl PFCPModel for URR_ID {
    const ID: u16 = 81;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(URR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkedURR_ID {
    pub rule_id: u32,
}
impl PFCPModel for LinkedURR_ID {
    const ID: u16 = 82;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(LinkedURR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct QER_ID {
    pub rule_id: u32,
}
impl QER_ID {
    pub fn is_predefined(&self) -> bool {
        self.rule_id & 0b1000_0000 != 0
    }
}
impl PFCPModel for QER_ID {
    const ID: u16 = 109;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(QER_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivatePredefinedRules {
    pub name: String,
}
impl PFCPModel for ActivatePredefinedRules {
    const ID: u16 = 106;

    fn encode(&self) -> Vec<u8> {
        let mut name_bytes = self.name.as_bytes().to_vec();
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut (name_bytes.len() as u16).to_be_bytes().to_vec());
        result.append(&mut name_bytes);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let name = match std::str::from_utf8(stream) {
            Ok(a) => a.to_string(),
            Err(_) => return Err(PFCPError::new(&format!("Name should be UTF-8 encoded"))),
        };
        Ok(Self { name: name })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeactivatePredefinedRules {
    pub name: String,
}
impl PFCPModel for DeactivatePredefinedRules {
    const ID: u16 = 107;

    fn encode(&self) -> Vec<u8> {
        let mut name_bytes = self.name.as_bytes().to_vec();
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut (name_bytes.len() as u16).to_be_bytes().to_vec());
        result.append(&mut name_bytes);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let name = match std::str::from_utf8(stream) {
            Ok(a) => a.to_string(),
            Err(_) => return Err(PFCPError::new(&format!("Name should be UTF-8 encoded"))),
        };
        Ok(Self { name: name })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationTime {}
impl PFCPModel for ActivationTime {
    const ID: u16 = 163;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeactivationTime {}
impl PFCPModel for DeactivationTime {
    const ID: u16 = 164;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MAR_ID {
    pub rule_id: u16,
}
impl PFCPModel for MAR_ID {
    const ID: u16 = 170;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(MAR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketReplicationAndDetectionCarryOnInformation {}
impl PFCPModel for PacketReplicationAndDetectionCarryOnInformation {
    const ID: u16 = 179;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UE_IPAddressPoolIdentity {}
impl PFCPModel for UE_IPAddressPoolIdentity {
    const ID: u16 = 177;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MPTCPApplicableIndication {}
impl PFCPModel for MPTCPApplicableIndication {
    const ID: u16 = 265;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq, Hash)]
    pub struct ApplyAction(u16);
    u8;
    pub getDROP, setDROP: 8, 8;
    pub getFORW, setFORW: 9, 9;
    pub getBUFF, setBUFF: 10, 10;
    pub getNOCP, setNOCP: 11, 11;
    pub getDUPL, setDUPL: 12, 12;
    pub getIPMA, setIPMA: 13, 13;
    pub getIPMD, setIPMD: 14, 14;
    pub getDFRT, setDFRT: 15, 15;
    pub getEDRT, setEDRT: 0, 0;
    pub getBDPN, setBDPN: 1, 1;
    pub getDDPN, setDDPN: 2, 2;
}
impl PFCPModel for ApplyAction {
    const ID: u16 = 44;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.0.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(ApplyAction { 0: val })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct BAR_ID {
    pub rule_id: u8,
}
impl BAR_ID {
    pub fn is_predefined(&self) -> bool {
        self.rule_id & 0b1000_0000 != 0
    }
}
impl PFCPModel for BAR_ID {
    const ID: u16 = 88;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.rule_id);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(BAR_ID { rule_id: val })
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum PDNType {
    IPv4 = 1,
    IPv6 = 2,
    IPv4v6 = 3,
    NonIP = 4,
    Ethernet = 5,
}
impl PFCPModel for PDNType {
    const ID: u16 = 113;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(*self as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(decode_primitive_u8!(Self, val))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum SourceInterface {
    AccessSide = 0,
    CoreSide = 1,
    SGi_LAN_N6_LAN = 2,
    CP_Function = 3,
    _5G_VN_internal = 4,
}
impl PFCPModel for SourceInterface {
    const ID: u16 = 20;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(*self as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(decode_primitive_u8!(Self, val))
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct F_TEIDFlags(u8);
    u8;
    pub getV4, setV4: 0, 0;
    pub getV6, setV6: 1, 1;
    pub getCH, setCH: 2, 2;
    pub getCHID, setCHID: 3, 3;
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct F_TEID {
    pub flags: F_TEIDFlags,
    pub teid: Option<u32>,
    pub ipv4: Option<std::net::Ipv4Addr>,
    pub ipv6: Option<std::net::Ipv6Addr>,
    pub choose_id: Option<u8>,
}
impl F_TEID {
    pub fn new_choose(ipv4: bool, ipv6: bool) -> F_TEID {
        let mut result = F_TEID {
            flags: F_TEIDFlags(0),
            teid: None,
            ipv4: None,
            ipv6: None,
            choose_id: None,
        };
        result.flags.setCH(1);
        if ipv4 {
            result.flags.setV4(1);
        }
        if ipv6 {
            result.flags.setV6(1);
        }
        result
    }
    pub fn from_ohc(ohc: &OuterHeaderCreation) -> Option<F_TEID> {
        if ohc.desc.getGTP_U_UDP_IPv4() == 1 {
            Some(Self {
                flags: {
                    let mut flags = F_TEIDFlags(0);
                    flags.setV4(1);
                    flags
                },
                teid: Some(if let Some(a) = ohc.teid {
                    a
                } else {
                    return None;
                }),
                ipv4: Some(if let Some(a) = ohc.ipv4 {
                    a
                } else {
                    return None;
                }),
                ipv6: None,
                choose_id: None,
            })
        } else if ohc.desc.getGTP_U_UDP_IPv6() == 1 {
            Some(Self {
                flags: {
                    let mut flags = F_TEIDFlags(0);
                    flags.setV6(1);
                    flags
                },
                teid: Some(if let Some(a) = ohc.teid {
                    a
                } else {
                    return None;
                }),
                ipv4: None,
                ipv6: Some(if let Some(a) = ohc.ipv6 {
                    a
                } else {
                    return None;
                }),
                choose_id: None,
            })
        } else {
            None
        }
    }
    pub fn from_ip_teid(ip: IpAddr, teid: u32) -> F_TEID {
        match ip {
            IpAddr::V4(ip) => F_TEID {
                flags: {
                    let mut flags = F_TEIDFlags(0);
                    flags.setV4(1);
                    flags
                },
                teid: Some(teid),
                ipv4: Some(ip),
                ipv6: None,
                choose_id: None,
            },
            IpAddr::V6(ip) => F_TEID {
                flags: {
                    let mut flags = F_TEIDFlags(0);
                    flags.setV6(1);
                    flags
                },
                teid: Some(teid),
                ipv4: None,
                ipv6: Some(ip),
                choose_id: None,
            },
        }
    }
}
impl PFCPModel for F_TEID {
    const ID: u16 = 21;

    fn encode(&self) -> Vec<u8> {
        let mut length: u16 = 1;
        self.teid.map(|_| length += 4);
        self.ipv4.map(|_| length += 4);
        self.ipv6.map(|_| length += 16);
        self.choose_id.map(|_| length += 1);
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut length.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.teid
            .map(|id| result.append(&mut id.to_be_bytes().to_vec()));
        self.ipv4.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.ipv6.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.choose_id.map(|id| result.push(id));
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = F_TEIDFlags(stream[0]);
        stream = &stream[1..];
        let teid = if flags.getCH() == 0 {
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(u32::from_be_bytes(tmp))
        } else {
            None
        };
        let ipv4 = if flags.getV4() != 0 && flags.getCH() == 0 {
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(Ipv4Addr::from(tmp))
        } else {
            None
        };
        let ipv6 = if flags.getV6() != 0 && flags.getCH() == 0 {
            let tmp: [u8; 16] = stream[..16].try_into().unwrap();
            stream = &stream[16..];
            Some(Ipv6Addr::from(tmp))
        } else {
            None
        };
        let choose_id = if flags.getCHID() != 0 {
            let tmp = stream[0];
            stream = &stream[1..];
            Some(tmp)
        } else {
            None
        };
        Ok(F_TEID {
            flags: flags,
            teid: teid,
            ipv4: ipv4,
            ipv6: ipv6,
            choose_id: choose_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInstance {}
impl PFCPModel for NetworkInstance {
    const ID: u16 = 22;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct UE_IPAddressFlags(u8);
    u8;
    pub getV6, setV6: 0, 0;
    pub getV4, setV4: 1, 1;
    /// 0 for Source, 1 for Destination
    pub getSD, setSD: 2, 2;
    pub getIPv6D, setIPv6D: 3, 3;
    pub getCHV4, setCHV4: 4, 4;
    pub getCHV6, setCHV6: 5, 5;
    pub getIP6PL, setI6PL: 6, 3;
}
#[derive(Debug, Clone, PartialEq)]
pub struct UE_IPAddress {
    pub flags: UE_IPAddressFlags,
    pub ipv4: Option<std::net::Ipv4Addr>,
    pub ipv6: Option<std::net::Ipv6Addr>,
    pub ipv6_prefix_delegation_bits: Option<u8>,
    pub ipv6_prefix_length: Option<u8>,
}
impl UE_IPAddress {
    pub fn new() -> UE_IPAddress {
        UE_IPAddress {
            flags: UE_IPAddressFlags(0),
            ipv4: None,
            ipv6: None,
            ipv6_prefix_delegation_bits: None,
            ipv6_prefix_length: None,
        }
    }
}
impl PFCPModel for UE_IPAddress {
    const ID: u16 = 93;

    fn encode(&self) -> Vec<u8> {
        let mut length: u16 = 1;
        self.ipv4.map(|_| length += 4);
        self.ipv6.map(|_| length += 16);
        self.ipv6_prefix_delegation_bits.map(|_| length += 1);
        self.ipv6_prefix_length.map(|_| length += 1);
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut length.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.ipv4.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.ipv6.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.ipv6_prefix_delegation_bits.map(|id| result.push(id));
        self.ipv6_prefix_length.map(|id| result.push(id));
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = UE_IPAddressFlags(stream[0]);
        stream = &stream[1..];
        let ipv4 = if flags.getV4() != 0 {
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(Ipv4Addr::from(tmp))
        } else {
            None
        };
        let ipv6 = if flags.getV6() != 0 {
            let tmp: [u8; 16] = stream[..16].try_into().unwrap();
            stream = &stream[16..];
            Some(Ipv6Addr::from(tmp))
        } else {
            None
        };
        let v6d = if flags.getIPv6D() != 0 {
            let tmp = stream[0];
            stream = &stream[1..];
            Some(tmp)
        } else {
            None
        };
        let v6pl = if flags.getIP6PL() != 0 {
            let tmp = stream[0];
            stream = &stream[1..];
            Some(tmp)
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            ipv4: ipv4,
            ipv6: ipv6,
            ipv6_prefix_delegation_bits: v6d,
            ipv6_prefix_length: v6pl,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrafficEndpointID {
    pub id: u8,
}
impl PFCPModel for TrafficEndpointID {
    const ID: u16 = 131;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.id as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(Self { id: val })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct SDFFilterFlags(u8);
    u8;
    pub getFD, setFD: 0, 0;
    pub getTTC, setTTC: 1, 1;
    /// 0 for Source, 1 for Destination
    pub getSPI, setSPI: 2, 2;
    pub getFL, setFL: 3, 3;
    pub getBID, setBID: 4, 4;
}
#[derive(Debug, Clone, PartialEq)]
pub struct SDFFilter {
    pub length_flow_desc: Option<u16>,
    pub flow_desc: Option<String>,
    pub tos_traffic_class: Option<u16>,
    pub security_parameter_index: Option<u32>,
    pub flow_label: Option<[u8; 3]>,
    pub sdf_filter_id: Option<u32>,
}
impl SDFFilter {
    pub fn get_tos(&self) -> Option<(u8, u8)> {
        if self.tos_traffic_class.is_none() {
            return None;
        }
        let tos = self.tos_traffic_class.unwrap();
        let tos_bytes = tos.to_be_bytes();
        Some((tos_bytes[0], tos_bytes[1]))
    }
    pub fn from_FlowDescriptionAVP(flow_desc: &str) -> Self {
        let flow_desc_bytes = flow_desc.as_bytes();
        Self {
            length_flow_desc: Some(flow_desc_bytes.len() as u16),
            flow_desc: Some(flow_desc.to_string()),
            tos_traffic_class: None,
            security_parameter_index: None,
            flow_label: None,
            sdf_filter_id: None
        }
    }
}
impl PFCPModel for SDFFilter {
    const ID: u16 = 23;

    fn encode(&self) -> Vec<u8> {
        let mut flag = SDFFilterFlags(0);
        if self.length_flow_desc.is_some() && self.flow_desc.is_some() {
            flag.setFD(1);
        }
        if self.tos_traffic_class.is_some() {
            flag.setTTC(1);
        }
        if self.security_parameter_index.is_some() {
            flag.setSPI(1);
        }
        if self.flow_label.is_some() {
            flag.setFL(1);
        }
        if self.sdf_filter_id.is_some() {
            flag.setBID(1);
        }
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(flag.0);
        result.push(0);
        self.length_flow_desc
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.flow_desc
            .as_ref()
            .map(|o| result.append(&mut o.as_bytes().to_vec()));
        self.tos_traffic_class
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.security_parameter_index
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.flow_label
            .as_ref()
            .map(|o| result.append(&mut o.to_vec()));
        self.sdf_filter_id
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flag = SDFFilterFlags(stream[0]);
        stream = &stream[1..];
        stream = &stream[1..]; // skip spare
        let (len_fd, fd) = if flag.getFD() != 0 {
            if stream.len() < 2 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 2, got {}",
                    length
                )));
            }
            let tmp: [u8; 2] = stream[..2].try_into().unwrap();
            stream = &stream[2..];
            let len = u16::from_be_bytes(tmp);
            if stream.len() < len as usize {
                return Err(PFCPError::new(&format!(
                    "Expect length at least {}, got {}",
                    len, length
                )));
            }
            let fd = stream[..len as usize].to_vec();
            stream = &stream[len as usize..];
            (Some(len), Some(fd))
        } else {
            (None, None)
        };
        let tos_traffic_class = if flag.getTTC() != 0 {
            if stream.len() < 2 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 2, got {}",
                    length
                )));
            }
            let tmp: [u8; 2] = stream[..2].try_into().unwrap();
            stream = &stream[2..];
            Some(u16::from_be_bytes(tmp))
        } else {
            None
        };
        let security_parameter_index = if flag.getSPI() != 0 {
            if stream.len() < 4 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 4, got {}",
                    length
                )));
            }
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(u32::from_be_bytes(tmp))
        } else {
            None
        };
        let flow_label = if flag.getFL() != 0 {
            if stream.len() < 3 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 3, got {}",
                    length
                )));
            }
            let tmp: [u8; 3] = stream[..3].try_into().unwrap();
            stream = &stream[3..];
            Some(tmp)
        } else {
            None
        };
        let sdf_filter_id = if flag.getBID() != 0 {
            if stream.len() != 4 {
                return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
            }
            let tmp: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(u32::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            length_flow_desc: len_fd,
            flow_desc: fd.map(|fa| std::str::from_utf8(fa.as_slice()).unwrap().to_string()),
            tos_traffic_class: tos_traffic_class,
            security_parameter_index: security_parameter_index,
            flow_label: flow_label,
            sdf_filter_id: sdf_filter_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationID {}
impl PFCPModel for ApplicationID {
    const ID: u16 = 24;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EthernetPDUSessionInformation {}
impl PFCPModel for EthernetPDUSessionInformation {
    const ID: u16 = 142;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FramedRoute {}
impl PFCPModel for FramedRoute {
    const ID: u16 = 153;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FramedRouting {}
impl PFCPModel for FramedRouting {
    const ID: u16 = 154;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FramedIPv6Route {}
impl PFCPModel for FramedIPv6Route {
    const ID: u16 = 155;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum _3GPPInterfaceType {
    S1_U = 0,
    S5_S8_U = 1,
    S4_U = 2,
    S11_U = 3,
    S12_U = 4,
    Gn_Gp_U = 5,
    S2a_U = 6,
    S2b_U = 7,
    eNodeB_GTP_U_interface_for_DL_data_forwarding = 8,
    eNodeB_GTP_U_interface_for_UL_data_forwarding = 9,
    SGW_UPF_GTP_U_interface_for_DL_data_forwarding = 10,
    N3_3GPP_Access = 11,
    N3_Trusted_Non_3GPP_Access = 12,
    N3_Untrusted_Non_3GPP_Access = 13,
    N3_for_data_forwarding = 14,
    N9 = 15,
    SGi = 16,
    N6 = 17,
    N19 = 18,
    S8_U = 19,
    Gp_U = 20,
}
impl PFCPModel for _3GPPInterfaceType {
    const ID: u16 = 160;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(*self as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(decode_primitive_u8!(Self, val))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum DestinationInterface {
    AccessSide = 0,
    CoreSide = 1,
    SGi_LAN_N6_LAN = 2,
    CP_Function = 3,
    LI_Function = 4,
    _5G_VN_internal = 5,
}
impl PFCPModel for DestinationInterface {
    const ID: u16 = 42;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(*self as u8);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(decode_primitive_u8!(Self, val))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedirectInformation {}
impl PFCPModel for RedirectInformation {
    const ID: u16 = 38;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct OuterHeaderCreationDescription(u16);
    u8;
    pub getGTP_U_UDP_IPv4, setGTP_U_UDP_IPv4: 8, 8;
    pub getGTP_U_UDP_IPv6, setGTP_U_UDP_IPv6: 9, 9;
    pub getUDP_IPv4, setUDP_IPv4: 10, 10;
    pub getUDP_IPv6, setUDP_IPv6: 11, 11;
    pub getIPv4, setIPv4: 12, 12;
    pub getIPv6, setIPv6: 13, 13;
    pub getC_TAG, setC_TAG: 14, 14;
    pub getS_TAG, setS_TAG: 15, 15;
    pub getN19Indication, setN19Indication: 0, 0;
    pub getN6Indication, setN6Indication: 1, 1;
}
#[derive(Debug, Clone, PartialEq)]
pub struct OuterHeaderCreation {
    pub desc: OuterHeaderCreationDescription,
    pub teid: Option<u32>,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
    pub port: Option<u16>,
    pub c_tag: Option<[u8; 3]>,
    pub s_tag: Option<[u8; 3]>,
}
impl OuterHeaderCreation {
    pub fn new() -> OuterHeaderCreation {
        OuterHeaderCreation {
            desc: OuterHeaderCreationDescription(0),
            teid: None,
            ipv4: None,
            ipv6: None,
            port: None,
            c_tag: None,
            s_tag: None,
        }
    }
}
impl PFCPModel for OuterHeaderCreation {
    const ID: u16 = 84;

    fn encode(&self) -> Vec<u8> {
        let mut length: u16 = 2;
        self.teid.map(|_| length += 4);
        self.ipv4.map(|_| length += 4);
        self.ipv6.map(|_| length += 16);
        self.port.map(|_| length += 2);
        self.c_tag.map(|_| length += 3);
        self.s_tag.map(|_| length += 3);
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut length.to_be_bytes().to_vec());
        result.append(&mut self.desc.0.to_be_bytes().to_vec());
        self.teid
            .map(|teid| result.append(&mut teid.to_be_bytes().to_vec()));
        self.ipv4.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.ipv6.map(|ip| result.append(&mut ip.octets().to_vec()));
        self.port
            .map(|port| result.append(&mut port.to_be_bytes().to_vec()));
        self.c_tag.map(|tag| result.append(&mut tag.to_vec()));
        self.s_tag.map(|tag| result.append(&mut tag.to_vec()));
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 2 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 2, got {}",
                length
            )));
        }
        let desc = {
            let bytes: [u8; 2] = stream[..2].try_into().unwrap();
            stream = &stream[2..];
            OuterHeaderCreationDescription(u16::from_be_bytes(bytes))
        };
        let teid = if desc.getGTP_U_UDP_IPv4() != 0 || desc.getGTP_U_UDP_IPv6() != 0 {
            let bytes: [u8; 4] = stream[..4].try_into().unwrap();
            stream = &stream[4..];
            Some(u32::from_be_bytes(bytes))
        } else {
            None
        };
        let ipv4 =
            if desc.getGTP_U_UDP_IPv4() != 0 || desc.getUDP_IPv4() != 0 || desc.getIPv4() != 0 {
                let bytes: [u8; 4] = stream[..4].try_into().unwrap();
                stream = &stream[4..];
                Some(Ipv4Addr::from(bytes))
            } else {
                None
            };
        let ipv6 =
            if desc.getGTP_U_UDP_IPv6() != 0 || desc.getUDP_IPv6() != 0 || desc.getIPv6() != 0 {
                let bytes: [u8; 16] = stream[..16].try_into().unwrap();
                stream = &stream[16..];
                Some(Ipv6Addr::from(bytes))
            } else {
                None
            };
        let port = if desc.getUDP_IPv6() != 0 || desc.getUDP_IPv4() != 0 {
            let bytes: [u8; 2] = stream[..2].try_into().unwrap();
            stream = &stream[2..];
            Some(u16::from_be_bytes(bytes))
        } else {
            None
        };
        let c_tag = if desc.getC_TAG() != 0 {
            let bytes: [u8; 3] = stream[..3].try_into().unwrap();
            stream = &stream[3..];
            Some(bytes)
        } else {
            None
        };
        let s_tag = if desc.getS_TAG() != 0 {
            let bytes: [u8; 3] = stream[..3].try_into().unwrap();
            stream = &stream[3..];
            Some(bytes)
        } else {
            None
        };
        Ok(Self {
            desc: desc,
            teid: teid,
            ipv4: ipv4,
            ipv6: ipv6,
            port: port,
            c_tag: c_tag,
            s_tag: s_tag,
        })
    }
}

/// The ToS/Traffic Class shall be encoded on two octets as an OctetString. \
/// The first octet shall contain the DSCP value in the IPv4 Type-of-Service or the IPv6 Traffic-Class field and the second octet shall contain the ToS/Traffic Class mask field, \
/// which shall be set to "0xFC". See clause 5.3.15 of 3GPPTS29.212[8].
#[derive(Debug, Clone, PartialEq)]
pub struct TransportLevelMarking {
    pub tos_val: u16,
}
impl TransportLevelMarking {
    pub fn to_tos(&self) -> u8 {
        (self.tos_val >> 8) as u8
    }
}
impl PFCPModel for TransportLevelMarking {
    const ID: u16 = 30;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.tos_val.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(Self { tos_val: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardingPolicy {}
impl PFCPModel for ForwardingPolicy {
    const ID: u16 = 41;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderEnrichment {}
impl PFCPModel for HeaderEnrichment {
    const ID: u16 = 98;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proxying {}
impl PFCPModel for Proxying {
    const ID: u16 = 137;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataNetworkAccessIdentifier {
    pub id: Vec<u8>,
}
impl PFCPModel for DataNetworkAccessIdentifier {
    const ID: u16 = 232;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut (self.id.len() as u16).to_be_bytes().to_vec());
        result.append(&mut self.id.clone());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        let val = stream[..length as usize].to_vec();
        stream = &stream[length as usize..];
        Ok(Self { id: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgoUpfPerfReport {
    pub stats1: u64,
    pub stats2: u64,
    pub stats3: u64,
    pub stats4: u64,
    pub stats_total: u64
}
impl PFCPModel for AgoUpfPerfReport {
    const ID: u16 = 999;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 40u16.to_be_bytes().to_vec());
        result.append(&mut self.stats1.to_be_bytes().to_vec());
        result.append(&mut self.stats2.to_be_bytes().to_vec());
        result.append(&mut self.stats3.to_be_bytes().to_vec());
        result.append(&mut self.stats4.to_be_bytes().to_vec());
        result.append(&mut self.stats_total.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 40 {
            return Err(PFCPError::new(&format!("Expect length 40, got {}", length)));
        }
        let stats1: u64 = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        let stats2 = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        let stats3 = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        let stats4 = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        let stats_total = u64::from_be_bytes(stream[..8].try_into().unwrap());
        stream = &stream[8..];
        Ok(Self { stats1, stats2, stats3, stats4, stats_total })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OffendingIE {
    pub ie: u16,
}
impl PFCPModel for OffendingIE {
    const ID: u16 = 40;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.ie.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(Self { ie: val })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct PFCPSMReqFlags(u8);
    u8;
    pub getDROBU, setDROBU: 0, 0;
    pub getSNDEM, setSNDEM: 1, 1;
    pub getQAURR, setQAURR: 2, 2;
}
impl PFCPModel for PFCPSMReqFlags {
    const ID: u16 = 49;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = PFCPSMReqFlags(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReportType(u8);
    u8;
    /// Downlink Data Report
    pub getDLDR, setDLDR: 0, 0;
    /// Usage Report
    pub getUSAR, setUSAR: 1, 1;
    /// Error Indication Report
    pub getERIR, setERIR: 2, 2;
    /// User Plane Inactivity Report
    pub getUPIR, setUPIR: 3, 3;
    /// TSCManagement Information Report
    pub getTMIR, setTMIR: 4, 4;
    /// Session Report
    pub getSESR, setSESR: 5, 5;
    /// UP Initiated Session Request
    pub getUISR, setUISR: 6, 6;
}
impl PFCPModel for ReportType {
    const ID: u16 = 39;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = ReportType(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownlinkDataServiceInformation {}
impl PFCPModel for DownlinkDataServiceInformation {
    const ID: u16 = 45;

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        todo!()
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct DataStatus(u8);
    u8;
    pub getDROP, setDROP: 0, 0;
    pub getBUFF, setBUFF: 1, 1;
}
impl PFCPModel for DataStatus {
    const ID: u16 = 260;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = DataStatus(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct UR_SEQN {
    pub sqn: u32,
}
impl PFCPModel for UR_SEQN {
    const ID: u16 = 104;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.sqn.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let val = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self { sqn: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QERCorrelationID {
    pub rule_id: u16,
}
impl PFCPModel for QERCorrelationID {
    const ID: u16 = 28;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(QERCorrelationID { rule_id: val })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct GateStatus(u8);
    u8;
    /// 0 - OPEN, 1 - CLOSED
    pub getDLGate, setDLDate: 1, 0;
    /// 0 - OPEN, 1 - CLOSED
    pub getULGate, setULGate: 3, 2;
}
impl PFCPModel for GateStatus {
    const ID: u16 = 25;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = GateStatus(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MBR {
    /// BR fields shall be encoded as kilobits per second (1 kbps = 1000 bps), 40 bits long
    /// If such conversions result in fractions, then the value of UL/DL MBR fields shall be rounded upwards
    pub ul_mbr: u64,
    /// BR fields shall be encoded as kilobits per second (1 kbps = 1000 bps), 40 bits long
    /// If such conversions result in fractions, then the value of UL/DL MBR fields shall be rounded upwards
    pub dl_mbr: u64,
}
impl PFCPModel for MBR {
    const ID: u16 = 26;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 10u16.to_be_bytes().to_vec());
        result.append(&mut self.ul_mbr.to_be_bytes()[3..].to_vec());
        result.append(&mut self.dl_mbr.to_be_bytes()[3..].to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 10 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 10, got {}",
                length
            )));
        }
        let ul = {
            let mut tmp: [u8; 8] = [0u8; 8];
            tmp[3..].clone_from_slice(&stream[..5]);
            stream = &stream[5..];
            u64::from_be_bytes(tmp)
        };
        let dl = {
            let mut tmp: [u8; 8] = [0u8; 8];
            tmp[3..].clone_from_slice(&stream[..5]);
            stream = &stream[5..];
            u64::from_be_bytes(tmp)
        };
        Ok(Self {
            ul_mbr: ul,
            dl_mbr: dl,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GBR {
    /// BR fields shall be encoded as kilobits per second (1 kbps = 1000 bps), 40 bits long
    /// If such conversions result in fractions, then the value of UL/DL MBR fields shall be rounded upwards
    pub ul_mbr: u64,
    /// BR fields shall be encoded as kilobits per second (1 kbps = 1000 bps), 40 bits long
    /// If such conversions result in fractions, then the value of UL/DL MBR fields shall be rounded upwards
    pub dl_mbr: u64,
}
impl PFCPModel for GBR {
    const ID: u16 = 27;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 10u16.to_be_bytes().to_vec());
        result.append(&mut self.ul_mbr.to_be_bytes()[3..].to_vec());
        result.append(&mut self.dl_mbr.to_be_bytes()[3..].to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 10 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 10, got {}",
                length
            )));
        }
        let ul = {
            let mut tmp: [u8; 8] = [0u8; 8];
            tmp[3..].clone_from_slice(&stream[..5]);
            stream = &stream[5..];
            u64::from_be_bytes(tmp)
        };
        let dl = {
            let mut tmp: [u8; 8] = [0u8; 8];
            tmp[3..].clone_from_slice(&stream[..5]);
            stream = &stream[5..];
            u64::from_be_bytes(tmp)
        };
        Ok(Self {
            ul_mbr: ul,
            dl_mbr: dl,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct QFI(u8);
    u8;
    pub getQFI, setQFI: 5, 0;
}
impl PFCPModel for QFI {
    const ID: u16 = 124;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = QFI(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct RQI(u8);
    u8;
    pub getRQI, setRQI: 0, 0;
}
impl PFCPModel for RQI {
    const ID: u16 = 123;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = RQI(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct PagingPolicyIndicator(u8);
    u8;
    pub getPPI, setPPI: 2, 0;
}
impl PFCPModel for PagingPolicyIndicator {
    const ID: u16 = 158;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = PagingPolicyIndicator(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AveragingWindow {
    pub averaging_window: u32,
}
impl AveragingWindow {}
impl PFCPModel for AveragingWindow {
    const ID: u16 = 157;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.averaging_window.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<AveragingWindow, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let averaging_window = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(AveragingWindow {
            averaging_window: averaging_window,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct QERControlIndications(u8);
    u8;
    pub getRCSR, setRCSR: 0, 0;
}
impl PFCPModel for QERControlIndications {
    const ID: u16 = 251;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = QERControlIndications(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownlinkDataNotificationDelay {
    /// Delay in 50ms unit
    pub delay_multiply_50ms: u8,
}
impl PFCPModel for DownlinkDataNotificationDelay {
    const ID: u16 = 46;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.delay_multiply_50ms);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(Self {
            delay_multiply_50ms: val,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuggestedBufferingPacketsCount {
    pub value: u8,
}
impl PFCPModel for SuggestedBufferingPacketsCount {
    const ID: u16 = 140;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.value);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!("Expect length 1, got {}", length)));
        }
        let val = stream[0];
        stream = &stream[1..];
        Ok(Self { value: val })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct DLBufferingDuration(u8);
    u8;
    pub getTimerValue, setTimerValue: 4, 0;
    /// 0 0 0  value is incremented in multiples of 2 seconds
    /// 0 0 1  value is incremented in multiples of 1 minute
    /// 0 1 0  value is incremented in multiples of 10 minutes
    /// 0 1 1  value is incremented in multiples of 1 hour
    /// 1 0 0  value is incremented in multiples of 10 hours
    /// 1 1 1  value indicates that the timer is infinite
    pub getTimerUnit, setTimerUnit: 7, 5;
}
impl PFCPModel for DLBufferingDuration {
    const ID: u16 = 47;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = DLBufferingDuration(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct MeasurementMethod(u8);
    u8;
    /// DURAT (Duration): when set to "1", this indicates a request for measuring the duration of the traffic.
    pub getDURAT, setDURAT: 0, 0;
    /// VOLUM (Volume): when set to "1", this indicates a request for measuring the volume of the traffic.
    pub getVOLUM, setVOLUM: 1, 1;
    /// EVENT (Event): when set to "1", this indicates a request for measuring the events.
    pub getEVENT, setEVENT: 2, 2;
}
impl PFCPModel for MeasurementMethod {
    const ID: u16 = 62;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = MeasurementMethod(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReportingTriggers(u32);
    u8;
    /// (TODO) PERI(Periodic Reporting): when set to "1", this indicates a request for periodic reporting.
    pub getPERIO, setPERIO: 24, 24;
    /// (TODO) VOLTH (Volume Threshold): when set to "1", this indicates a request for reporting when the data volume usage reaches a volume threshold
    pub getVOLTH, setVOLTH: 25, 25;
    /// (TODO) TIMTH (Time Threshold): when set to "1", this indicates a request for reporting when the time usage reaches a time threshold.
    pub getTIMTH, setTIMTH: 26, 26;
    /// (TODO) QUHTI (Quota Holding Time): when set to "1", this indicates a request for reporting when no packets have been received for a period exceeding the Quota Holding Time.
    pub getQUHTI, setQUHTI: 27, 27;
    /// (TODO) START (Start of Traffic): when set to "1", this indicates a request for reporting when detecting the start of an SDF or Application traffic.
    pub getSTART, setSTART: 28, 28;
    /// (TODO) STOPT (Stop of Traffic): when set to "1", this indicates a request for reporting when detectingthe stop of an SDF or Application Traffic.
    pub getSTOPT, setSTOPT: 29, 29;
    /// (TODO) DROTH (Dropped DL Traffic Threshold): when set to "1", this indicates a request for reporting when the DL traffic being dropped reaches a threshold.
    pub getDROTH, setDROTH: 30, 30;
    /// LIUSA (Linked Usage Reporting): when set to "1", this indicates a request for linked usage reporting, i.e. a request for reporting a usage report for a URR when a usage report is reported for a linked URR (see clause 5.2.2.4).
    pub getLIUSA, setLIUSA: 31, 31;
    /// (TODO) VOLQU (Volume Quota): when set to "1", this indicates a request for reporting when a Volume Quota is exhausted.
    pub getVOLQU, setVOLQU: 16, 16;
    /// (TODO) TIMQU (Time Quota): when set to "1", this indicates a request for reporting when a Time Quota is exhausted.
    pub getTIMQU, setTIMQU: 17, 17;
    /// ENVCL (Envelope Closure): when set to "1", this indicates a request for reporting when conditions for closure of envelope is met (see clause 5.2.2.3).
    pub getENVCL, setENVCL: 18, 18;
    /// MACAR (MAC Addresses Reporting): when set to "1", this indicates a request for reporting the MAC (Ethernet) addresses used as source address of frames sent UL by the UE.
    pub getMACAR, setMACAR: 19, 19;
    /// EVETH (Event Threshold): when set to "1", this indicates a request for reporting when an event threshold is reached.
    pub getEVETH, setEVETH: 20, 20;
    /// EVEQU (Event Quota): when set to "1", this indicates a request for reporting when an Event Quota is reached.
    pub getEVEQU, setEVEQU: 21, 21;
    /// IPMJL (IP Multicast Join/Leave): when set to "1", this indicates a request for reporting whenthe UPF adds or removes the PDU session to/from the DL replication tree associated with an IP multicast flow.
    pub getIPMJL, setIPMJL: 22, 22;
    /// IQUVTI(QuotaValidity Time): when set to "1", this indicates a usage report being reported for a URR due to the quota validity timer expiry.
    pub getQUVTI, setQUVTI: 23, 23;
    /// REEMR (REport the End Marker Reception): when set to "1", the SMF instructs the UPF to report the reception of the End Marker packet. See clause 5.2.2.2.1and also clauses 4.2.3.2 and 4.23.4.3 in 3GPP TS23.502[29].
    pub getREEMR, setREEMR: 8, 8;
    /// REDWD (REport Domain WatchDog): when set to "1", the SMF instructs the UPF to report the detection malware traffic.
    pub getREDWD, setREDWD: 9, 9;
}

impl PFCPModel for ReportingTriggers {
    const ID: u16 = 37;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 3u16.to_be_bytes().to_vec());
        result.append(&mut self.0.to_be_bytes()[..3].to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let mut bytes = stream[..3].to_vec();
        stream = &stream[3..];
        bytes.push(0);
        let bytes_4bytes: [u8; 4] = bytes.as_slice().try_into().unwrap();
        let flags = ReportingTriggers(u32::from_be_bytes(bytes_4bytes));
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct VolumeThresholdFlags(u8);
    u8;
    /// TOVOL: If this bit is set to "1", then the Total Volume field shall be present, otherwise the Total Volume field shall not be present.
    pub getTOVOL, setTOVOL: 0, 0;
    /// ULVOL: If this bit is set to "1", then the Uplink Volume field shall be present, otherwise the Uplink Volume field shall not be present.
    pub getULVOL, setULVOL: 1, 1;
    /// DLVOL: If this bit is set to "1", then the Downlink Volume field shall be present, otherwise the Downlink Volume field shall not be present.
    pub getDLVOL, setDLVOL: 2, 2;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeThreshold {
    pub flags: VolumeThresholdFlags,
    /// Total Volume in octets
    pub total_volume: Option<u64>,
    /// Uplink Volume in octets
    pub uplink_volume: Option<u64>,
    /// Downlink Volume in octets
    pub downlink_volume: Option<u64>,
}

impl PFCPModel for VolumeThreshold {
    const ID: u16 = 31;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        let mut flags = self.flags.clone();
        flags.setTOVOL(self.total_volume.is_some() as u8);
        flags.setULVOL(self.uplink_volume.is_some() as u8);
        flags.setDLVOL(self.downlink_volume.is_some() as u8);
        result.push(flags.0);
        self.total_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.uplink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.downlink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = VolumeThresholdFlags(stream[0]);
        stream = &stream[1..];
        let total = if flags.getTOVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let uplink = if flags.getULVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let downlink = if flags.getDLVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            total_volume: total,
            uplink_volume: uplink,
            downlink_volume: downlink,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct VolumeQuotaFlags(u8);
    u8;
    /// TOVOL: If this bit is set to "1", then the Total Volume field shall be present, otherwise the Total Volume field shall not be present.
    pub getTOVOL, setTOVOL: 0, 0;
    /// ULVOL: If this bit is set to "1", then the Uplink Volume field shall be present, otherwise the Uplink Volume field shall not be present.
    pub getULVOL, setULVOL: 1, 1;
    /// DLVOL: If this bit is set to "1", then the Downlink Volume field shall be present, otherwise the Downlink Volume field shall not be present.
    pub getDLVOL, setDLVOL: 2, 2;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeQuota {
    pub flags: VolumeQuotaFlags,
    /// Total Volume in octets
    pub total_volume: Option<u64>,
    /// Uplink Volume in octets
    pub uplink_volume: Option<u64>,
    /// Downlink Volume in octets
    pub downlink_volume: Option<u64>,
}

impl PFCPModel for VolumeQuota {
    const ID: u16 = 73;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.total_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.uplink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.downlink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = VolumeQuotaFlags(stream[0]);
        stream = &stream[1..];
        let total = if flags.getTOVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let uplink = if flags.getULVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let downlink = if flags.getDLVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            total_volume: total,
            uplink_volume: uplink,
            downlink_volume: downlink,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventThreshold {
    pub value: u32,
}
impl PFCPModel for EventThreshold {
    const ID: u16 = 149;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventQuota {
    pub value: u32,
}
impl PFCPModel for EventQuota {
    const ID: u16 = 148;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeThreshold {
    pub value: u32,
}
impl PFCPModel for TimeThreshold {
    const ID: u16 = 32;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeQuota {
    pub value: u32,
}
impl PFCPModel for TimeQuota {
    const ID: u16 = 74;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotaHoldingTime {
    pub value: u32,
}
impl PFCPModel for QuotaHoldingTime {
    const ID: u16 = 71;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct DroppedDLTrafficThresholdFlags(u8);
    u8;
    /// DLPA: If this bit is set to "1", then the Downlink Packets field shall be present, otherwise the Downlink Packets field shall not be present.
    pub getDLPA, setDLPA: 0, 0;
    /// DLBY: If this bitis set to "1", then the Number of Bytes of Downlink Datafield shall be present, otherwise the Number of Bytes of Downlink Datafield shall not be present.
    pub getDLBY, setDLBY: 1, 1;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DroppedDLTrafficThreshold {
    pub flags: DroppedDLTrafficThresholdFlags,
    pub downlink_packets: Option<u64>,
    pub downlink_bytes: Option<u64>,
}

impl PFCPModel for DroppedDLTrafficThreshold {
    const ID: u16 = 72;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.downlink_packets
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.downlink_bytes
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = DroppedDLTrafficThresholdFlags(stream[0]);
        stream = &stream[1..];
        let packets = if flags.getDLPA() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let bytes = if flags.getDLBY() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            downlink_packets: packets,
            downlink_bytes: bytes,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotaValidityTime {
    pub value: u32,
}
impl PFCPModel for QuotaValidityTime {
    const ID: u16 = 181;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct MonitoringTime {
    pub timestamp: u32,
}
impl MonitoringTime {
    pub fn new(t: chrono::DateTime<chrono::offset::Utc>) -> Self {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (t - start).num_seconds() as u32;
        Self { timestamp: diff }
    }
}
impl PFCPModel for MonitoringTime {
    const ID: u16 = 33;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            timestamp: timestamp,
        })
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentVolumeThreshold {
    pub flags: VolumeThresholdFlags,
    /// Total Volume in octets
    pub total_volume: Option<u64>,
    /// Uplink Volume in octets
    pub uplink_volume: Option<u64>,
    /// Downlink Volume in octets
    pub downlink_volume: Option<u64>,
}

impl PFCPModel for SubsequentVolumeThreshold {
    const ID: u16 = 34;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.total_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.uplink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.downlink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = VolumeThresholdFlags(stream[0]);
        stream = &stream[1..];
        let total = if flags.getTOVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let uplink = if flags.getULVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let downlink = if flags.getDLVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            total_volume: total,
            uplink_volume: uplink,
            downlink_volume: downlink,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentTimeThreshold {
    pub value: u32,
}
impl PFCPModel for SubsequentTimeThreshold {
    const ID: u16 = 35;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentEventThreshold {
    pub value: u32,
}
impl PFCPModel for SubsequentEventThreshold {
    const ID: u16 = 151;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentVolumeQuota {
    pub flags: VolumeQuotaFlags,
    /// Total Volume in octets
    pub total_volume: Option<u64>,
    /// Uplink Volume in octets
    pub uplink_volume: Option<u64>,
    /// Downlink Volume in octets
    pub downlink_volume: Option<u64>,
}

impl PFCPModel for SubsequentVolumeQuota {
    const ID: u16 = 121;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(self.flags.0);
        self.total_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.uplink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        self.downlink_volume
            .as_ref()
            .map(|o| result.append(&mut o.to_be_bytes().to_vec()));
        let length: u16 = result.len() as u16 - 4;
        let length_be = length.to_be_bytes();
        result[2] = length_be[0];
        result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = VolumeQuotaFlags(stream[0]);
        stream = &stream[1..];
        let total = if flags.getTOVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let uplink = if flags.getULVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        let downlink = if flags.getDLVOL() != 0 {
            if length < 8 {
                return Err(PFCPError::new(&format!(
                    "Expect length at least 8, got {}",
                    length
                )));
            }
            let tmp: [u8; 8] = stream[..8].try_into().unwrap();
            stream = &stream[8..];
            Some(u64::from_be_bytes(tmp))
        } else {
            None
        };
        Ok(Self {
            flags: flags,
            total_volume: total,
            uplink_volume: uplink,
            downlink_volume: downlink,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentTimeQuota {
    pub value: u32,
}
impl PFCPModel for SubsequentTimeQuota {
    const ID: u16 = 122;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubsequentEventQuota {
    pub value: u32,
}
impl PFCPModel for SubsequentEventQuota {
    const ID: u16 = 150;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InactivityDetectionTime {
    pub value: u32,
}
impl PFCPModel for InactivityDetectionTime {
    const ID: u16 = 36;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct MeasurementInformation(u8);
    u8;
    /// MBQE (Measurement Before QoS Enforcement): when set to "1", this indicates a request to measure the traffic usage before QoS enforcement.
    pub getMBQE, setMBQE: 0, 0;
    /// INAM (Inactive Measurement): when set to "1", this indicates that the measurement shall be paused (inactive).
    pub getINAM, setINAM: 1, 1;
    /// RADI (Reduced Application Detection Information): when set to "1", this indicates that the Application Detection Information reported to the CP function, upon detecting the start or stop of an application, shall only contain the Application ID.
    pub getRADI, setRADI: 2, 2;
    /// ISTM (Immediate Start Time Metering): when set to "1", this indicates that time metering shall start immediately when the flag is received.
    pub getISTM, setISTM: 3, 3;
    /// MNOP (Measurement of Number of Packets): when set to "1", this indicate a request to measure the number of packets transferred in UL/DL/Total in addition to the measurement in octets when Volume based measurement applies.
    pub getMNOP, setMNOP: 4, 4;
    /// SSPOC (Send Start Pause of Charging): when set to "1", this indicate that the UP Function shall send a Start Pause of Charging indication to the upstream GTP-U entity(s) when the Dropped DL Traffic Threshold is reached.
    pub getSSPOC, setSSPOC: 5, 5;
    /// ASPOC (Applicable for Start of Pause of Charging): when set to "1", this indicate that UP Function the the URR is applicable for Start of Pause of Charging, so that it shall stop the usage measurement for the URR when receiving Start Pause of Charging indication from the peer downstream GTP-U entity.
    pub getASPOC, setASPOC: 6, 6;
    /// CIAM (Control of Inactive Measurement): when set to "1", this indicates that the UP function shall read the current value of the "INAM" flag and stop (if "INAM" is set to "1") or resume (if "INAM" is set to "0") the usage measurement accordingly for the given URR with the "APSOC" flag set to "1". If the "CIAM" flag is set to 0 for a URR with the "ASPOC" flag set to "1", the UP function shall ignore the "INAM" flag received in the Measurement Information. The "CIAM" flag shall be ignored for a URR without the ASPOC" flag set to "1".
    pub getCIAM, setCIAM: 7, 7;
}
impl PFCPModel for MeasurementInformation {
    const ID: u16 = 100;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = MeasurementInformation(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPeriod {
    pub value: u32,
}
impl PFCPModel for MeasurementPeriod {
    const ID: u16 = 64;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct NumberOfReports {
    pub rule_id: u16,
}
impl PFCPModel for NumberOfReports {
    const ID: u16 = 182;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 2u16.to_be_bytes().to_vec());
        result.append(&mut self.rule_id.to_be_bytes().to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 2 {
            return Err(PFCPError::new(&format!("Expect length 2, got {}", length)));
        }
        let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
        stream = &stream[2..];
        Ok(Self { rule_id: val })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EthernetInactivityTimer {
    pub value: u32,
}
impl PFCPModel for EthernetInactivityTimer {
    const ID: u16 = 146;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.value.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let value = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            value,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct NodeReportType(u8);
    u8;
    /// UPFR (User Plane Path Failure Report): when set to "1", this indicates a User Plane Path Failure Report.
    pub getUPFR, setUPFR: 0, 0;
    /// UPRR (User Plane Path Recovery Report): when set to "1", this indicates a User Plane Path Recovery Report.
    pub getUPRR, setUPRR: 1, 1;
    /// CKDR (Clock Drift Report): when set to "1", this indicates a Clock Drift Report.
    pub getCKDR, setCKDR: 2, 2;
    /// GPQR (GTP-U Path QoS Report): when set to "1", this indicates a GTP-U Path QoS Report.
    pub getGPQR, setGPQR: 3, 3;
}
impl PFCPModel for NodeReportType {
    const ID: u16 = 101;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!(
                "Expect length at least 1, got {}",
                length
            )));
        }
        let flags = NodeReportType(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct UsageReportTrigger(u32);
    u8;
    /// PERI(Periodic Reporting): when set to "1", this indicates a request for periodic reporting.
    pub getPERIO, setPERIO: 24, 24;
    /// VOLTH (Volume Threshold): when set to "1", this indicates a request for reporting when the data volume usage reaches a volume threshold
    pub getVOLTH, setVOLTH: 25, 25;
    /// TIMTH (Time Threshold): when set to "1", this indicates a request for reporting when the time usage reaches a time threshold.
    pub getTIMTH, setTIMTH: 26, 26;
    /// QUHTI (Quota Holding Time): when set to "1", this indicates a request for reporting when no packets have been received for a period exceeding the Quota Holding Time.
    pub getQUHTI, setQUHTI: 27, 27;
    /// START (Start of Traffic): when set to "1", this indicates a request for reporting when detecting the start of an SDF or Application traffic.
    pub getSTART, setSTART: 28, 28;
    /// STOPT (Stop of Traffic): when set to "1", this indicates a request for reporting when detectingthe stop of an SDF or Application Traffic.
    pub getSTOPT, setSTOPT: 29, 29;
    /// DROTH (Dropped DL Traffic Threshold): when set to "1", this indicates a request for reporting when the DL traffic being dropped reaches a threshold.
    pub getDROTH, setDROTH: 30, 30;
    /// IMMER (Immediate Report): when set to "1", this indicates an immediate report reported on CP function demand.
    pub getIMMER, setIMMER: 31, 31;
    /// VOLQU (Volume Quota): when set to "1", this indicates a request for reporting when a Volume Quota is exhausted.
    pub getVOLQU, setVOLQU: 16, 16;
    /// TIMQU (Time Quota): when set to "1", this indicates a request for reporting when a Time Quota is exhausted.
    pub getTIMQU, setTIMQU: 17, 17;
    /// LIUSA (Linked Usage Reporting): when set to "1", this indicates a linked usage report, i.e. a usage report being reported for a URR due to a usage report being also reported for a linked URR (see clause 5.2.2.4).
    pub getLIUSA, setLIUSA: 18, 18;
    ///  (Termination Report): when set to "1", this indicates a usage report being reported (in a PFCP Session Deletion Response) for a URR due to the termination of the PFCP session, or a usage report being reported (in a PFCP Session Modification Response) for a URR due to the removal of the URR or dissociated from the last PDR.
    pub getTERMR, setTERMR: 19, 19;
    /// MONIT (Monitoring Time): when set to "1", this indicates a usage report being reported for a URR due to the Monitoring Time being reached.
    pub getMONIT, setMONIT: 20, 20;
    /// ENVCL (Envelope Closure): when set to "1", this indicates the usage report is generated for closure of an envelope (see clause 5.2.2.3).
    pub getENVCL, setENVCL: 21, 21;
    /// MACAR (MAC Addresses Reporting): when set to "1", this indicates a usage report to report MAC (Ethernet) addresses used as source address of frames sent UL by the UE.
    pub getMACAR, setMACAR: 22, 22;
    /// EVETH (Event Threshold): when set to "1", this indicates a usage report is generated when an event threshold is reached.
    pub getEVETH, setEVETH: 23, 23;
    /// EVEQU (Event Quota): when set to "1", this indicates that the Event Quota has been exhausted.
    pub getEVEQU, setEVEQU: 8, 8;
    /// TEBUR (Termination By UP function Report): when set to "1", this indicates a usage report being reported for a URR due to the termination of the PFCP session which is initiated by the UP function.
    pub getTEBUR, setTEBUR: 9, 9;
    /// IPMJL (IP Multicast Join/Leave): when set to "1", this indicates a usage report being reported for a URR due to the UPF adding or removing the PDU session to/from the DL replication tree associated with an IP multicast flow.
    pub getIPMJL, setIPMJL: 10, 10;
    /// QUVTI (Quota Validity Time): when set to "1", this indicates a usage report being reported for a URR due to the quota validity timer expiry.
    pub getQUVTI, setQUVTI: 11, 11;
    /// EMRRE (End Marker Reception REport): this indicates that the UP function has received End Marker from the old I-UPF. See clauses 4.2.3.2 and 4.23.4.3 in 3GPP TS 23.502 [29])
    pub getEMRRE, setEMRRE: 12, 12;
}

impl PFCPModel for UsageReportTrigger {
    const ID: u16 = 63;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 3u16.to_be_bytes().to_vec());
        result.append(&mut self.0.to_be_bytes()[..3].to_vec());
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 3 {
            return Err(PFCPError::new(&format!(
                "Expect length 3, got {}",
                length
            )));
        }
        let mut bytes = stream[..3].to_vec();
        stream = &stream[3..];
        bytes.push(0);
        let bytes_4bytes: [u8; 4] = bytes.as_slice().try_into().unwrap();
        let flags = Self(u32::from_be_bytes(bytes_4bytes));
        Ok(flags)
    }
}

#[derive(Debug, Clone)]
pub struct StartTime {
    pub timestamp: u32,
}
impl StartTime {
    pub fn new(t: chrono::DateTime<chrono::offset::Utc>) -> Self {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (t - start).num_seconds() as u32;
        Self { timestamp: diff }
    }
}
impl PFCPModel for StartTime {
    const ID: u16 = 75;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            timestamp: timestamp,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EndTime {
    pub timestamp: u32,
}
impl EndTime {
    pub fn new(t: chrono::DateTime<chrono::offset::Utc>) -> Self {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (t - start).num_seconds() as u32;
        Self { timestamp: diff }
    }
}
impl PFCPModel for EndTime {
    const ID: u16 = 76;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            timestamp: timestamp,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct VolumeMeasurement {
    pub total_volume: Option<u64>,
    pub uplink_volume: Option<u64>,
    pub downlink_volume: Option<u64>,
    pub total_packets: Option<u64>,
    pub uplink_packets: Option<u64>,
    pub downlink_packets: Option<u64>,
}
impl PFCPModel for VolumeMeasurement {
    const ID: u16 = 66;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        let mut flag = 0u8;
        result.append(&mut 0u16.to_be_bytes().to_vec());
        result.push(0);
		self.total_volume.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 0; });
		self.uplink_volume.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 1; });
		self.downlink_volume.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 2; });
		self.total_packets.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 3; });
		self.uplink_packets.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 4; });
		self.downlink_packets.map(|o| { result.extend_from_slice(o.to_be_bytes().as_slice()); flag |= 1u8 << 5; });
        result[4] = flag;
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
        let length = stream.len();
        let mut stream = stream;
        if length < 1 {
            return Err(PFCPError::new(&format!("Expect length at least 1, got {}", length)));
        }
        let flag = stream[0]; stream = &stream[1..];
        let total_volume = if flag & (1u8 << 0) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        let uplink_volume = if flag & (1u8 << 1) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        let downlink_volume = if flag & (1u8 << 2) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        let total_packets = if flag & (1u8 << 3) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        let uplink_packets = if flag & (1u8 << 4) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        let downlink_packets = if flag & (1u8 << 5) != 0 {
            if stream.len() < 8 {
                return Err(PFCPError::new(&format!("Expect length at least 8, got {}", stream.len())));
            }
            let val: [u8; 8] = stream[..8].try_into().unwrap(); stream = &stream[8..];
            Some(u64::from_be_bytes(val))
        } else {
            None
        };
        Ok(Self {
            total_volume,
            uplink_volume,
            downlink_volume,
            total_packets,
            uplink_packets,
            downlink_packets,
        })
    }
}


#[derive(Debug, Clone)]
pub struct DurationMeasurement {
    pub seconds: u32,
}
impl PFCPModel for DurationMeasurement {
    const ID: u16 = 67;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.seconds.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let seconds = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            seconds
        })
    }
}


#[derive(Debug, Clone)]
pub struct TimeOfFirstPacket {
    pub timestamp: u32,
}
impl TimeOfFirstPacket {
    pub fn new(t: chrono::DateTime<chrono::offset::Utc>) -> Self {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (t - start).num_seconds() as u32;
        Self { timestamp: diff }
    }
}
impl PFCPModel for TimeOfFirstPacket {
    const ID: u16 = 69;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            timestamp: timestamp,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TimeOfLastPacket {
    pub timestamp: u32,
}
impl TimeOfLastPacket {
    pub fn new(t: chrono::DateTime<chrono::offset::Utc>) -> Self {
        use chrono::{TimeZone};
        let start = chrono::Utc.ymd(1900, 1, 1).and_hms(0, 0, 0);
        let diff = (t - start).num_seconds() as u32;
        Self { timestamp: diff }
    }
}
impl PFCPModel for TimeOfLastPacket {
    const ID: u16 = 70;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 4u16.to_be_bytes().to_vec());
        result.append(&mut self.timestamp.to_be_bytes().to_vec());
        result
    }
    fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
        let length = stream.len();
        let mut stream = stream;
        if length != 4 {
            return Err(PFCPError::new(&format!("Expect length 4, got {}", length)));
        }
        let timestamp = u32::from_be_bytes(stream[..4].try_into().unwrap());
        stream = &stream[4..];
        Ok(Self {
            timestamp: timestamp,
        })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct UsageInformation(u8);
    u8;
    /// BEF (Before): when set to "1", this indicates usage before a monitoring time.
    pub getBEF, setBEF: 0, 0;
    /// AFT (After): when set to "1", this indicates a usage after a monitoring time.
    pub getAFT, setAFT: 1, 1;
    /// UAE (Usage After Enforcement): when set to "1", this indicates a usage after QoS enforcement.
    pub getUAE, setUAE: 2, 2;
    /// UBE (Usage Before Enforcement): when set to "1", this indicates a usage before QoS enforcement.
    pub getUBE, setUBE: 3, 3;
}
impl PFCPModel for UsageInformation {
    const ID: u16 = 90;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!(
                "Expect length of 1, got {}",
                length
            )));
        }
        let flags = UsageInformation(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PredefinedRulesName {
    pub name: String,
}
impl PFCPModel for PredefinedRulesName {
    const ID: u16 = 299;

    fn encode(&self) -> Vec<u8> {
        let mut name_bytes = self.name.as_bytes().to_vec();
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut (name_bytes.len() as u16).to_be_bytes().to_vec());
        result.append(&mut name_bytes);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let name = match std::str::from_utf8(stream) {
            Ok(a) => a.to_string(),
            Err(_) => return Err(PFCPError::new(&format!("Name should be UTF-8 encoded"))),
        };
        Ok(Self { name: name })
    }
}

bitfield! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct PFCPSRRspFlags(u8);
    u8;
    /// DROBU (Drop Buffered Packets): if this bit is set to "1", it indicates that the UP function shall drop all the packets currently buffered for the PFCP session, if any, prior to further applying the action specified in the Apply Action value of the FARs.
    pub getDROBU, setDROBU: 0, 0;
}
impl PFCPModel for PFCPSRRspFlags {
    const ID: u16 = 50;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        result.append(&mut 1u16.to_be_bytes().to_vec());
        result.push(self.0);
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 {
            return Err(PFCPError::new(&format!(
                "Expect length of 1, got {}",
                length
            )));
        }
        let flags = PFCPSRRspFlags(stream[0]);
        stream = &stream[1..];
        Ok(flags)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct DLBufferingSuggestedPacketCount {
    pub value: u16,
}
impl PFCPModel for DLBufferingSuggestedPacketCount {
    const ID: u16 = 48;

    fn encode(&self) -> Vec<u8> {
        let mut result = Self::ID.to_be_bytes().to_vec();
        if self.value < 256 {
            result.append(&mut 1u16.to_be_bytes().to_vec());
            result.push(self.value as u8)
        } else {
            result.append(&mut 2u16.to_be_bytes().to_vec());
            result.extend_from_slice(self.value.to_be_bytes().as_slice());
        }
        result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError>
    where
        Self: Sized,
    {
        let length = stream.len();
        let mut stream = stream;
        if length != 1 && length != 2 {
            return Err(PFCPError::new(&format!("Expect length 1 or 2, got {}", length)));
        }
        if length == 1 {
            let val = stream[0];
            stream = &stream[1..];
            Ok(Self { value: val as u16 })
        } else {
            let val = u16::from_be_bytes(stream[..2].try_into().unwrap());
            stream = &stream[2..];
            Ok(Self { value: val })
        }
    }
}

