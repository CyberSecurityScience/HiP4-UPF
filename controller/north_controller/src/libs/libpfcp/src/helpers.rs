
use crate::PFCPError;


pub struct DiameterIPFilterRule {
    pub ip_proto: Option<u8>,
    pub src_ip: Option<cidr::IpCidr>,
    pub dst_ip: Option<cidr::IpCidr>,
    pub src_port_range: Vec<(u16, u16)>,
    pub dst_port_range: Vec<(u16, u16)>,
}

// fn ipv4_to_string(ip: std::net::Ipv4Addr, prefixlen: Option<u8>) -> String {
//     let ip_str = ip.to_string();
//     if let Some(prefixlen) = prefixlen {
//         format!("{}/{}", ip_str, prefixlen)
//     } else {
//         ip_str
//     }
// }

// fn ipv6_to_string(ip: std::net::Ipv6Addr, prefixlen: Option<u8>) -> String {
//     let ip_str = ip.to_string();
//     if let Some(prefixlen) = prefixlen {
//         format!("{}/{}", ip_str, prefixlen)
//     } else {
//         ip_str
//     }
// }

impl DiameterIPFilterRule {
    pub fn from_string(rule: &str) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let items = rule.split(' ').collect::<Vec<_>>();
        if items.len() != 9 {
            return Err(PFCPError::new_boxed("Unsupported IPFilterRule"));
        }
        let proto_str = items[2];
        let src_ip_str = items[4];
        let src_port_str = items[5];
        let dst_ip_str = items[7];
        let dst_port_str = items[8];
        let ip_proto = if proto_str == "any" {
            None
        } else {
            Some(proto_str.parse::<u8>()?)
        };
        let src_ip = if src_ip_str == "any" {
            None
        } else {
            Some(src_ip_str.parse::<cidr::IpCidr>()?)
        };
        let dst_ip = if dst_ip_str == "any" {
            None
        } else {
            Some(dst_ip_str.parse::<cidr::IpCidr>()?)
        };
        let src_port_range = if src_port_str == "any" {
            vec![]
        } else {
            let port_items = src_port_str.split(',').collect::<Vec<_>>();
            let mut ret = Vec::new();
            for range_str in port_items {
                let range_items = range_str.split('-').collect::<Vec<_>>();
                if range_items.len() != 2 && range_items.len() != 1 {
                    return Err(PFCPError::new_boxed("Incorrect IPFilterRule (port must be a range in format of <low>-<high> or <port>)"));
                }
                let low = range_items[0].parse::<u16>()?;
                let high = range_items[range_items.len() - 1].parse::<u16>()?;
                ret.push((low, high));
            }
            ret
        };
        let dst_port_range = if dst_port_str == "any" {
            vec![]
        } else {
            let port_items = dst_port_str.split(',').collect::<Vec<_>>();
            let mut ret = Vec::new();
            for range_str in port_items {
                let range_items = range_str.split('-').collect::<Vec<_>>();
                if range_items.len() != 2 && range_items.len() != 1 {
                    return Err(PFCPError::new_boxed("Incorrect IPFilterRule (port must be a range in format of <low>-<high> or <port>)"));
                }
                let low = range_items[0].parse::<u16>()?;
                let high = range_items[range_items.len() - 1].parse::<u16>()?;
                ret.push((low, high));
            }
            ret
        };
        Ok(Self {
            ip_proto,
            src_ip,
            src_port_range,
            dst_ip,
            dst_port_range
        })
    }
    pub fn to_string(&self) -> String {
        let proto = self.ip_proto.map_or("any".to_string(), |f| f.to_string());
        let src_ip = self.src_ip.map_or("any".to_string(), |f| f.to_string());
        let dst_ip = self.dst_ip.map_or("any".to_string(), |f| f.to_string());
        let src_port = if self.src_port_range.len() == 0 {
            "any".to_string()
        } else {
            let strs = self.src_port_range.iter().map(|(low, high)| if low == high { low.to_string() } else { format!("{}-{}", low, high) }).collect::<Vec<_>>();
            strs.join(",")
        };
        let dst_port = if self.dst_port_range.len() == 0 {
            "any".to_string()
        } else {
            let strs = self.dst_port_range.iter().map(|(low, high)| if low == high { low.to_string() } else { format!("{}-{}", low, high) }).collect::<Vec<_>>();
            strs.join(",")
        };
        format!("permit out {} from {} {} to {} {}", proto, src_ip, src_port, dst_ip, dst_port)
    }
}

#[cfg(test)]
mod test {
    use crate::helpers::DiameterIPFilterRule;
    use std::net::Ipv4Addr;

    use cidr::{IpCidr, Ipv4Cidr};
    
    #[test]
    fn test_DiameterIPFilterRule_1() {
        let rule_str = "permit out 17 from 138.132.115.253 30325 to 172.26.46.0/24 11125-11128";
        let ret = DiameterIPFilterRule::from_string(rule_str).unwrap();
        assert_eq!(ret.ip_proto, Some(17));
        assert_eq!(ret.src_port_range, vec![(30325, 30325)]);
        assert_eq!(ret.dst_port_range, vec!((11125, 11128)));
        assert_eq!(ret.src_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(138,132,115,253), 32).unwrap())));
        assert_eq!(ret.dst_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(172,26,46,0), 24).unwrap())));
    }
    #[test]
    fn test_DiameterIPFilterRule_2() {
        let rule_str = "permit out any from 138.132.115.253 30325 to 172.26.46.0/24 11125-11128,11451";
        let ret = DiameterIPFilterRule::from_string(rule_str).unwrap();
        assert_eq!(ret.ip_proto, None);
        assert_eq!(ret.src_port_range, vec![(30325, 30325)]);
        assert_eq!(ret.dst_port_range, vec![(11125, 11128), (11451, 11451)]);
        assert_eq!(ret.src_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(138,132,115,253), 32).unwrap())));
        assert_eq!(ret.dst_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(172,26,46,0), 24).unwrap())));
    }
    #[test]
    fn test_DiameterIPFilterRule_3() {
        let rule_str = "permit out 17 from 138.132.115.253 30325,1-10 to 172.26.46.0/24 11125-11128";
        let ret = DiameterIPFilterRule::from_string(rule_str).unwrap();
        assert_eq!(ret.ip_proto, Some(17));
        assert_eq!(ret.src_port_range, vec![(30325, 30325), (1, 10)]);
        assert_eq!(ret.dst_port_range, vec![(11125, 11128)]);
        assert_eq!(ret.src_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(138,132,115,253), 32).unwrap())));
        assert_eq!(ret.dst_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(172,26,46,0), 24).unwrap())));
        let new_rule_str = ret.to_string();
        assert_eq!(rule_str, new_rule_str);
    }
    #[test]
    fn test_DiameterIPFilterRule_4() {
        let rule_str = "permit out 17 from any any to 172.26.46.0/24 11125-11128";
        let ret = DiameterIPFilterRule::from_string(rule_str).unwrap();
        assert_eq!(ret.ip_proto, Some(17));
        assert_eq!(ret.src_port_range, vec![]);
        assert_eq!(ret.dst_port_range, vec![(11125, 11128)]);
        assert_eq!(ret.src_ip, None);
        assert_eq!(ret.dst_ip, Some(IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(172,26,46,0), 24).unwrap())));
        let new_rule_str = ret.to_string();
        assert_eq!(rule_str, new_rule_str);
    }
}
