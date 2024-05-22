#![allow(unused_variables)]


use std::{convert::TryInto, net::Ipv4Addr};

use super::{PFCPModel, models::*};
use super::PFCPError;

pub struct HeartbeatRequest {
	pub recovery_time_stamp: RecoveryTimeStamp,
	pub source_ip_address: Option<SourceIPAddress>
}
impl PFCPModel for HeartbeatRequest {
	const ID: u16 = 1;

	fn encode(&self) -> Vec<u8> {
		let mut ret = vec![];
		ret.append(&mut self.recovery_time_stamp.encode());
		self.source_ip_address.as_ref().map(|o| ret.append(&mut o.encode()));
		ret
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut recovery_time_stamp = None;
		let mut source_ip_address = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				RecoveryTimeStamp::ID => { recovery_time_stamp = Some(RecoveryTimeStamp::decode(curmsg)?); }

				SourceIPAddress::ID => { source_ip_address = Some(SourceIPAddress::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			recovery_time_stamp: recovery_time_stamp.ok_or(PFCPError::new("Missing mandatory field RecoveryTimeStamp"))?,

			source_ip_address: source_ip_address,
		})
	}
}

pub struct HeartbeatResponse {
	pub recovery_time_stamp: RecoveryTimeStamp,
}
impl PFCPModel for HeartbeatResponse {
	const ID: u16 = 2;

	fn encode(&self) -> Vec<u8> {
		let mut ret = vec![];
		ret.append(&mut self.recovery_time_stamp.encode());
		ret
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut recovery_time_stamp = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				RecoveryTimeStamp::ID => { recovery_time_stamp = Some(RecoveryTimeStamp::decode(curmsg)?); }

				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			recovery_time_stamp: recovery_time_stamp.ok_or(PFCPError::new("Missing mandatory field RecoveryTimeStamp"))?,
		})
	}
}

pub struct AssociationSetupRequest {
	pub node_id: NodeID,
	pub recovery_time_stamp: RecoveryTimeStamp,
	pub up_function_features: Option<UPFunctionFeatures>,
	pub cp_function_features: Option<CPFunctionFeatures>
}

impl PFCPModel for AssociationSetupRequest {
	const ID: u16 = 5;
	fn encode(&self) -> Vec<u8> {
		let mut ret;
		ret = self.node_id.encode();
		ret.append(&mut self.recovery_time_stamp.encode());
		self.up_function_features.as_ref().map(|o| ret.append(&mut o.encode()));
		self.cp_function_features.as_ref().map(|o| ret.append(&mut o.encode()));
		ret
	}
	fn decode(stream: &[u8]) -> Result<Self, PFCPError> {
		let mut stream = stream;
		let mut node_id = None;
		let mut recovery_time_stamp = None;
		let mut up_function_features = None;
		let mut cp_function_features = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				RecoveryTimeStamp::ID => { recovery_time_stamp = Some(RecoveryTimeStamp::decode(curmsg)?); }

				UPFunctionFeatures::ID => { up_function_features = Some(UPFunctionFeatures::decode(curmsg)?); }
				CPFunctionFeatures::ID => { cp_function_features = Some(CPFunctionFeatures::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			recovery_time_stamp: recovery_time_stamp.ok_or(PFCPError::new("Missing mandatory field RecoveryTimeStamp"))?,

			up_function_features: up_function_features,
			cp_function_features: cp_function_features,
		})
	}
}

pub struct AssociationSetupResponse {
	pub node_id: NodeID,
	pub cause: Cause,
	pub recovery_time_stamp: RecoveryTimeStamp,
	pub up_function_features: Option<UPFunctionFeatures>,
	pub cp_function_features: Option<CPFunctionFeatures>
}

impl PFCPModel for AssociationSetupResponse {
	const ID: u16 = 6;

	fn encode(&self) -> Vec<u8> {
		let mut ret;
		ret = self.node_id.encode();
		ret.append(&mut self.cause.encode());
		ret.append(&mut self.recovery_time_stamp.encode());
		self.up_function_features.as_ref().map(|o| ret.append(&mut o.encode()));
		self.cp_function_features.as_ref().map(|o| ret.append(&mut o.encode()));
		ret
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut cause = None;
		let mut recovery_time_stamp = None;
		let mut up_function_features = None;
		let mut cp_function_features = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }
				RecoveryTimeStamp::ID => { recovery_time_stamp = Some(RecoveryTimeStamp::decode(curmsg)?); }

				UPFunctionFeatures::ID => { up_function_features = Some(UPFunctionFeatures::decode(curmsg)?); }
				CPFunctionFeatures::ID => { cp_function_features = Some(CPFunctionFeatures::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			recovery_time_stamp: recovery_time_stamp.ok_or(PFCPError::new("Missing mandatory field RecoveryTimeStamp"))?,

			up_function_features: up_function_features,
			cp_function_features: cp_function_features,
		})
	}
}

pub struct AssociationReleaseRequest {
	pub node_id: NodeID,
}
impl PFCPModel for AssociationReleaseRequest {
    const ID: u16 = 9;

    fn encode(&self) -> Vec<u8> {
		let ret = self.node_id.encode();
		ret
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
        let mut stream = stream;
		let mut node_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
		})
    }
}

pub struct AssociationReleaseResponse {
	pub node_id: NodeID,
	pub cause: Cause,
}

impl PFCPModel for AssociationReleaseResponse {
	const ID: u16 = 10;

	fn encode(&self) -> Vec<u8> {
		let mut ret;
		ret = self.node_id.encode();
		ret.append(&mut self.cause.encode());
		ret
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut cause = None;
		
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct IPMulticastAddressingInfo {
	
}
impl PFCPModel for IPMulticastAddressingInfo {
	const ID: u16 = 188;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct EthernetPacketFilter {

}
impl PFCPModel for EthernetPacketFilter {
	const ID: u16 = 132;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct RedundantTransmissionDetectionParameters {

}
impl PFCPModel for RedundantTransmissionDetectionParameters {
	const ID: u16 = 255;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct PDI {
	pub source_interface: SourceInterface,
	pub local_f_teid: Option<F_TEID>,
	pub network_instnace: Option<NetworkInstance>,
	pub redundant_transmission_detection_parameters: Option<RedundantTransmissionDetectionParameters>,
	pub ue_ip_address: Vec<UE_IPAddress>,
	pub traffic_endpoint_id: Vec<TrafficEndpointID>,
	pub sdf_filter: Option<SDFFilter>,
	pub application_id: Option<ApplicationID>,
	pub ethernet_pdu_session_nformation: Option<EthernetPDUSessionInformation>,
	pub ethernet_pakcet_filter: Option<EthernetPacketFilter>,
	pub qfi: Vec<QFI>,
	pub framed_route: Vec<FramedRoute>,
	pub framed_routing: Option<FramedRouting>,
	pub framed_ipv6_route: Vec<FramedIPv6Route>,
	pub source_interface_type: Option<_3GPPInterfaceType>,
	pub ip_multicast_addressing_info: Vec<IPMulticastAddressingInfo>
}
impl PFCPModel for PDI {
	const ID: u16 = 2;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.source_interface.encode());
		self.local_f_teid.as_ref().map(|o| result.append(&mut o.encode()));
		self.network_instnace.as_ref().map(|o| result.append(&mut o.encode()));
		self.redundant_transmission_detection_parameters.as_ref().map(|o| result.append(&mut o.encode()));
		self.ue_ip_address.iter().for_each(|o| result.append(&mut o.encode()));
		self.traffic_endpoint_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.sdf_filter.as_ref().map(|o| result.append(&mut o.encode()));
		self.application_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.ethernet_pdu_session_nformation.as_ref().map(|o| result.append(&mut o.encode()));
		self.ethernet_pakcet_filter.as_ref().map(|o| result.append(&mut o.encode()));
		self.ip_multicast_addressing_info.iter().for_each(|o| result.append(&mut o.encode()));
		self.qfi.iter().for_each(|o| result.append(&mut o.encode()));
		self.framed_route.iter().for_each(|o| result.append(&mut o.encode()));
		self.framed_routing.as_ref().map(|o| result.append(&mut o.encode()));
		self.framed_ipv6_route.iter().for_each(|o| result.append(&mut o.encode()));
		self.source_interface_type.as_ref().map(|o| result.append(&mut o.encode()));
		self.ip_multicast_addressing_info.iter().for_each(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut source_interface = None;
		let mut local_f_teid = None;
		let mut network_instnace = None;
		let mut redundant_transmission_detection_parameters = None;
		let mut ue_ip_address = vec![];
		let mut traffic_endpoint_id = vec![];
		let mut sdf_filter = None;
		let mut application_id = None;
		let mut ethernet_pdu_session_nformation = None;
		let mut ethernet_pakcet_filter = None;
		let mut qfi = vec![];
		let mut framed_route = vec![];
		let mut framed_routing = None;
		let mut framed_ipv6_route = vec![];
		let mut source_interface_type = None;
		let mut ip_multicast_addressing_info = vec![];
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				SourceInterface::ID => { source_interface = Some(SourceInterface::decode(curmsg)?); }

				F_TEID::ID => { local_f_teid = Some(F_TEID::decode(curmsg)?); }
				NetworkInstance::ID => { network_instnace = Some(NetworkInstance::decode(curmsg)?); }
				RedundantTransmissionDetectionParameters::ID => { redundant_transmission_detection_parameters = Some(RedundantTransmissionDetectionParameters::decode(curmsg)?); }
				UE_IPAddress::ID => { ue_ip_address.push(UE_IPAddress::decode(curmsg)?); }
				TrafficEndpointID::ID => { traffic_endpoint_id.push(TrafficEndpointID::decode(curmsg)?); }
				SDFFilter::ID => { sdf_filter = Some(SDFFilter::decode(curmsg)?); }
				ApplicationID::ID => { application_id = Some(ApplicationID::decode(curmsg)?); }
				EthernetPDUSessionInformation::ID => { ethernet_pdu_session_nformation = Some(EthernetPDUSessionInformation::decode(curmsg)?); }
				EthernetPacketFilter::ID => { ethernet_pakcet_filter = Some(EthernetPacketFilter::decode(curmsg)?); }
				QFI::ID => { qfi.push(QFI::decode(curmsg)?); }
				FramedRoute::ID => { framed_route.push(FramedRoute::decode(curmsg)?); }
				FramedRouting::ID => { framed_routing = Some(FramedRouting::decode(curmsg)?); }
				FramedIPv6Route::ID => { framed_ipv6_route.push(FramedIPv6Route::decode(curmsg)?); }
				_3GPPInterfaceType::ID => { source_interface_type = Some(_3GPPInterfaceType::decode(curmsg)?); }
				IPMulticastAddressingInfo::ID => { ip_multicast_addressing_info.push(IPMulticastAddressingInfo::decode(curmsg)?); }
				
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			source_interface: source_interface.ok_or(PFCPError::new("Missing mandatory field SourceInterface"))?,

			local_f_teid: local_f_teid,
			network_instnace: network_instnace,
			redundant_transmission_detection_parameters: redundant_transmission_detection_parameters,
			ue_ip_address: ue_ip_address,
			traffic_endpoint_id: traffic_endpoint_id,
			sdf_filter: sdf_filter,
			application_id: application_id,
			ethernet_pdu_session_nformation: ethernet_pdu_session_nformation,
			ethernet_pakcet_filter: ethernet_pakcet_filter,
			qfi: qfi,
			framed_route: framed_route,
			framed_routing: framed_routing,
			framed_ipv6_route: framed_ipv6_route,
			source_interface_type: source_interface_type,
			ip_multicast_addressing_info: ip_multicast_addressing_info,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportDelayReporting {

}
impl PFCPModel for TransportDelayReporting {
	const ID: u16 = 271;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePDR {
	pub pdr_id: PDR_ID,
	pub precedence: Precedence,
	pub pdi: PDI,
	pub outer_header_removal: Option<OuterHeaderRemoval>,
	pub far_id: Option<FAR_ID>,
	pub urr_id: Vec<URR_ID>,
	pub qer_id: Option<QER_ID>,
	pub activate_predefined_rules: Vec<ActivatePredefinedRules>,
	// no part of CreatePDR, used only for UpdatePDR
	//pub deactivate_predefined_rules: Vec<DeactivatePredefinedRules>,
	pub activation_time: Option<ActivationTime>,
	pub deactivation_time: Option<DeactivationTime>,
	pub mar_id: Option<MAR_ID>,
	pub packet_replication_and_detection_carry_on_information: Option<PacketReplicationAndDetectionCarryOnInformation>,
	pub ip_multicast_addressing_info: Vec<IPMulticastAddressingInfo>,
	pub ue_ip_address_pool_identity: Option<UE_IPAddressPoolIdentity>,
	pub mptcp_applicable_indication: Option<MPTCPApplicableIndication>,
	pub transport_delay_reporting: Option<TransportDelayReporting>,
}
impl CreatePDR {
	// pub fn apply_update(&mut self) {
	// 	self.deactivate_predefined_rules.clear();
	// }
}
impl PFCPModel for CreatePDR {
	const ID: u16 = 1;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.pdr_id.encode());
		result.append(&mut self.precedence.encode());
		result.append(&mut self.pdi.encode());
		self.outer_header_removal.as_ref().map(|o| result.append(&mut o.encode()));
		self.far_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.urr_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.qer_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.activate_predefined_rules.iter().for_each(|o| result.append(&mut o.encode()));
		self.activation_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.deactivation_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.mar_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.packet_replication_and_detection_carry_on_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.ip_multicast_addressing_info.iter().for_each(|o| result.append(&mut o.encode()));
		self.ue_ip_address_pool_identity.as_ref().map(|o| result.append(&mut o.encode()));
		self.mptcp_applicable_indication.as_ref().map(|o| result.append(&mut o.encode()));
		self.transport_delay_reporting.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut pdr_id = None;
		let mut precedence = None;
		let mut pdi = None;
		let mut outer_header_removal = None;
		let mut far_id = None;
		let mut urr_id = vec![];
		let mut qer_id = None;
		let mut activate_predefined_rules = vec![];
		let mut activation_time = None;
		let mut deactivation_time = None;
		let mut mar_id = None;
		let mut packet_replication_and_detection_carry_on_information = None;
		let mut ip_multicast_addressing_info = vec![];
		let mut ue_ip_address_pool_identity = None;
		let mut mptcp_applicable_indication = None;
		let mut transport_delay_reporting = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				PDR_ID::ID => { pdr_id = Some(PDR_ID::decode(curmsg)?); }
				Precedence::ID => { precedence = Some(Precedence::decode(curmsg)?); }
				PDI::ID => { pdi = Some(PDI::decode(curmsg)?); }

				OuterHeaderRemoval::ID => { outer_header_removal = Some(OuterHeaderRemoval::decode(curmsg)?); }
				FAR_ID::ID => { far_id = Some(FAR_ID::decode(curmsg)?); }
				URR_ID::ID => { urr_id.push(URR_ID::decode(curmsg)?); }
				QER_ID::ID => { qer_id = Some(QER_ID::decode(curmsg)?); }
				ActivatePredefinedRules::ID => { activate_predefined_rules.push(ActivatePredefinedRules::decode(curmsg)?); }
				ActivationTime::ID => { activation_time = Some(ActivationTime::decode(curmsg)?); }
				DeactivationTime::ID => { deactivation_time = Some(DeactivationTime::decode(curmsg)?); }
				MAR_ID::ID => { mar_id = Some(MAR_ID::decode(curmsg)?); }
				PacketReplicationAndDetectionCarryOnInformation::ID => { packet_replication_and_detection_carry_on_information = Some(PacketReplicationAndDetectionCarryOnInformation::decode(curmsg)?); }
				IPMulticastAddressingInfo::ID => { ip_multicast_addressing_info.push(IPMulticastAddressingInfo::decode(curmsg)?); }
				UE_IPAddressPoolIdentity::ID => { ue_ip_address_pool_identity = Some(UE_IPAddressPoolIdentity::decode(curmsg)?); }
				MPTCPApplicableIndication::ID => { mptcp_applicable_indication = Some(MPTCPApplicableIndication::decode(curmsg)?); }
				TransportDelayReporting::ID => { transport_delay_reporting = Some(TransportDelayReporting::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			pdr_id: pdr_id.ok_or(PFCPError::new("Missing mandatory field PDR_ID"))?,
			precedence: precedence.ok_or(PFCPError::new("Missing mandatory field Precedence"))?,
			pdi: pdi.ok_or(PFCPError::new("Missing mandatory field PDI"))?,

			outer_header_removal: outer_header_removal,
			far_id: far_id,
			urr_id: urr_id,
			qer_id: qer_id,
			activate_predefined_rules: activate_predefined_rules,
			//deactivate_predefined_rules: vec![],
			activation_time: activation_time,
			deactivation_time: deactivation_time,
			mar_id: mar_id,
			packet_replication_and_detection_carry_on_information: packet_replication_and_detection_carry_on_information,
			ip_multicast_addressing_info: ip_multicast_addressing_info,
			ue_ip_address_pool_identity: ue_ip_address_pool_identity,
			mptcp_applicable_indication: mptcp_applicable_indication,
			transport_delay_reporting: transport_delay_reporting,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardingParameters {
	pub destination_interface: DestinationInterface,
	pub network_instnace: Option<NetworkInstance>,
	pub redirect_information: Option<RedirectInformation>,
	pub outer_header_creation: Option<OuterHeaderCreation>,
	pub transport_level_marking: Option<TransportLevelMarking>,
	pub forwarding_policy: Option<ForwardingPolicy>,
	pub header_enrichment: Option<HeaderEnrichment>,
	pub linked_traffic_endpoint_id: Option<TrafficEndpointID>,
	// no used in ForwardingParameters
	pub pfcpsm_req_flags: Option<PFCPSMReqFlags>,
	pub proxying: Option<Proxying>,
	pub destination_interface_type: Option<_3GPPInterfaceType>,
	pub data_network_access_identifier: Option<DataNetworkAccessIdentifier>
}
impl PFCPModel for ForwardingParameters {
	const ID: u16 = 4;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.destination_interface.encode());
		self.network_instnace.as_ref().map(|o| result.append(&mut o.encode()));
		self.redirect_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.outer_header_creation.as_ref().map(|o| result.append(&mut o.encode()));
		self.transport_level_marking.as_ref().map(|o| result.append(&mut o.encode()));
		self.forwarding_policy.as_ref().map(|o| result.append(&mut o.encode()));
		self.header_enrichment.as_ref().map(|o| result.append(&mut o.encode()));
		self.linked_traffic_endpoint_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.proxying.as_ref().map(|o| result.append(&mut o.encode()));
		self.destination_interface_type.as_ref().map(|o| result.append(&mut o.encode()));
		self.data_network_access_identifier.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut destination_interface = None;
		let mut network_instnace = None;
		let mut redirect_information = None;
		let mut outer_header_creation = None;
		let mut transport_level_marking = None;
		let mut forwarding_policy = None;
		let mut header_enrichment = None;
		let mut linked_traffic_endpoint_id = None;
		let mut proxying = None;
		let mut destination_interface_type = None;
		let mut data_network_access_identifier = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				DestinationInterface::ID => { destination_interface = Some(DestinationInterface::decode(curmsg)?); }

				NetworkInstance::ID => { network_instnace = Some(NetworkInstance::decode(curmsg)?); }
				RedirectInformation::ID => { redirect_information = Some(RedirectInformation::decode(curmsg)?); }
				OuterHeaderCreation::ID => { outer_header_creation = Some(OuterHeaderCreation::decode(curmsg)?); }
				TransportLevelMarking::ID => { transport_level_marking = Some(TransportLevelMarking::decode(curmsg)?); }
				ForwardingPolicy::ID => { forwarding_policy = Some(ForwardingPolicy::decode(curmsg)?); }
				HeaderEnrichment::ID => { header_enrichment = Some(HeaderEnrichment::decode(curmsg)?); }
				TrafficEndpointID::ID => { linked_traffic_endpoint_id = Some(TrafficEndpointID::decode(curmsg)?); }
				Proxying::ID => { proxying = Some(Proxying::decode(curmsg)?); }
				_3GPPInterfaceType::ID => { destination_interface_type = Some(_3GPPInterfaceType::decode(curmsg)?); }
				DataNetworkAccessIdentifier::ID => { data_network_access_identifier = Some(DataNetworkAccessIdentifier::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			destination_interface: destination_interface.ok_or(PFCPError::new("Missing mandatory field DestinationInterface"))?,
			
			network_instnace: network_instnace,
			redirect_information: redirect_information,
			outer_header_creation: outer_header_creation,
			transport_level_marking: transport_level_marking,
			forwarding_policy: forwarding_policy,
			header_enrichment: header_enrichment,
			linked_traffic_endpoint_id: linked_traffic_endpoint_id,
			pfcpsm_req_flags: None,
			proxying: proxying,
			destination_interface_type: destination_interface_type,
			data_network_access_identifier: data_network_access_identifier,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuplicatingParameters {
	pub destination_interface: DestinationInterface,
	pub outer_header_creation: Option<OuterHeaderCreation>,
	pub transport_level_marking: Option<TransportLevelMarking>,
	pub forwarding_policy: Option<ForwardingPolicy>,
}
impl PFCPModel for DuplicatingParameters {
	const ID: u16 = 5;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.destination_interface.encode());
		self.outer_header_creation.as_ref().map(|o| result.append(&mut o.encode()));
		self.transport_level_marking.as_ref().map(|o| result.append(&mut o.encode()));
		self.forwarding_policy.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut destination_interface = None;
		let mut outer_header_creation = None;
		let mut transport_level_marking = None;
		let mut forwarding_policy = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				DestinationInterface::ID => { destination_interface = Some(DestinationInterface::decode(curmsg)?); }

				OuterHeaderCreation::ID => { outer_header_creation = Some(OuterHeaderCreation::decode(curmsg)?); }
				TransportLevelMarking::ID => { transport_level_marking = Some(TransportLevelMarking::decode(curmsg)?); }
				ForwardingPolicy::ID => { forwarding_policy = Some(ForwardingPolicy::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			destination_interface: destination_interface.ok_or(PFCPError::new("Missing mandatory field DestinationInterface"))?,
			
			outer_header_creation: outer_header_creation,
			transport_level_marking: transport_level_marking,
			forwarding_policy: forwarding_policy,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedundantTransmissionForwardingParameters {
	
}
impl PFCPModel for RedundantTransmissionForwardingParameters {
	const ID: u16 = 270;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateFAR {
	pub far_id: FAR_ID,
	pub apply_action: ApplyAction,
	pub forwarding_parameters: Option<ForwardingParameters>,
	//pub update_forwarding_parameters: Option<UpdateForwardingParameters>,
	//pub duplicating_parameters: Vec<DuplicatingParameters>,
	pub bar_id: Option<BAR_ID>,
	pub redundant_transmission_forwarding_parameters: Option<RedundantTransmissionForwardingParameters>,
}
impl CreateFAR {
	// pub fn apply_update(&mut self) {
	// 	let tmp_forwarding = self.forwarding_parameters.clone();
	// 	self.forwarding_parameters = self.update_forwarding_parameters.as_ref().map(|o| ForwardingParameters {
	// 		destination_interface: if let Some(a) = o.destination_interface { a } else { tmp_forwarding.as_ref().unwrap().destination_interface.clone() },
	// 		network_instnace: o.network_instnace.clone(),
	// 		redirect_information: o.redirect_information.clone(),
	// 		outer_header_creation: o.outer_header_creation.clone(),
	// 		transport_level_marking: o.transport_level_marking.clone(),
	// 		forwarding_policy: o.forwarding_policy.clone(),
	// 		header_enrichment: o.header_enrichment.clone(),
	// 		linked_traffic_endpoint_id: o.linked_traffic_endpoint_id.clone(),
	// 		pfcpsm_req_flags: None,
	// 		proxying: if let Some(a) = tmp_forwarding.as_ref() { a.proxying.clone() } else { None }, // no way to create proxying in an UpdateFAR request
	// 		destination_interface_type: o.destination_interface_type.clone(),
	// 		data_network_access_identifier: o.data_network_access_identifier.clone(),
	// 	});
	// 	self.update_forwarding_parameters = None;
	// 	// self.duplicating_parameters = self.update_duplicating_parameters.iter().map(|o| DuplicatingParameters {
	// 	// 	destination_interface: o.destination_interface.unwrap(),
	// 	// 	outer_header_creation: o.outer_header_creation.clone(),
	// 	// 	transport_level_marking: o.transport_level_marking.clone(),
	// 	// 	forwarding_policy: o.forwarding_policy.clone(),

	// 	// }).collect::<Vec<_>>();
	// 	// self.duplicating_parameters.clear();
	// }
}
impl PFCPModel for CreateFAR {
	const ID: u16 = 3;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.far_id.encode());
		result.append(&mut self.apply_action.encode());
		self.forwarding_parameters.as_ref().map(|o| result.append(&mut o.encode()));
		//self.duplicating_parameters.iter().for_each(|o| result.append(&mut o.encode()));
		self.bar_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.redundant_transmission_forwarding_parameters.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut far_id = None;
		let mut apply_action = None;
		let mut forwarding_parameters = None;
		let mut duplicating_parameters = vec![];
		let mut bar_id = None;
		let mut redundant_transmission_forwarding_parameters = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				FAR_ID::ID => { far_id = Some(FAR_ID::decode(curmsg)?); }
				ApplyAction::ID => { apply_action = Some(ApplyAction::decode(curmsg)?); }

				ForwardingParameters::ID => { forwarding_parameters = Some(ForwardingParameters::decode(curmsg)?); }
				DuplicatingParameters::ID => { duplicating_parameters.push(DuplicatingParameters::decode(curmsg)?); }
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }
				RedundantTransmissionForwardingParameters::ID => { redundant_transmission_forwarding_parameters = Some(RedundantTransmissionForwardingParameters::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			far_id: far_id.ok_or(PFCPError::new("Missing mandatory field FAR_ID"))?,
			apply_action: apply_action.ok_or(PFCPError::new("Missing mandatory field ApplyAction"))?,
			
			forwarding_parameters: forwarding_parameters,
			//duplicating_parameters: duplicating_parameters,
			bar_id: bar_id,
			redundant_transmission_forwarding_parameters: redundant_transmission_forwarding_parameters,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdditionalMonitoringTime {
	pub monitoring_time: MonitoringTime,
	pub subsequent_volume_threshold: Option<SubsequentVolumeThreshold>,
	pub subsequent_time_threshold: Option<SubsequentTimeThreshold>,
	pub subsequent_volume_quota: Option<SubsequentVolumeQuota>,
	pub subsequent_time_quota: Option<SubsequentTimeQuota>,
	pub subsequent_event_threshold: Option<SubsequentEventThreshold>,
	pub subsequent_event_quota: Option<SubsequentEventQuota>,
}

impl PFCPModel for AdditionalMonitoringTime {
    const ID: u16 = 147;

    fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.monitoring_time.encode());
		self.subsequent_volume_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_volume_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_quota.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
    }

    fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
        let mut stream = stream;
		let mut monitoring_time = None;

		let mut subsequent_volume_threshold = None;
		let mut subsequent_time_threshold = None;
		let mut subsequent_volume_quota = None;
		let mut subsequent_time_quota = None;
		let mut subsequent_event_threshold = None;
		let mut subsequent_event_quota = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				MonitoringTime::ID => { monitoring_time = Some(MonitoringTime::decode(curmsg)?); }

				SubsequentVolumeThreshold::ID => { subsequent_volume_threshold = Some(SubsequentVolumeThreshold::decode(curmsg)?); }
				SubsequentTimeThreshold::ID => { subsequent_time_threshold = Some(SubsequentTimeThreshold::decode(curmsg)?); }
				SubsequentVolumeQuota::ID => { subsequent_volume_quota = Some(SubsequentVolumeQuota::decode(curmsg)?); }
				SubsequentTimeQuota::ID => { subsequent_time_quota = Some(SubsequentTimeQuota::decode(curmsg)?); }
				SubsequentEventThreshold::ID => { subsequent_event_threshold = Some(SubsequentEventThreshold::decode(curmsg)?); }
				SubsequentEventQuota::ID => { subsequent_event_quota = Some(SubsequentEventQuota::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			monitoring_time: monitoring_time.ok_or(PFCPError::new("Missing mandatory field MonitoringTime"))?,
			
			subsequent_volume_threshold,
			subsequent_time_threshold,
			subsequent_volume_quota,
			subsequent_time_quota,
			subsequent_event_threshold,
			subsequent_event_quota,
		})
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateURR {
	pub urr_id: URR_ID,
	pub measurement_method: MeasurementMethod,
	pub reporting_triggers: ReportingTriggers,
	pub measurement_period: Option<MeasurementPeriod>,
	pub volume_threshold: Option<VolumeThreshold>,
	pub volume_quota: Option<VolumeQuota>,
	pub event_threshold: Option<EventThreshold>,
	pub event_quota: Option<EventQuota>,
	pub time_threshold: Option<TimeThreshold>,
	pub time_quota: Option<TimeQuota>,
	pub quota_holding_time: Option<QuotaHoldingTime>,
	pub dropped_dl_traffic_threshold: Option<DroppedDLTrafficThreshold>,
	pub quota_validity_time: Option<QuotaValidityTime>,
	pub monitoring_time: Option<MonitoringTime>,
	pub subsequent_volume_threshold: Option<SubsequentVolumeThreshold>,
	pub subsequent_time_threshold: Option<SubsequentTimeThreshold>,
	pub subsequent_volume_quota: Option<SubsequentVolumeQuota>,
	pub subsequent_time_quota: Option<SubsequentTimeQuota>,
	pub subsequent_event_threshold: Option<SubsequentEventThreshold>,
	pub subsequent_event_quota: Option<SubsequentEventQuota>,
	pub inactivity_detection_time: Option<InactivityDetectionTime>,
	pub linked_urr_id: Vec<LinkedURR_ID>,
	pub measurement_information: Option<MeasurementInformation>,
	// (NOT N4): Time Quota Mechanism
	// (NOT N4): Aggregated URRs
	pub far_id_for_quota_action: Option<FAR_ID>,
	pub ethernet_inactivity_timer: Option<EthernetInactivityTimer>,
	pub additional_monitoring_time: Vec<AdditionalMonitoringTime>,
	pub number_of_reports: Option<NumberOfReports>,
	pub exempted_application_id_for_quota_action: Vec<ApplicationID>,
	pub exempted_sdf_filter_for_quota_action: Vec<SDFFilter>
}
impl CreateURR {
	pub fn apply_update(&mut self) {
		
	}
}
impl PFCPModel for CreateURR {
	const ID: u16 = 6;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.urr_id.encode());
		result.append(&mut self.measurement_method.encode());
		result.append(&mut self.reporting_triggers.encode());
		self.measurement_period.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.event_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.event_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.quota_holding_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.dropped_dl_traffic_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.quota_validity_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.monitoring_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_volume_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_volume_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.inactivity_detection_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.linked_urr_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.measurement_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.far_id_for_quota_action.as_ref().map(|o| result.append(&mut o.encode()));
		self.ethernet_inactivity_timer.as_ref().map(|o| result.append(&mut o.encode()));
		self.additional_monitoring_time.iter().for_each(|o| result.append(&mut o.encode()));
		self.number_of_reports.as_ref().map(|o| result.append(&mut o.encode()));
		self.exempted_application_id_for_quota_action.iter().for_each(|o| result.append(&mut o.encode()));
		self.exempted_sdf_filter_for_quota_action.iter().for_each(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut urr_id = None;
		let mut measurement_method = None;
		let mut reporting_triggers = None;

		let mut measurement_period = None;
		let mut volume_threshold = None;
		let mut volume_quota = None;
		let mut event_threshold = None;
		let mut event_quota = None;
		let mut time_threshold = None;
		let mut time_quota = None;
		let mut quota_holding_time = None;
		let mut dropped_dl_traffic_threshold = None;
		let mut quota_validity_time = None;
		let mut monitoring_time = None;
		let mut subsequent_volume_threshold = None;
		let mut subsequent_time_threshold = None;
		let mut subsequent_volume_quota = None;
		let mut subsequent_time_quota = None;
		let mut subsequent_event_threshold = None;
		let mut subsequent_event_quota = None;
		let mut inactivity_detection_time = None;
		let mut linked_urr_id = vec![];
		let mut measurement_information = None;
		let mut far_id_for_quota_action = None;
		let mut ethernet_inactivity_timer = None;
		let mut additional_monitoring_time = vec![];
		let mut number_of_reports = None;
		let mut exempted_application_id_for_quota_action = vec![];
		let mut exempted_sdf_filter_for_quota_action = vec![];
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				URR_ID::ID => { urr_id = Some(URR_ID::decode(curmsg)?); }
				MeasurementMethod::ID => { measurement_method = Some(MeasurementMethod::decode(curmsg)?); }
				ReportingTriggers::ID => { reporting_triggers = Some(ReportingTriggers::decode(curmsg)?); }

				MeasurementPeriod::ID => { measurement_period = Some(MeasurementPeriod::decode(curmsg)?); }
				VolumeThreshold::ID => { volume_threshold = Some(VolumeThreshold::decode(curmsg)?); }
				VolumeQuota::ID => { volume_quota = Some(VolumeQuota::decode(curmsg)?); }
				EventThreshold::ID => { event_threshold = Some(EventThreshold::decode(curmsg)?); }
				EventQuota::ID => { event_quota = Some(EventQuota::decode(curmsg)?); }
				TimeThreshold::ID => { time_threshold = Some(TimeThreshold::decode(curmsg)?); }
				TimeQuota::ID => { time_quota = Some(TimeQuota::decode(curmsg)?); }
				QuotaHoldingTime::ID => { quota_holding_time = Some(QuotaHoldingTime::decode(curmsg)?); }
				DroppedDLTrafficThreshold::ID => { dropped_dl_traffic_threshold = Some(DroppedDLTrafficThreshold::decode(curmsg)?); }
				QuotaValidityTime::ID => { quota_validity_time = Some(QuotaValidityTime::decode(curmsg)?); }
				MonitoringTime::ID => { monitoring_time = Some(MonitoringTime::decode(curmsg)?); }
				SubsequentVolumeThreshold::ID => { subsequent_volume_threshold = Some(SubsequentVolumeThreshold::decode(curmsg)?); }
				SubsequentTimeThreshold::ID => { subsequent_time_threshold = Some(SubsequentTimeThreshold::decode(curmsg)?); }
				SubsequentVolumeQuota::ID => { subsequent_volume_quota = Some(SubsequentVolumeQuota::decode(curmsg)?); }
				SubsequentTimeQuota::ID => { subsequent_time_quota = Some(SubsequentTimeQuota::decode(curmsg)?); }
				SubsequentEventThreshold::ID => { subsequent_event_threshold = Some(SubsequentEventThreshold::decode(curmsg)?); }
				SubsequentEventQuota::ID => { subsequent_event_quota = Some(SubsequentEventQuota::decode(curmsg)?); }
				InactivityDetectionTime::ID => { inactivity_detection_time = Some(InactivityDetectionTime::decode(curmsg)?); }
				LinkedURR_ID::ID => { linked_urr_id.push(LinkedURR_ID::decode(curmsg)?); }
				MeasurementInformation::ID => { measurement_information = Some(MeasurementInformation::decode(curmsg)?); }
				FAR_ID::ID => { far_id_for_quota_action = Some(FAR_ID::decode(curmsg)?); }
				EthernetInactivityTimer::ID => { ethernet_inactivity_timer = Some(EthernetInactivityTimer::decode(curmsg)?); }
				AdditionalMonitoringTime::ID => { additional_monitoring_time.push(AdditionalMonitoringTime::decode(curmsg)?); }
				NumberOfReports::ID => { number_of_reports = Some(NumberOfReports::decode(curmsg)?); }
				ApplicationID::ID => { exempted_application_id_for_quota_action.push(ApplicationID::decode(curmsg)?); }
				SDFFilter::ID => { exempted_sdf_filter_for_quota_action.push(SDFFilter::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			urr_id: urr_id.ok_or(PFCPError::new("Missing mandatory field URR_ID"))?,
			measurement_method: measurement_method.ok_or(PFCPError::new("Missing mandatory field MeasurementMethod"))?,
			reporting_triggers: reporting_triggers.ok_or(PFCPError::new("Missing mandatory field ReportingTriggers"))?,
			
			measurement_period,
			volume_threshold,
			volume_quota,
			event_threshold,
			event_quota,
			time_threshold,
			time_quota,
			quota_holding_time,
			dropped_dl_traffic_threshold,
			quota_validity_time,
			monitoring_time,
			subsequent_volume_threshold,
			subsequent_time_threshold,
			subsequent_volume_quota,
			subsequent_time_quota,
			subsequent_event_threshold,
			subsequent_event_quota,
			inactivity_detection_time,
			linked_urr_id,
			measurement_information,
			far_id_for_quota_action,
			ethernet_inactivity_timer,
			additional_monitoring_time,
			number_of_reports,
			exempted_application_id_for_quota_action,
			exempted_sdf_filter_for_quota_action
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateQER {
	pub qer_id: QER_ID,
	pub qer_corrleation_id: Option<QERCorrelationID>,
	pub gate_status: GateStatus,
	pub maximum_bitrate: Option<MBR>,
	pub guaranteed_bitrate: Option<GBR>,
	// MAYBE TODO: Packet Rate Status
	pub qfi: Option<QFI>,
	pub rqi: Option<RQI>,
	pub paging_policy_indicator: Option<PagingPolicyIndicator>,
	pub averaging_window: Option<AveragingWindow>,
	pub qer_control_indications: Option<QERControlIndications>
}
impl CreateQER {
	pub fn apply_update(&mut self) {
		todo!()
	}
}
impl PFCPModel for CreateQER {
	const ID: u16 = 7;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.qer_id.encode());
		self.qer_corrleation_id.as_ref().map(|o| result.append(&mut o.encode()));
		result.append(&mut self.gate_status.encode());
		self.maximum_bitrate.as_ref().map(|o| result.append(&mut o.encode()));
		self.guaranteed_bitrate.as_ref().map(|o| result.append(&mut o.encode()));
		self.qfi.as_ref().map(|o| result.append(&mut o.encode()));
		self.rqi.as_ref().map(|o| result.append(&mut o.encode()));
		self.paging_policy_indicator.as_ref().map(|o| result.append(&mut o.encode()));
		self.averaging_window.as_ref().map(|o| result.append(&mut o.encode()));
		self.qer_control_indications.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut qer_id = None;
		let mut qer_corrleation_id = None;
		let mut gate_status = None;
		let mut maximum_bitrate = None;
		let mut guaranteed_bitrate = None;
		let mut qfi = None;
		let mut rqi = None;
		let mut paging_policy_indicator = None;
		let mut averaging_window = None;
		let mut qer_control_indications = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				QER_ID::ID => { qer_id = Some(QER_ID::decode(curmsg)?); }
				GateStatus::ID => { gate_status = Some(GateStatus::decode(curmsg)?); }

				QERCorrelationID::ID => { qer_corrleation_id = Some(QERCorrelationID::decode(curmsg)?); }
				MBR::ID => { maximum_bitrate = Some(MBR::decode(curmsg)?); }
				GBR::ID => { guaranteed_bitrate = Some(GBR::decode(curmsg)?); }
				QFI::ID => { qfi = Some(QFI::decode(curmsg)?); }
				RQI::ID => { rqi = Some(RQI::decode(curmsg)?); }
				PagingPolicyIndicator::ID => { paging_policy_indicator = Some(PagingPolicyIndicator::decode(curmsg)?); }
				AveragingWindow::ID => { averaging_window = Some(AveragingWindow::decode(curmsg)?); }
				QERControlIndications::ID => { qer_control_indications = Some(QERControlIndications::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			qer_id: qer_id.ok_or(PFCPError::new("Missing mandatory field QER_ID"))?,
			gate_status: gate_status.ok_or(PFCPError::new("Missing mandatory field GateStatus"))?,
			
			qer_corrleation_id: qer_corrleation_id,
			maximum_bitrate: maximum_bitrate,
			guaranteed_bitrate: guaranteed_bitrate,
			qfi: qfi,
			rqi: rqi,
			paging_policy_indicator: paging_policy_indicator,
			averaging_window: averaging_window,
			qer_control_indications: qer_control_indications,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateBAR {
	pub bar_id: BAR_ID,
	pub downlink_data_notification_delay: Option<DownlinkDataNotificationDelay>,
	pub suggested_buffering_packets_count: Option<SuggestedBufferingPacketsCount>
}
impl CreateBAR {
	pub fn apply_update(&mut self) {
		todo!()
	}
}
impl PFCPModel for CreateBAR {
	const ID: u16 = 85;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.bar_id.encode());
		self.downlink_data_notification_delay.as_ref().map(|o| result.append(&mut o.encode()));
		self.suggested_buffering_packets_count.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut bar_id = None;
		let mut downlink_data_notification_delay = None;
		let mut suggested_buffering_packets_count = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }

				DownlinkDataNotificationDelay::ID => { downlink_data_notification_delay = Some(DownlinkDataNotificationDelay::decode(curmsg)?); }
				SuggestedBufferingPacketsCount::ID => { suggested_buffering_packets_count = Some(SuggestedBufferingPacketsCount::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			bar_id: bar_id.ok_or(PFCPError::new("Missing mandatory field BAR_ID"))?,
			
			downlink_data_notification_delay: downlink_data_notification_delay,
			suggested_buffering_packets_count: suggested_buffering_packets_count
		})
	}
}

#[derive(Debug, Clone)]
pub struct CreateTrafficEndpoint {
}
impl PFCPModel for CreateTrafficEndpoint {
	const ID: u16 = 127;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

#[derive(Clone, Debug)]
pub struct PFCPSessionEstablishmentRequest {
	pub node_id: NodeID,
	pub cp_f_seid: F_SEID,
	pub create_pdr: Vec<CreatePDR>,
	pub create_far: Vec<CreateFAR>,
	pub create_urr: Vec<CreateURR>,
	pub create_qer: Vec<CreateQER>,
	pub create_bar: Option<CreateBAR>,
	pub create_traffic_endpoint: Vec<CreateTrafficEndpoint>,
	pub pdn_type: Option<PDNType>,
	// TODO: User Plane Inactivity Timer
	// TODO: User ID
	// TODO: Trace Information
	// TODO: APN/DNN
	// TODO: Create MAR
	// TODO: PFCPSEReqFlags
	// TODO: Create Bridge Info for TSC
	// TODO: Create SRR
	// TODO: Provide ATSSS Control Information
	// TODO: Recovery Time Stamp
	// TODO: S-NSSAI 
	// TODO: Provide RDS configuration information
}
impl PFCPSessionEstablishmentRequest {
	pub fn new() -> PFCPSessionEstablishmentRequest {
		PFCPSessionEstablishmentRequest {
			node_id: NodeID { node_id_type: NodeIdType::FQDN, node_id: vec![] },
			cp_f_seid: F_SEID::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
			create_pdr: vec![],
			create_far: vec![],
			create_urr: vec![],
			create_qer: vec![],
			create_bar: None,
			create_traffic_endpoint: vec![],
			pdn_type: None,
		}
	}
}
impl PFCPModel for PFCPSessionEstablishmentRequest {
	const ID: u16 = 50;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.node_id.encode());
		result.append(&mut self.cp_f_seid.encode());
		self.create_far.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_urr.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_qer.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_bar.as_ref().map(|o| result.append(&mut o.encode()));
		self.create_pdr.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_traffic_endpoint.iter().for_each(|o| result.append(&mut o.encode()));
		self.pdn_type.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut cp_f_seid = None;
		let mut create_pdr = vec![];
		let mut create_far = vec![];
		let mut create_urr = vec![];
		let mut create_qer = vec![];
		let mut create_bar = None;
		let mut create_traffic_endpoint = vec![];
		let mut pdn_type = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				F_SEID::ID => { cp_f_seid = Some(F_SEID::decode(curmsg)?); }

				CreatePDR::ID => { create_pdr.push(CreatePDR::decode(curmsg)?); }
				CreateFAR::ID => { create_far.push(CreateFAR::decode(curmsg)?); }
				CreateURR::ID => { create_urr.push(CreateURR::decode(curmsg)?); }
				CreateQER::ID => { create_qer.push(CreateQER::decode(curmsg)?); }

				CreateBAR::ID => { create_bar = Some(CreateBAR::decode(curmsg)?); }
				CreateTrafficEndpoint::ID => { create_traffic_endpoint.push(CreateTrafficEndpoint::decode(curmsg)?); }
				PDNType::ID => { pdn_type = Some(PDNType::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			cp_f_seid: cp_f_seid.ok_or(PFCPError::new("Missing mandatory field CP-F-SEID"))?,
			
			create_pdr: create_pdr,
			create_far: create_far,
			create_urr: create_urr,
			create_qer: create_qer,
			create_bar: create_bar,
			create_traffic_endpoint: create_traffic_endpoint,
			pdn_type: pdn_type,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatedPDR {
	pub pdr_id: PDR_ID,
	pub local_f_teid: Option<F_TEID>,
	pub local_f_teid_for_redundant_transmission: Option<F_TEID>,
	pub ue_ip_address: Vec<UE_IPAddress>
}
impl CreatedPDR {
	pub fn from_create_pdr(pdr: &CreatePDR) -> CreatedPDR {
		CreatedPDR {
			pdr_id: pdr.pdr_id.clone(),
			local_f_teid: pdr.pdi.local_f_teid.clone(),
			local_f_teid_for_redundant_transmission: None,
			ue_ip_address: pdr.pdi.ue_ip_address.clone()
		}
	}
}
impl PFCPModel for CreatedPDR {
	const ID: u16 = 8;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.pdr_id.encode());
		self.local_f_teid.as_ref().map(|o| result.append(&mut o.encode()));
		self.local_f_teid_for_redundant_transmission.as_ref().map(|o| result.append(&mut o.encode()));
		self.ue_ip_address.iter().for_each(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut pdr_id = None;
		let mut local_f_teid = None;
		let mut local_f_teid_for_redundant_transmission = None;
		let mut ue_ip_address = vec![];
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				PDR_ID::ID => { pdr_id = Some(PDR_ID::decode(curmsg)?); }

				F_TEID::ID => {
					let s = F_TEID::decode(curmsg)?;
					if local_f_teid.is_none() {
						local_f_teid = Some(s);
					} else {
						local_f_teid_for_redundant_transmission = Some(s);
					}
				}
				UE_IPAddress::ID => { ue_ip_address.push(UE_IPAddress::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			pdr_id: pdr_id.ok_or(PFCPError::new("Missing mandatory field PDR_ID"))?,
			
			local_f_teid: local_f_teid,
			local_f_teid_for_redundant_transmission: local_f_teid_for_redundant_transmission,
			ue_ip_address: ue_ip_address,
		})
	}
}

pub struct PFCPSessionEstablishmentResponse {
	pub node_id: NodeID,
	pub cause: Cause,
	pub offending_ie: Option<OffendingIE>,
	pub up_f_seid: Option<F_SEID>,
	pub created_pdr: Vec<CreatedPDR>,
	// ...

	pub ago_perf: Option<AgoUpfPerfReport>
}
impl PFCPModel for PFCPSessionEstablishmentResponse {
	const ID: u16 = 51;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.node_id.encode());
		result.append(&mut self.cause.encode());
		self.offending_ie.as_ref().map(|o| result.append(&mut o.encode()));
		self.up_f_seid.as_ref().map(|o| result.append(&mut o.encode()));
		self.created_pdr.iter().for_each(|o| result.append(&mut o.encode()));

		self.ago_perf.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut cause = None;
		let mut offending_ie = None;
		let mut up_f_seid = None;
		let mut created_pdr = vec![];
		let mut ago_perf = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }

				OffendingIE::ID => { offending_ie = Some(OffendingIE::decode(curmsg)?); }
				F_SEID::ID => { up_f_seid = Some(F_SEID::decode(curmsg)?); }
				CreatedPDR::ID => { created_pdr.push(CreatedPDR::decode(curmsg)?); }

				AgoUpfPerfReport::ID => { ago_perf = Some(AgoUpfPerfReport::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			offending_ie: offending_ie,
			up_f_seid: up_f_seid,
			created_pdr: created_pdr,

			ago_perf
		})
	}
}
pub struct PFCPSessionDeletionRequest {

}
impl PFCPModel for PFCPSessionDeletionRequest {
	const ID: u16 = 54;

	fn encode(&self) -> Vec<u8> {
		vec![]
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		Ok(Self {

		})
	}
}

pub struct PFCPSessionDeletionResponse {
	pub cause: Cause,
	pub offending_ie: Option<OffendingIE>,
	// TODO: Load Control Information
	// TODO: Overload Control Information
	pub usage_report: Vec<UsageReportSessionDeletion>,
	// TODO: Additional Usage Reports Information
	// TODO: Packet Rate Status Report
	// TODO: Session Report
	// TODO: MBS Session N4 Information
	pub ago_perf: Option<AgoUpfPerfReport>
}
impl PFCPModel for PFCPSessionDeletionResponse {
	const ID: u16 = 55;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.cause.encode());
		self.offending_ie.as_ref().map(|o| result.append(&mut o.encode()));
		self.usage_report.iter().for_each(|o| result.append(&mut o.encode()));
		self.ago_perf.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut cause = None;
		let mut offending_ie = None;
		let mut usage_report = vec![];
		let mut ago_perf = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }

				OffendingIE::ID => { offending_ie = Some(OffendingIE::decode(curmsg)?); }
				UsageReportSessionDeletion::ID => { usage_report.push(UsageReportSessionDeletion::decode(curmsg)?); }

				AgoUpfPerfReport::ID => { ago_perf = Some(AgoUpfPerfReport::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			offending_ie,
			usage_report,

			ago_perf
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemovePDR {
	pub pdr_id: PDR_ID
}
impl PFCPModel for RemovePDR {
	const ID: u16 = 15;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.pdr_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut pdr_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				PDR_ID::ID => { pdr_id = Some(PDR_ID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			pdr_id: pdr_id.ok_or(PFCPError::new("Missing mandatory field PDR_ID"))?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemoveFAR {
	pub far_id: FAR_ID
}
impl PFCPModel for RemoveFAR {
	const ID: u16 = 16;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.far_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut far_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				FAR_ID::ID => { far_id = Some(FAR_ID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			far_id: far_id.ok_or(PFCPError::new("Missing mandatory field FAR_ID"))?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemoveURR {
	pub urr_id: URR_ID
}
impl PFCPModel for RemoveURR {
	const ID: u16 = 17;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.urr_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut urr_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				URR_ID::ID => { urr_id = Some(URR_ID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			urr_id: urr_id.ok_or(PFCPError::new("Missing mandatory field URR_ID"))?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemoveQER {
	pub qer_id: QER_ID
}
impl PFCPModel for RemoveQER {
	const ID: u16 = 18;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.qer_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut qer_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				QER_ID::ID => { qer_id = Some(QER_ID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			qer_id: qer_id.ok_or(PFCPError::new("Missing mandatory field QER_ID"))?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemoveBAR {
	pub bar_id: BAR_ID
}
impl PFCPModel for RemoveBAR {
	const ID: u16 = 87;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.bar_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut bar_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			bar_id: bar_id.ok_or(PFCPError::new("Missing mandatory field BAR_ID"))?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RemoveTrafficEndpoint {
	pub traffic_endpoint_id: TrafficEndpointID
}
impl PFCPModel for RemoveTrafficEndpoint {
	const ID: u16 = 130;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.traffic_endpoint_id.encode());
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut traffic_endpoint_id = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				TrafficEndpointID::ID => { traffic_endpoint_id = Some(TrafficEndpointID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			traffic_endpoint_id: traffic_endpoint_id.ok_or(PFCPError::new("Missing mandatory field TrafficEndpointID"))?,
		})
	}
}

macro_rules! UpdateRules {
	(m $field:ident, $old:ident, $new:ident) => {
		{ if $old.$field != $new.$field { Some($new.$field.clone()) } else { None } }
	};
	(o $field:ident, $old:ident, $new:ident) => {
		{ if $old.$field != $new.$field { $new.$field.as_ref().map_or(None, |o| Some(o.clone())) } else { None } }
	};
	(v $field:ident, $old:ident, $new:ident) => {
		{ if $old.$field != $new.$field { $new.$field.clone() } else { vec![] } }
	};
}

#[derive(Debug, Clone)]
pub struct UpdatePDR {
	pub pdr_id: PDR_ID,
	pub precedence: Option<Precedence>,
	pub pdi: Option<PDI>,
	pub outer_header_removal: Option<OuterHeaderRemoval>,
	pub far_id: Option<FAR_ID>,
	pub urr_id: Vec<URR_ID>,
	pub qer_id: Option<QER_ID>,
	pub activate_predefined_rules: Vec<ActivatePredefinedRules>,
	pub deactivate_predefined_rules: Vec<DeactivatePredefinedRules>,
	pub activation_time: Option<ActivationTime>,
	pub deactivation_time: Option<DeactivationTime>,
	pub ip_multicast_addressing_info: Vec<IPMulticastAddressingInfo>,
	pub transport_delay_reporting: Option<TransportDelayReporting>,
}
impl UpdatePDR {
	// pub fn from_create_pdr(old_pdr: &CreatePDR, pdr: &CreatePDR) -> UpdatePDR {
	// 	UpdatePDR {
	// 		pdr_id: old_pdr.pdr_id.clone(),
	// 		//precedence: if old_pdr.precedence != pdr.precedence { pdr.precedence.clone() } else { None },
	// 		precedence: UpdateRules!(m precedence, old_pdr, pdr),
	// 		pdi: UpdateRules!(m pdi, old_pdr, pdr),
	// 		outer_header_removal: UpdateRules!(o outer_header_removal, old_pdr, pdr),
	// 		far_id: UpdateRules!(o far_id, old_pdr, pdr),
	// 		urr_id: UpdateRules!(v urr_id, old_pdr, pdr),
	// 		qer_id: UpdateRules!(o qer_id, old_pdr, pdr),
	// 		activate_predefined_rules: UpdateRules!(v activate_predefined_rules, old_pdr, pdr),
	// 		deactivate_predefined_rules: UpdateRules!(v deactivate_predefined_rules, old_pdr, pdr),
	// 		activation_time: UpdateRules!(o activation_time, old_pdr, pdr),
	// 		deactivation_time: UpdateRules!(o deactivation_time, old_pdr, pdr),
	// 		ip_multicast_addressing_info: UpdateRules!(v ip_multicast_addressing_info, old_pdr, pdr),
	// 		transport_delay_reporting: UpdateRules!(o transport_delay_reporting, old_pdr, pdr),
	// 	}
	// }
	pub fn assign_new_update(&mut self, new_update: &UpdatePDR) {
		new_update.precedence.as_ref().map(|o| self.precedence = Some(o.clone()));
		new_update.pdi.as_ref().map(|o| self.pdi = Some(o.clone()));
		new_update.outer_header_removal.as_ref().map(|o| self.outer_header_removal = Some(o.clone()));
		new_update.far_id.as_ref().map(|o| self.far_id = Some(o.clone()));
		self.urr_id = new_update.urr_id.clone();
		new_update.qer_id.as_ref().map(|o| self.qer_id = Some(o.clone()));
		self.activate_predefined_rules = new_update.activate_predefined_rules.clone();
		self.deactivate_predefined_rules = new_update.deactivate_predefined_rules.clone();
		new_update.activation_time.as_ref().map(|o| self.activation_time = Some(o.clone()));
		new_update.deactivation_time.as_ref().map(|o| self.deactivation_time = Some(o.clone()));
		self.ip_multicast_addressing_info = new_update.ip_multicast_addressing_info.clone();
		new_update.transport_delay_reporting.as_ref().map(|o| self.transport_delay_reporting = Some(o.clone()));
	}
}
impl PFCPModel for UpdatePDR {
	const ID: u16 = 9;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.pdr_id.encode());
		self.precedence.as_ref().map(|o| result.append(&mut o.encode()));
		self.pdi.as_ref().map(|o| result.append(&mut o.encode()));
		self.outer_header_removal.as_ref().map(|o| result.append(&mut o.encode()));
		self.far_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.urr_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.qer_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.activate_predefined_rules.iter().for_each(|o| result.append(&mut o.encode()));
		self.deactivate_predefined_rules.iter().for_each(|o| result.append(&mut o.encode()));
		self.activation_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.deactivation_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.ip_multicast_addressing_info.iter().for_each(|o| result.append(&mut o.encode()));
		self.transport_delay_reporting.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut pdr_id = None;
		let mut precedence = None;
		let mut pdi = None;
		let mut outer_header_removal = None;
		let mut far_id = None;
		let mut urr_id = vec![];
		let mut qer_id = None;
		let mut activate_predefined_rules = vec![];
		let mut deactivate_predefined_rules = vec![];
		let mut activation_time = None;
		let mut deactivation_time = None;
		let mut ip_multicast_addressing_info = vec![];
		let mut transport_delay_reporting = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				PDR_ID::ID => { pdr_id = Some(PDR_ID::decode(curmsg)?); }
				Precedence::ID => { precedence = Some(Precedence::decode(curmsg)?); }
				PDI::ID => { pdi = Some(PDI::decode(curmsg)?); }

				OuterHeaderRemoval::ID => { outer_header_removal = Some(OuterHeaderRemoval::decode(curmsg)?); }
				FAR_ID::ID => { far_id = Some(FAR_ID::decode(curmsg)?); }
				URR_ID::ID => { urr_id.push(URR_ID::decode(curmsg)?); }
				QER_ID::ID => { qer_id = Some(QER_ID::decode(curmsg)?); }
				ActivatePredefinedRules::ID => { activate_predefined_rules.push(ActivatePredefinedRules::decode(curmsg)?); }
				DeactivatePredefinedRules::ID => { deactivate_predefined_rules.push(DeactivatePredefinedRules::decode(curmsg)?); }
				ActivationTime::ID => { activation_time = Some(ActivationTime::decode(curmsg)?); }
				DeactivationTime::ID => { deactivation_time = Some(DeactivationTime::decode(curmsg)?); }
				IPMulticastAddressingInfo::ID => { ip_multicast_addressing_info.push(IPMulticastAddressingInfo::decode(curmsg)?); }
				TransportDelayReporting::ID => { transport_delay_reporting = Some(TransportDelayReporting::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			pdr_id: pdr_id.ok_or(PFCPError::new("Missing mandatory field PDR_ID"))?,

			precedence: precedence,
			pdi: pdi,
			outer_header_removal: outer_header_removal,
			far_id: far_id,
			urr_id: urr_id,
			qer_id: qer_id,
			activate_predefined_rules: activate_predefined_rules,
			deactivate_predefined_rules: deactivate_predefined_rules,
			activation_time: activation_time,
			deactivation_time: deactivation_time,
			ip_multicast_addressing_info: ip_multicast_addressing_info,
			transport_delay_reporting: transport_delay_reporting,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateForwardingParameters {
	pub destination_interface: Option<DestinationInterface>,
	pub network_instnace: Option<NetworkInstance>,
	pub redirect_information: Option<RedirectInformation>,
	pub outer_header_creation: Option<OuterHeaderCreation>,
	pub transport_level_marking: Option<TransportLevelMarking>,
	pub forwarding_policy: Option<ForwardingPolicy>,
	pub header_enrichment: Option<HeaderEnrichment>,
	pub pfcpsm_req_flags: Option<PFCPSMReqFlags>,
	pub linked_traffic_endpoint_id: Option<TrafficEndpointID>,
	pub destination_interface_type: Option<_3GPPInterfaceType>,
	pub data_network_access_identifier: Option<DataNetworkAccessIdentifier>
}
impl UpdateForwardingParameters {
	pub fn from_forwarding_parameters(old: &ForwardingParameters, new: &ForwardingParameters) -> UpdateForwardingParameters {
		UpdateForwardingParameters {
			destination_interface: UpdateRules!(m destination_interface, old, new),
			network_instnace: UpdateRules!(o network_instnace, old, new),
			redirect_information: UpdateRules!(o redirect_information, old, new),
			outer_header_creation: UpdateRules!(o outer_header_creation, old, new),
			transport_level_marking: UpdateRules!(o transport_level_marking, old, new),
			forwarding_policy: UpdateRules!(o forwarding_policy, old, new),
			header_enrichment: UpdateRules!(o header_enrichment, old, new),
			pfcpsm_req_flags: UpdateRules!(o pfcpsm_req_flags, old, new),
			linked_traffic_endpoint_id: UpdateRules!(o linked_traffic_endpoint_id, old, new),
			destination_interface_type: UpdateRules!(o destination_interface_type, old, new),
			data_network_access_identifier: UpdateRules!(o data_network_access_identifier, old, new),
		}
	}
}
impl PFCPModel for UpdateForwardingParameters {
	const ID: u16 = 11;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		self.destination_interface.as_ref().map(|o| result.append(&mut o.encode()));
		self.network_instnace.as_ref().map(|o| result.append(&mut o.encode()));
		self.redirect_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.outer_header_creation.as_ref().map(|o| result.append(&mut o.encode()));
		self.transport_level_marking.as_ref().map(|o| result.append(&mut o.encode()));
		self.forwarding_policy.as_ref().map(|o| result.append(&mut o.encode()));
		self.header_enrichment.as_ref().map(|o| result.append(&mut o.encode()));
		self.linked_traffic_endpoint_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.pfcpsm_req_flags.as_ref().map(|o| result.append(&mut o.encode()));
		self.destination_interface_type.as_ref().map(|o| result.append(&mut o.encode()));
		self.data_network_access_identifier.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut destination_interface = None;
		let mut network_instnace = None;
		let mut redirect_information = None;
		let mut outer_header_creation = None;
		let mut transport_level_marking = None;
		let mut forwarding_policy = None;
		let mut header_enrichment = None;
		let mut linked_traffic_endpoint_id = None;
		let mut pfcpsm_req_flags = None;
		let mut destination_interface_type = None;
		let mut data_network_access_identifier = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				DestinationInterface::ID => { destination_interface = Some(DestinationInterface::decode(curmsg)?); }

				NetworkInstance::ID => { network_instnace = Some(NetworkInstance::decode(curmsg)?); }
				RedirectInformation::ID => { redirect_information = Some(RedirectInformation::decode(curmsg)?); }
				OuterHeaderCreation::ID => { outer_header_creation = Some(OuterHeaderCreation::decode(curmsg)?); }
				TransportLevelMarking::ID => { transport_level_marking = Some(TransportLevelMarking::decode(curmsg)?); }
				ForwardingPolicy::ID => { forwarding_policy = Some(ForwardingPolicy::decode(curmsg)?); }
				HeaderEnrichment::ID => { header_enrichment = Some(HeaderEnrichment::decode(curmsg)?); }
				PFCPSMReqFlags::ID => { pfcpsm_req_flags = Some(PFCPSMReqFlags::decode(curmsg)?); }
				TrafficEndpointID::ID => { linked_traffic_endpoint_id = Some(TrafficEndpointID::decode(curmsg)?); }
				_3GPPInterfaceType::ID => { destination_interface_type = Some(_3GPPInterfaceType::decode(curmsg)?); }
				DataNetworkAccessIdentifier::ID => { data_network_access_identifier = Some(DataNetworkAccessIdentifier::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			destination_interface: destination_interface,
			
			network_instnace: network_instnace,
			redirect_information: redirect_information,
			outer_header_creation: outer_header_creation,
			transport_level_marking: transport_level_marking,
			forwarding_policy: forwarding_policy,
			header_enrichment: header_enrichment,
			linked_traffic_endpoint_id: linked_traffic_endpoint_id,
			pfcpsm_req_flags: pfcpsm_req_flags,
			destination_interface_type: destination_interface_type,
			data_network_access_identifier: data_network_access_identifier,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateDuplicatingParameters {
	pub destination_interface: Option<DestinationInterface>,
	pub outer_header_creation: Option<OuterHeaderCreation>,
	pub transport_level_marking: Option<TransportLevelMarking>,
	pub forwarding_policy: Option<ForwardingPolicy>,
}
impl PFCPModel for UpdateDuplicatingParameters {
	const ID: u16 = 105;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		self.destination_interface.as_ref().map(|o| result.append(&mut o.encode()));
		self.outer_header_creation.as_ref().map(|o| result.append(&mut o.encode()));
		self.transport_level_marking.as_ref().map(|o| result.append(&mut o.encode()));
		self.forwarding_policy.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut destination_interface = None;
		let mut outer_header_creation = None;
		let mut transport_level_marking = None;
		let mut forwarding_policy = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				DestinationInterface::ID => { destination_interface = Some(DestinationInterface::decode(curmsg)?); }
				OuterHeaderCreation::ID => { outer_header_creation = Some(OuterHeaderCreation::decode(curmsg)?); }
				TransportLevelMarking::ID => { transport_level_marking = Some(TransportLevelMarking::decode(curmsg)?); }
				ForwardingPolicy::ID => { forwarding_policy = Some(ForwardingPolicy::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			destination_interface: destination_interface,
			outer_header_creation: outer_header_creation,
			transport_level_marking: transport_level_marking,
			forwarding_policy: forwarding_policy,
		})
	}
}

#[derive(Debug, Clone)]
pub struct UpdateFAR {
	pub far_id: FAR_ID,
	pub apply_action: Option<ApplyAction>,
	pub update_forwarding_parameters: Option<UpdateForwardingParameters>,
	//pub update_duplicating_parameters: Vec<UpdateDuplicatingParameters>,
	pub bar_id: Option<BAR_ID>,
	pub redundant_transmission_forwarding_parameters: Option<RedundantTransmissionForwardingParameters>,
}
impl UpdateFAR {
	// pub fn from_create_far(old: &CreateFAR, new: &CreateFAR) -> UpdateFAR {
	// 	UpdateFAR {
	// 		far_id: old.far_id.clone(),
	// 		apply_action: UpdateRules!(m apply_action, old, new),
	// 		update_forwarding_parameters: new.update_forwarding_parameters.clone(),
	// 		//update_duplicating_parameters: new.update_duplicating_parameters.clone(),
	// 		bar_id: UpdateRules!(o bar_id, old, new),
	// 		redundant_transmission_forwarding_parameters: UpdateRules!(o redundant_transmission_forwarding_parameters, old, new),
	// 	}
	// }
	pub fn assign_new_update(&mut self, new_update: &UpdateFAR) {
		new_update.apply_action.as_ref().map(|o| self.apply_action = Some(o.clone()));
		new_update.update_forwarding_parameters.as_ref().map(|o| self.update_forwarding_parameters = Some(o.clone()));
		new_update.bar_id.as_ref().map(|o| self.bar_id = Some(o.clone()));
		new_update.redundant_transmission_forwarding_parameters.as_ref().map(|o| self.redundant_transmission_forwarding_parameters = Some(o.clone()));
	}
}
impl PFCPModel for UpdateFAR {
	const ID: u16 = 10;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.far_id.encode());
		self.apply_action.as_ref().map(|o| result.append(&mut o.encode()));
		self.update_forwarding_parameters.as_ref().map(|o| result.append(&mut o.encode()));
		//self.update_duplicating_parameters.iter().for_each(|o| result.append(&mut o.encode()));
		self.bar_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.redundant_transmission_forwarding_parameters.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut far_id = None;
		let mut apply_action = None;
		let mut update_forwarding_parameters = None;
		//let mut update_duplicating_parameters = vec![];
		let mut bar_id = None;
		let mut redundant_transmission_forwarding_parameters = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				FAR_ID::ID => { far_id = Some(FAR_ID::decode(curmsg)?); }

				ApplyAction::ID => { apply_action = Some(ApplyAction::decode(curmsg)?); }
				UpdateForwardingParameters::ID => { update_forwarding_parameters = Some(UpdateForwardingParameters::decode(curmsg)?); }
				//UpdateDuplicatingParameters::ID => { update_duplicating_parameters.push(UpdateDuplicatingParameters::decode(curmsg)?); }
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }
				RedundantTransmissionForwardingParameters::ID => { redundant_transmission_forwarding_parameters = Some(RedundantTransmissionForwardingParameters::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			far_id: far_id.ok_or(PFCPError::new("Missing mandatory field FAR_ID"))?,

			apply_action: apply_action,
			update_forwarding_parameters: update_forwarding_parameters,
			//update_duplicating_parameters: update_duplicating_parameters,
			bar_id: bar_id,
			redundant_transmission_forwarding_parameters: redundant_transmission_forwarding_parameters,
		})
	}
}

#[derive(Debug, Clone)]
pub struct UpdateURR {
	pub urr_id: URR_ID,
	pub measurement_method: Option<MeasurementMethod>,
	pub reporting_triggers: Option<ReportingTriggers>,
	pub measurement_period: Option<MeasurementPeriod>,
	pub volume_threshold: Option<VolumeThreshold>,
	pub volume_quota: Option<VolumeQuota>,
	pub event_threshold: Option<EventThreshold>,
	pub event_quota: Option<EventQuota>,
	pub time_threshold: Option<TimeThreshold>,
	pub time_quota: Option<TimeQuota>,
	pub quota_holding_time: Option<QuotaHoldingTime>,
	pub dropped_dl_traffic_threshold: Option<DroppedDLTrafficThreshold>,
	pub quota_validity_time: Option<QuotaValidityTime>,
	pub monitoring_time: Option<MonitoringTime>,
	pub subsequent_volume_threshold: Option<SubsequentVolumeThreshold>,
	pub subsequent_time_threshold: Option<SubsequentTimeThreshold>,
	pub subsequent_volume_quota: Option<SubsequentVolumeQuota>,
	pub subsequent_time_quota: Option<SubsequentTimeQuota>,
	pub subsequent_event_threshold: Option<SubsequentEventThreshold>,
	pub subsequent_event_quota: Option<SubsequentEventQuota>,
	pub inactivity_detection_time: Option<InactivityDetectionTime>,
	pub linked_urr_id: Vec<LinkedURR_ID>,
	pub measurement_information: Option<MeasurementInformation>,
	// (NOT N4): Time Quota Mechanism
	// (NOT N4): Aggregated URRs
	pub far_id_for_quota_action: Option<FAR_ID>,
	pub ethernet_inactivity_timer: Option<EthernetInactivityTimer>,
	pub additional_monitoring_time: Vec<AdditionalMonitoringTime>,
	pub number_of_reports: Option<NumberOfReports>,
	pub exempted_application_id_for_quota_action: Vec<ApplicationID>,
	pub exempted_sdf_filter_for_quota_action: Vec<SDFFilter>
}
impl UpdateURR {
	pub fn from_create_urr(old: &CreateURR, new: &CreateURR) -> UpdateURR {
		todo!()
	}
	pub fn assign_new_update(&mut self, new_update: &UpdateURR) {
		new_update.measurement_method.as_ref().map(|o| self.measurement_method.replace(o.clone()));
		new_update.reporting_triggers.as_ref().map(|o| self.reporting_triggers.replace(o.clone()));
		new_update.measurement_period.as_ref().map(|o| self.measurement_period.replace(o.clone()));
		new_update.volume_threshold.as_ref().map(|o| self.volume_threshold.replace(o.clone()));
		new_update.volume_quota.as_ref().map(|o| self.volume_quota.replace(o.clone()));
		new_update.event_threshold.as_ref().map(|o| self.event_threshold.replace(o.clone()));
		new_update.event_quota.as_ref().map(|o| self.event_quota.replace(o.clone()));
		new_update.time_threshold.as_ref().map(|o| self.time_threshold.replace(o.clone()));
		new_update.time_quota.as_ref().map(|o| self.time_quota.replace(o.clone()));
		new_update.quota_holding_time.as_ref().map(|o| self.quota_holding_time.replace(o.clone()));
		new_update.dropped_dl_traffic_threshold.as_ref().map(|o| self.dropped_dl_traffic_threshold.replace(o.clone()));
		new_update.quota_validity_time.as_ref().map(|o| self.quota_validity_time.replace(o.clone()));
		new_update.monitoring_time.as_ref().map(|o| self.monitoring_time.replace(o.clone()));
		new_update.subsequent_volume_threshold.as_ref().map(|o| self.subsequent_volume_threshold.replace(o.clone()));
		new_update.subsequent_time_threshold.as_ref().map(|o| self.subsequent_time_threshold.replace(o.clone()));
		new_update.subsequent_volume_quota.as_ref().map(|o| self.subsequent_volume_quota.replace(o.clone()));
		new_update.subsequent_time_quota.as_ref().map(|o| self.subsequent_time_quota.replace(o.clone()));
		new_update.subsequent_event_threshold.as_ref().map(|o| self.subsequent_event_threshold.replace(o.clone()));
		new_update.subsequent_event_quota.as_ref().map(|o| self.subsequent_event_quota.replace(o.clone()));
		new_update.inactivity_detection_time.as_ref().map(|o| self.inactivity_detection_time.replace(o.clone()));
		if new_update.linked_urr_id.len() != 0 { self.linked_urr_id = new_update.linked_urr_id.clone(); }
		new_update.measurement_information.as_ref().map(|o| self.measurement_information.replace(o.clone()));
		new_update.far_id_for_quota_action.as_ref().map(|o| self.far_id_for_quota_action.replace(o.clone()));
		new_update.ethernet_inactivity_timer.as_ref().map(|o| self.ethernet_inactivity_timer.replace(o.clone()));
		if new_update.additional_monitoring_time.len() != 0 { self.additional_monitoring_time = new_update.additional_monitoring_time.clone(); }
		new_update.number_of_reports.as_ref().map(|o| self.number_of_reports.replace(o.clone()));
		if new_update.exempted_application_id_for_quota_action.len() != 0 { self.exempted_application_id_for_quota_action = new_update.exempted_application_id_for_quota_action.clone(); }
		if new_update.exempted_sdf_filter_for_quota_action.len() != 0 { self.exempted_sdf_filter_for_quota_action = new_update.exempted_sdf_filter_for_quota_action.clone(); }
	}
}
impl PFCPModel for UpdateURR {
	const ID: u16 = 13;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.urr_id.encode());
		self.measurement_method.as_ref().map(|o| result.append(&mut o.encode()));
		self.reporting_triggers.as_ref().map(|o| result.append(&mut o.encode()));
		self.measurement_period.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.event_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.event_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.quota_holding_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.dropped_dl_traffic_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.quota_validity_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.monitoring_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_volume_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_volume_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_time_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_threshold.as_ref().map(|o| result.append(&mut o.encode()));
		self.subsequent_event_quota.as_ref().map(|o| result.append(&mut o.encode()));
		self.inactivity_detection_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.linked_urr_id.iter().for_each(|o| result.append(&mut o.encode()));
		self.measurement_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.far_id_for_quota_action.as_ref().map(|o| result.append(&mut o.encode()));
		self.ethernet_inactivity_timer.as_ref().map(|o| result.append(&mut o.encode()));
		self.additional_monitoring_time.iter().for_each(|o| result.append(&mut o.encode()));
		self.number_of_reports.as_ref().map(|o| result.append(&mut o.encode()));
		self.exempted_application_id_for_quota_action.iter().for_each(|o| result.append(&mut o.encode()));
		self.exempted_sdf_filter_for_quota_action.iter().for_each(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut urr_id = None;
		let mut measurement_method = None;
		let mut reporting_triggers = None;
		let mut measurement_period = None;
		let mut volume_threshold = None;
		let mut volume_quota = None;
		let mut event_threshold = None;
		let mut event_quota = None;
		let mut time_threshold = None;
		let mut time_quota = None;
		let mut quota_holding_time = None;
		let mut dropped_dl_traffic_threshold = None;
		let mut quota_validity_time = None;
		let mut monitoring_time = None;
		let mut subsequent_volume_threshold = None;
		let mut subsequent_time_threshold = None;
		let mut subsequent_volume_quota = None;
		let mut subsequent_time_quota = None;
		let mut subsequent_event_threshold = None;
		let mut subsequent_event_quota = None;
		let mut inactivity_detection_time = None;
		let mut linked_urr_id = vec![];
		let mut measurement_information = None;
		let mut far_id_for_quota_action = None;
		let mut ethernet_inactivity_timer = None;
		let mut additional_monitoring_time = vec![];
		let mut number_of_reports = None;
		let mut exempted_application_id_for_quota_action = vec![];
		let mut exempted_sdf_filter_for_quota_action = vec![];
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				URR_ID::ID => { urr_id = Some(URR_ID::decode(curmsg)?); }
				MeasurementMethod::ID => { measurement_method = Some(MeasurementMethod::decode(curmsg)?); }
				ReportingTriggers::ID => { reporting_triggers = Some(ReportingTriggers::decode(curmsg)?); }
				MeasurementPeriod::ID => { measurement_period = Some(MeasurementPeriod::decode(curmsg)?); }
				VolumeThreshold::ID => { volume_threshold = Some(VolumeThreshold::decode(curmsg)?); }
				VolumeQuota::ID => { volume_quota = Some(VolumeQuota::decode(curmsg)?); }
				EventThreshold::ID => { event_threshold = Some(EventThreshold::decode(curmsg)?); }
				EventQuota::ID => { event_quota = Some(EventQuota::decode(curmsg)?); }
				TimeThreshold::ID => { time_threshold = Some(TimeThreshold::decode(curmsg)?); }
				TimeQuota::ID => { time_quota = Some(TimeQuota::decode(curmsg)?); }
				QuotaHoldingTime::ID => { quota_holding_time = Some(QuotaHoldingTime::decode(curmsg)?); }
				DroppedDLTrafficThreshold::ID => { dropped_dl_traffic_threshold = Some(DroppedDLTrafficThreshold::decode(curmsg)?); }
				QuotaValidityTime::ID => { quota_validity_time = Some(QuotaValidityTime::decode(curmsg)?); }
				MonitoringTime::ID => { monitoring_time = Some(MonitoringTime::decode(curmsg)?); }
				SubsequentVolumeThreshold::ID => { subsequent_volume_threshold = Some(SubsequentVolumeThreshold::decode(curmsg)?); }
				SubsequentTimeThreshold::ID => { subsequent_time_threshold = Some(SubsequentTimeThreshold::decode(curmsg)?); }
				SubsequentVolumeQuota::ID => { subsequent_volume_quota = Some(SubsequentVolumeQuota::decode(curmsg)?); }
				SubsequentTimeQuota::ID => { subsequent_time_quota = Some(SubsequentTimeQuota::decode(curmsg)?); }
				SubsequentEventThreshold::ID => { subsequent_event_threshold = Some(SubsequentEventThreshold::decode(curmsg)?); }
				SubsequentEventQuota::ID => { subsequent_event_quota = Some(SubsequentEventQuota::decode(curmsg)?); }
				InactivityDetectionTime::ID => { inactivity_detection_time = Some(InactivityDetectionTime::decode(curmsg)?); }
				LinkedURR_ID::ID => { linked_urr_id.push(LinkedURR_ID::decode(curmsg)?); }
				MeasurementInformation::ID => { measurement_information = Some(MeasurementInformation::decode(curmsg)?); }
				FAR_ID::ID => { far_id_for_quota_action = Some(FAR_ID::decode(curmsg)?); }
				EthernetInactivityTimer::ID => { ethernet_inactivity_timer = Some(EthernetInactivityTimer::decode(curmsg)?); }
				AdditionalMonitoringTime::ID => { additional_monitoring_time.push(AdditionalMonitoringTime::decode(curmsg)?); }
				NumberOfReports::ID => { number_of_reports = Some(NumberOfReports::decode(curmsg)?); }
				ApplicationID::ID => { exempted_application_id_for_quota_action.push(ApplicationID::decode(curmsg)?); }
				SDFFilter::ID => { exempted_sdf_filter_for_quota_action.push(SDFFilter::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			urr_id: urr_id.ok_or(PFCPError::new("Missing mandatory field URR_ID"))?,

			measurement_method,
			reporting_triggers,
			measurement_period,
			volume_threshold,
			volume_quota,
			event_threshold,
			event_quota,
			time_threshold,
			time_quota,
			quota_holding_time,
			dropped_dl_traffic_threshold,
			quota_validity_time,
			monitoring_time,
			subsequent_volume_threshold,
			subsequent_time_threshold,
			subsequent_volume_quota,
			subsequent_time_quota,
			subsequent_event_threshold,
			subsequent_event_quota,
			inactivity_detection_time,
			linked_urr_id,
			measurement_information,
			far_id_for_quota_action,
			ethernet_inactivity_timer,
			additional_monitoring_time,
			number_of_reports,
			exempted_application_id_for_quota_action,
			exempted_sdf_filter_for_quota_action
		})
	}
}

#[derive(Debug, Clone)]
pub struct UpdateQER {
	pub qer_id: QER_ID,
	pub qer_corrleation_id: Option<QERCorrelationID>,
	pub gate_status: Option<GateStatus>,
	pub maximum_bitrate: Option<MBR>,
	pub guaranteed_bitrate: Option<GBR>,
	// MAYBE TODO: Packet Rate Status
	pub qfi: Option<QFI>,
	pub rqi: Option<RQI>,
	pub paging_policy_indicator: Option<PagingPolicyIndicator>,
	pub averaging_window: Option<AveragingWindow>,
	pub qer_control_indications: Option<QERControlIndications>
}
impl UpdateQER {
	pub fn from_create_qer(old: &CreateQER, new: &CreateQER) -> UpdateQER {
		todo!()
	}
	pub fn assign_new_update(&mut self, new_update: &UpdateQER) {
		new_update.qer_corrleation_id.as_ref().map(|o| self.qer_corrleation_id = Some(o.clone()));
		new_update.gate_status.as_ref().map(|o| self.gate_status = Some(o.clone()));
		new_update.maximum_bitrate.as_ref().map(|o| self.maximum_bitrate = Some(o.clone()));
		new_update.guaranteed_bitrate.as_ref().map(|o| self.guaranteed_bitrate = Some(o.clone()));
		new_update.qfi.as_ref().map(|o| self.qfi = Some(o.clone()));
		new_update.rqi.as_ref().map(|o| self.rqi = Some(o.clone()));
		new_update.paging_policy_indicator.as_ref().map(|o| self.paging_policy_indicator = Some(o.clone()));
		new_update.averaging_window.as_ref().map(|o| self.averaging_window = Some(o.clone()));
		new_update.qer_control_indications.as_ref().map(|o| self.qer_control_indications = Some(o.clone()));
	}
}
impl PFCPModel for UpdateQER {
	const ID: u16 = 14;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.qer_id.encode());
		self.qer_corrleation_id.as_ref().map(|o| result.append(&mut o.encode()));
		self.gate_status.as_ref().map(|o| result.append(&mut o.encode()));
		self.maximum_bitrate.as_ref().map(|o| result.append(&mut o.encode()));
		self.guaranteed_bitrate.as_ref().map(|o| result.append(&mut o.encode()));
		self.qfi.as_ref().map(|o| result.append(&mut o.encode()));
		self.rqi.as_ref().map(|o| result.append(&mut o.encode()));
		self.paging_policy_indicator.as_ref().map(|o| result.append(&mut o.encode()));
		self.averaging_window.as_ref().map(|o| result.append(&mut o.encode()));
		self.qer_control_indications.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut qer_id = None;
		let mut qer_corrleation_id = None;
		let mut gate_status = None;
		let mut maximum_bitrate = None;
		let mut guaranteed_bitrate = None;
		let mut qfi = None;
		let mut rqi = None;
		let mut paging_policy_indicator = None;
		let mut averaging_window = None;
		let mut qer_control_indications = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				QER_ID::ID => { qer_id = Some(QER_ID::decode(curmsg)?); }

				GateStatus::ID => { gate_status = Some(GateStatus::decode(curmsg)?); }
				QERCorrelationID::ID => { qer_corrleation_id = Some(QERCorrelationID::decode(curmsg)?); }
				MBR::ID => { maximum_bitrate = Some(MBR::decode(curmsg)?); }
				GBR::ID => { guaranteed_bitrate = Some(GBR::decode(curmsg)?); }
				QFI::ID => { qfi = Some(QFI::decode(curmsg)?); }
				RQI::ID => { rqi = Some(RQI::decode(curmsg)?); }
				PagingPolicyIndicator::ID => { paging_policy_indicator = Some(PagingPolicyIndicator::decode(curmsg)?); }
				AveragingWindow::ID => { averaging_window = Some(AveragingWindow::decode(curmsg)?); }
				QERControlIndications::ID => { qer_control_indications = Some(QERControlIndications::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			qer_id: qer_id.ok_or(PFCPError::new("Missing mandatory field QER_ID"))?,

			gate_status: gate_status,
			qer_corrleation_id: qer_corrleation_id,
			maximum_bitrate: maximum_bitrate,
			guaranteed_bitrate: guaranteed_bitrate,
			qfi: qfi,
			rqi: rqi,
			paging_policy_indicator: paging_policy_indicator,
			averaging_window: averaging_window,
			qer_control_indications: qer_control_indications,
		})
	}
}

#[derive(Debug, Clone)]
pub struct UpdateBAR {
	pub bar_id: BAR_ID,
	pub downlink_data_notification_delay: Option<DownlinkDataNotificationDelay>,
	pub suggested_buffering_packets_count: Option<SuggestedBufferingPacketsCount>,
	// MT-EDT Control Information not part of N4
}
impl UpdateBAR {
	pub fn from_create_bar(old: &CreateBAR, new: &CreateBAR) -> UpdateBAR {
		todo!()
	}
	pub fn assign_new_update(&mut self, new_update: &UpdateBAR) {
		new_update.downlink_data_notification_delay.as_ref().map(|o| self.downlink_data_notification_delay.replace(o.clone()));
		new_update.suggested_buffering_packets_count.as_ref().map(|o| self.suggested_buffering_packets_count.replace(o.clone()));
	}
}
impl PFCPModel for UpdateBAR {
	const ID: u16 = 86;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.bar_id.encode());
		self.downlink_data_notification_delay.as_ref().map(|o| result.append(&mut o.encode()));
		self.suggested_buffering_packets_count.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut bar_id = None;
		let mut downlink_data_notification_delay = None;
		let mut suggested_buffering_packets_count = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }

				DownlinkDataNotificationDelay::ID => { downlink_data_notification_delay = Some(DownlinkDataNotificationDelay::decode(curmsg)?); }
				SuggestedBufferingPacketsCount::ID => { suggested_buffering_packets_count = Some(SuggestedBufferingPacketsCount::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			bar_id: bar_id.ok_or(PFCPError::new("Missing mandatory field BAR_ID"))?,
			
			downlink_data_notification_delay: downlink_data_notification_delay,
			suggested_buffering_packets_count: suggested_buffering_packets_count
		})
	}
}

#[derive(Debug, Clone)]
pub struct UpdateTrafficEndpoint {

}
impl PFCPModel for UpdateTrafficEndpoint {
	const ID: u16 = 129;

	fn encode(&self) -> Vec<u8> {
		todo!()
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		todo!()
	}
}

pub struct PFCPSessionModificationRequest {
	pub cp_f_seid: Option<F_SEID>,
	pub remove_pdr: Vec<RemovePDR>,
	pub remove_far: Vec<RemoveFAR>,
	pub remove_urr: Vec<RemoveURR>,
	pub remove_qer: Vec<RemoveQER>,
	pub remove_bar: Option<RemoveBAR>,
	pub remove_traffic_endpoint: Option<RemoveTrafficEndpoint>,
	pub create_pdr: Vec<CreatePDR>,
	pub create_far: Vec<CreateFAR>,
	pub create_urr: Vec<CreateURR>,
	pub create_qer: Vec<CreateQER>,
	pub create_bar: Option<CreateBAR>,
	pub create_traffic_endpoint: Option<CreateTrafficEndpoint>,
	pub update_pdr: Vec<UpdatePDR>,
	pub update_far: Vec<UpdateFAR>,
	pub update_urr: Vec<UpdateURR>,
	pub update_qer: Vec<UpdateQER>,
	pub update_bar: Option<UpdateBAR>,
	pub update_traffic_endpoint: Option<UpdateTrafficEndpoint>,
	// User Plane Inactivity Timer
	// Query URR Reference
	// Trace Information
	// Remove MAR
	// Create MAR
	// Update MAR
	// Node ID
	// TSC Management Information
	// Remove SRR
	// Create SRR
	// Update SRR
	// Provide ATSSS Control Information
	// Ethernet Context Information
	// Access Availability Information
	// Query Packet Rate Status
}
impl PFCPModel for PFCPSessionModificationRequest {
	const ID: u16 = 52;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		self.cp_f_seid.as_ref().map(|o| result.append(&mut o.encode()));
		self.remove_pdr.iter().for_each(|o| result.append(&mut o.encode()));
		self.remove_far.iter().for_each(|o| result.append(&mut o.encode()));
		self.remove_urr.iter().for_each(|o| result.append(&mut o.encode()));
		self.remove_qer.iter().for_each(|o| result.append(&mut o.encode()));
		self.remove_bar.as_ref().map(|o| result.append(&mut o.encode()));
		self.remove_traffic_endpoint.as_ref().map(|o| result.append(&mut o.encode()));
		self.create_pdr.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_far.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_urr.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_qer.iter().for_each(|o| result.append(&mut o.encode()));
		self.create_bar.as_ref().map(|o| result.append(&mut o.encode()));
		self.create_traffic_endpoint.as_ref().map(|o| result.append(&mut o.encode()));
		self.update_pdr.iter().for_each(|o| result.append(&mut o.encode()));
		self.update_far.iter().for_each(|o| result.append(&mut o.encode()));
		self.update_urr.iter().for_each(|o| result.append(&mut o.encode()));
		self.update_qer.iter().for_each(|o| result.append(&mut o.encode()));
		self.update_bar.as_ref().map(|o| result.append(&mut o.encode()));
		self.update_traffic_endpoint.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut cp_f_seid = None;
		let mut remove_pdr = vec![];
		let mut remove_far = vec![];
		let mut remove_urr = vec![];
		let mut remove_qer = vec![];
		let mut remove_bar = None;
		let mut remove_traffic_endpoint = None;
		let mut update_pdr = vec![];
		let mut update_far = vec![];
		let mut update_urr = vec![];
		let mut update_qer = vec![];
		let mut update_bar = None;
		let mut update_traffic_endpoint = None;
		let mut create_pdr = vec![];
		let mut create_far = vec![];
		let mut create_urr = vec![];
		let mut create_qer = vec![];
		let mut create_bar = None;
		let mut create_traffic_endpoint = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				F_SEID::ID => { cp_f_seid = Some(F_SEID::decode(curmsg)?); }

				RemovePDR::ID => { remove_pdr.push(RemovePDR::decode(curmsg)?); }
				RemoveFAR::ID => { remove_far.push(RemoveFAR::decode(curmsg)?); }
				RemoveURR::ID => { remove_urr.push(RemoveURR::decode(curmsg)?); }
				RemoveQER::ID => { remove_qer.push(RemoveQER::decode(curmsg)?); }
				RemoveBAR::ID => { remove_bar = Some(RemoveBAR::decode(curmsg)?); }
				RemoveTrafficEndpoint::ID => { remove_traffic_endpoint = Some(RemoveTrafficEndpoint::decode(curmsg)?); }
				UpdatePDR::ID => { update_pdr.push(UpdatePDR::decode(curmsg)?); }
				UpdateFAR::ID => { update_far.push(UpdateFAR::decode(curmsg)?); }
				UpdateURR::ID => { update_urr.push(UpdateURR::decode(curmsg)?); }
				UpdateQER::ID => { update_qer.push(UpdateQER::decode(curmsg)?); }
				UpdateBAR::ID => { update_bar = Some(UpdateBAR::decode(curmsg)?); }
				UpdateTrafficEndpoint::ID => { update_traffic_endpoint = Some(UpdateTrafficEndpoint::decode(curmsg)?); }
				CreatePDR::ID => { create_pdr.push(CreatePDR::decode(curmsg)?); }
				CreateFAR::ID => { create_far.push(CreateFAR::decode(curmsg)?); }
				CreateURR::ID => { create_urr.push(CreateURR::decode(curmsg)?); }
				CreateQER::ID => { create_qer.push(CreateQER::decode(curmsg)?); }
				CreateBAR::ID => { create_bar = Some(CreateBAR::decode(curmsg)?); }
				CreateTrafficEndpoint::ID => { create_traffic_endpoint = Some(CreateTrafficEndpoint::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			cp_f_seid: cp_f_seid,
			
			remove_pdr: remove_pdr,
			remove_far: remove_far,
			remove_urr: remove_urr,
			remove_qer: remove_qer,
			remove_bar: remove_bar,
			remove_traffic_endpoint: remove_traffic_endpoint,
			update_pdr: update_pdr,
			update_far: update_far,
			update_urr: update_urr,
			update_qer: update_qer,
			update_bar: update_bar,
			update_traffic_endpoint: update_traffic_endpoint,
			create_pdr: create_pdr,
			create_far: create_far,
			create_urr: create_urr,
			create_qer: create_qer,
			create_bar: create_bar,
			create_traffic_endpoint: create_traffic_endpoint,
		})
	}
}

pub struct PFCPSessionModificationResponse {
	pub cause: Cause,
	pub offending_ie: Option<OffendingIE>,
	pub created_pdr: Vec<CreatedPDR>,
	pub ago_perf: Option<AgoUpfPerfReport>
}
impl PFCPModel for PFCPSessionModificationResponse {
	const ID: u16 = 53;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.cause.encode());
		self.offending_ie.as_ref().map(|o| result.append(&mut o.encode()));
		self.created_pdr.iter().for_each(|o| result.append(&mut o.encode()));
		self.ago_perf.iter().for_each(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut cause = None;
		let mut offending_ie = None;
		let mut created_pdr = vec![];
		let mut ago_perf = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }

				OffendingIE::ID => { offending_ie = Some(OffendingIE::decode(curmsg)?); }
				CreatedPDR::ID => { created_pdr.push(CreatedPDR::decode(curmsg)?); }
				AgoUpfPerfReport::ID => { ago_perf = Some(AgoUpfPerfReport::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			offending_ie: offending_ie,
			created_pdr: created_pdr,
			ago_perf: ago_perf
		})
	}
}


pub struct DownlinkDataReport {
	pub pdr_id: PDR_ID,
	pub downlink_data_service_information: Option<DownlinkDataServiceInformation>,
	// DL Data Packets Size not used on N4
	pub data_status: Option<DataStatus>
}
impl PFCPModel for DownlinkDataReport {
	const ID: u16 = 83;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.pdr_id.encode());
		self.downlink_data_service_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.data_status.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut pdr_id = None;
		let mut downlink_data_service_information = None;
		let mut data_status = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				PDR_ID::ID => { pdr_id = Some(PDR_ID::decode(curmsg)?); }

				DownlinkDataServiceInformation::ID => { downlink_data_service_information = Some(DownlinkDataServiceInformation::decode(curmsg)?); }
				DataStatus::ID => { data_status = Some(DataStatus::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			pdr_id: pdr_id.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			downlink_data_service_information: downlink_data_service_information,
			data_status: data_status,
		})
	}
}


#[derive(Clone, Debug)]
pub struct UsageReport {
	pub urr_id: URR_ID,
	pub ur_seqn: UR_SEQN,
	pub usage_report_trigger: UsageReportTrigger,
	pub start_time: Option<StartTime>,
	pub end_time: Option<EndTime>,
	pub volume_measurement: Option<VolumeMeasurement>,
	pub duration_measurement: Option<DurationMeasurement>,
	// TODO: Application Detection Information
	pub ue_ip_address: Option<UE_IPAddress>,
	pub network_instance: Option<NetworkInstance>,
	pub time_of_first_packet: Option<TimeOfFirstPacket>,
	pub time_of_last_packet: Option<TimeOfLastPacket>,
	pub usage_information: Option<UsageInformation>,
	// TODO: Query URR Reference
	// TODO: Event Time Stamp
	// TODO: Ethernet Traffic Information
	// TODO: Join IP Muticast Information
	// TODO: Leave IP Muticast Information
	pub predefined_rules_name: Option<PredefinedRulesName>
}
impl UsageReport {
	pub fn to_deletion_report(&self) -> UsageReportSessionDeletion {
		UsageReportSessionDeletion {
			urr_id: self.urr_id.clone(),
			ur_seqn: self.ur_seqn.clone(),
			usage_report_trigger: self.usage_report_trigger.clone(),
			start_time: self.start_time.clone(),
			end_time: self.end_time.clone(),
			volume_measurement: self.volume_measurement.clone(),
			duration_measurement: self.duration_measurement.clone(),
			time_of_first_packet: self.time_of_first_packet.clone(),
			time_of_last_packet: self.time_of_last_packet.clone(),
			usage_information: self.usage_information.clone(),
		}
	}
}
impl PFCPModel for UsageReport {
	const ID: u16 = 80;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.urr_id.encode());
		result.append(&mut self.ur_seqn.encode());
		result.append(&mut self.usage_report_trigger.encode());
		self.start_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.end_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_measurement.as_ref().map(|o| result.append(&mut o.encode()));
		self.duration_measurement.as_ref().map(|o| result.append(&mut o.encode()));
		self.ue_ip_address.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_of_first_packet.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_of_last_packet.as_ref().map(|o| result.append(&mut o.encode()));
		self.network_instance.as_ref().map(|o| result.append(&mut o.encode()));
		self.usage_information.as_ref().map(|o| result.append(&mut o.encode()));
		self.predefined_rules_name.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut urr_id = None;
		let mut ur_seqn = None;
		let mut usage_report_trigger = None;
		let mut start_time = None;
		let mut end_time = None;
		let mut volume_measurement = None;
		let mut duration_measurement = None;
		let mut ue_ip_address = None;
		let mut network_instance = None;
		let mut time_of_first_packet = None;
		let mut time_of_last_packet = None;
		let mut usage_information = None;
		let mut predefined_rules_name = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				URR_ID::ID => { urr_id = Some(URR_ID::decode(curmsg)?); }
				UR_SEQN::ID => { ur_seqn = Some(UR_SEQN::decode(curmsg)?); }
				UsageReportTrigger::ID => { usage_report_trigger = Some(UsageReportTrigger::decode(curmsg)?); }

				StartTime::ID => { start_time = Some(StartTime::decode(curmsg)?); }
				EndTime::ID => { end_time = Some(EndTime::decode(curmsg)?); }
				VolumeMeasurement::ID => { volume_measurement = Some(VolumeMeasurement::decode(curmsg)?); }
				DurationMeasurement::ID => { duration_measurement = Some(DurationMeasurement::decode(curmsg)?); }
				UE_IPAddress::ID => { ue_ip_address = Some(UE_IPAddress::decode(curmsg)?); }
				NetworkInstance::ID => { network_instance = Some(NetworkInstance::decode(curmsg)?); }
				TimeOfFirstPacket::ID => { time_of_first_packet = Some(TimeOfFirstPacket::decode(curmsg)?); }
				TimeOfLastPacket::ID => { time_of_last_packet = Some(TimeOfLastPacket::decode(curmsg)?); }
				UsageInformation::ID => { usage_information = Some(UsageInformation::decode(curmsg)?); }
				PredefinedRulesName::ID => { predefined_rules_name = Some(PredefinedRulesName::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			urr_id: urr_id.ok_or(PFCPError::new("Missing mandatory field URR_ID"))?,
			ur_seqn: ur_seqn.ok_or(PFCPError::new("Missing mandatory field UR_SEQN"))?,
			usage_report_trigger: usage_report_trigger.ok_or(PFCPError::new("Missing mandatory field UsageReportTrigger"))?,
			
			start_time,
			end_time,
			volume_measurement,
			duration_measurement,
			ue_ip_address,
			network_instance,
			time_of_first_packet,
			time_of_last_packet,
			usage_information,
			predefined_rules_name,
		})
	}
}


#[derive(Clone)]
pub struct UsageReportSessionDeletion {
	pub urr_id: URR_ID,
	pub ur_seqn: UR_SEQN,
	pub usage_report_trigger: UsageReportTrigger,
	pub start_time: Option<StartTime>,
	pub end_time: Option<EndTime>,
	pub volume_measurement: Option<VolumeMeasurement>,
	pub duration_measurement: Option<DurationMeasurement>,
	pub time_of_first_packet: Option<TimeOfFirstPacket>,
	pub time_of_last_packet: Option<TimeOfLastPacket>,
	pub usage_information: Option<UsageInformation>,
	// TODO: Ethernet Traffic Information
}
impl PFCPModel for UsageReportSessionDeletion {
	const ID: u16 = 79;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.urr_id.encode());
		result.append(&mut self.ur_seqn.encode());
		result.append(&mut self.usage_report_trigger.encode());
		self.start_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.end_time.as_ref().map(|o| result.append(&mut o.encode()));
		self.volume_measurement.as_ref().map(|o| result.append(&mut o.encode()));
		self.duration_measurement.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_of_first_packet.as_ref().map(|o| result.append(&mut o.encode()));
		self.time_of_last_packet.as_ref().map(|o| result.append(&mut o.encode()));
		self.usage_information.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut urr_id = None;
		let mut ur_seqn = None;
		let mut usage_report_trigger = None;
		let mut start_time = None;
		let mut end_time = None;
		let mut volume_measurement = None;
		let mut duration_measurement = None;
		let mut time_of_first_packet = None;
		let mut time_of_last_packet = None;
		let mut usage_information = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				URR_ID::ID => { urr_id = Some(URR_ID::decode(curmsg)?); }
				UR_SEQN::ID => { ur_seqn = Some(UR_SEQN::decode(curmsg)?); }
				UsageReportTrigger::ID => { usage_report_trigger = Some(UsageReportTrigger::decode(curmsg)?); }

				StartTime::ID => { start_time = Some(StartTime::decode(curmsg)?); }
				EndTime::ID => { end_time = Some(EndTime::decode(curmsg)?); }
				VolumeMeasurement::ID => { volume_measurement = Some(VolumeMeasurement::decode(curmsg)?); }
				DurationMeasurement::ID => { duration_measurement = Some(DurationMeasurement::decode(curmsg)?); }
				TimeOfFirstPacket::ID => { time_of_first_packet = Some(TimeOfFirstPacket::decode(curmsg)?); }
				TimeOfLastPacket::ID => { time_of_last_packet = Some(TimeOfLastPacket::decode(curmsg)?); }
				UsageInformation::ID => { usage_information = Some(UsageInformation::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			urr_id: urr_id.ok_or(PFCPError::new("Missing mandatory field URR_ID"))?,
			ur_seqn: ur_seqn.ok_or(PFCPError::new("Missing mandatory field UR_SEQN"))?,
			usage_report_trigger: usage_report_trigger.ok_or(PFCPError::new("Missing mandatory field UsageReportTrigger"))?,
			
			start_time,
			end_time,
			volume_measurement,
			duration_measurement,
			time_of_first_packet,
			time_of_last_packet,
			usage_information,
		})
	}
}

pub struct PFCPSessionReportRequest {
	pub report_type: ReportType,
	pub downlink_data_report: Option<DownlinkDataReport>,
	pub usage_report: Vec<UsageReport>,
	// TODO: Error Indication Report
	// TODO: Load Control Information
	// TODO: Overload Control Information
	// TODO: Additional Usage Reports Information
	// TODO: PFCPSRReq-Flags
	// TODO: Old CP F-SEID
	// TODO: Packet Rate Status Report
	// TODO: TSC ManagementInformation
	// TODO: Session Report
	pub cause: Option<Cause>
}
impl PFCPModel for PFCPSessionReportRequest {
	const ID: u16 = 56;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.report_type.encode());
		self.downlink_data_report.as_ref().map(|o| result.append(&mut o.encode()));
		self.usage_report.iter().for_each(|o| result.append(&mut o.encode()));
		self.cause.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut report_type = None;
		let mut downlink_data_report = None;
		let mut usage_report = vec![];
		let mut cause = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				ReportType::ID => { report_type = Some(ReportType::decode(curmsg)?); }

				DownlinkDataReport::ID => { downlink_data_report = Some(DownlinkDataReport::decode(curmsg)?); }
				UsageReport::ID => { usage_report.push(UsageReport::decode(curmsg)?); }
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			report_type: report_type.ok_or(PFCPError::new("Missing mandatory field ReportType"))?,
			
			downlink_data_report,
			usage_report,
			cause,
		})
	}
}


#[derive(Debug, Clone)]
pub struct UpdateBAR_SessionReportResponse {
	pub bar_id: BAR_ID,
	pub downlink_data_notification_delay: Option<DownlinkDataNotificationDelay>,
	pub dl_buffering_duration: Option<DLBufferingDuration>,
	pub dl_buffering_suggested_packet_count: Option<DLBufferingSuggestedPacketCount>,
	pub suggested_buffering_packets_count: Option<SuggestedBufferingPacketsCount>,
	// MT-EDT Control Information not part of N4
}
impl PFCPModel for UpdateBAR_SessionReportResponse {
	const ID: u16 = 12;

	fn encode(&self) -> Vec<u8> {
		let mut result = Self::ID.to_be_bytes().to_vec();
		result.append(&mut 0u16.to_be_bytes().to_vec());
		result.append(&mut self.bar_id.encode());
		self.downlink_data_notification_delay.as_ref().map(|o| result.append(&mut o.encode()));
		self.dl_buffering_duration.as_ref().map(|o| result.append(&mut o.encode()));
		self.dl_buffering_suggested_packet_count.as_ref().map(|o| result.append(&mut o.encode()));
		self.suggested_buffering_packets_count.as_ref().map(|o| result.append(&mut o.encode()));
		let length: u16 = result.len() as u16 - 4; let length_be = length.to_be_bytes(); result[2] = length_be[0]; result[3] = length_be[1];
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut bar_id = None;
		let mut downlink_data_notification_delay = None;
		let mut dl_buffering_duration = None;
		let mut dl_buffering_suggested_packet_count = None;
		let mut suggested_buffering_packets_count = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				BAR_ID::ID => { bar_id = Some(BAR_ID::decode(curmsg)?); }

				DownlinkDataNotificationDelay::ID => { downlink_data_notification_delay = Some(DownlinkDataNotificationDelay::decode(curmsg)?); }
				DLBufferingDuration::ID => { dl_buffering_duration = Some(DLBufferingDuration::decode(curmsg)?); }
				DLBufferingSuggestedPacketCount::ID => { dl_buffering_suggested_packet_count = Some(DLBufferingSuggestedPacketCount::decode(curmsg)?); }
				SuggestedBufferingPacketsCount::ID => { suggested_buffering_packets_count = Some(SuggestedBufferingPacketsCount::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			bar_id: bar_id.ok_or(PFCPError::new("Missing mandatory field BAR_ID"))?,
			
			downlink_data_notification_delay,
			dl_buffering_duration,
			dl_buffering_suggested_packet_count,
			suggested_buffering_packets_count,
		})
	}
}


pub struct PFCPSessionReportResponse {
	pub cause: Cause,
	pub offending_ie: Option<OffendingIE>,
	pub update_bar: Option<UpdateBAR_SessionReportResponse>,
	pub pfcpsr_rsp_flags: Option<PFCPSRRspFlags>,
	pub cp_f_seid: Option<F_SEID>,
	pub n4u_f_teid: Option<F_TEID>,
	// TODO: Alternative SMF IP Address
	// TODO: PGW-C/SMF FQ-CSID
	// TODO: Group Id
}
impl PFCPModel for PFCPSessionReportResponse {
	const ID: u16 = 57;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.cause.encode());
		self.offending_ie.as_ref().map(|o| result.append(&mut o.encode()));
		self.update_bar.as_ref().map(|o| result.append(&mut o.encode()));
		self.pfcpsr_rsp_flags.as_ref().map(|o| result.append(&mut o.encode()));
		self.cp_f_seid.as_ref().map(|o| result.append(&mut o.encode()));
		self.n4u_f_teid.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut cause = None;
		let mut offending_ie = None;
		let mut update_bar = None;
		let mut pfcpsr_rsp_flags = None;
		let mut cp_f_seid = None;
		let mut n4u_f_teid = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }

				OffendingIE::ID => { offending_ie = Some(OffendingIE::decode(curmsg)?); }
				UpdateBAR_SessionReportResponse::ID => { update_bar = Some(UpdateBAR_SessionReportResponse::decode(curmsg)?); }
				PFCPSRRspFlags::ID => { pfcpsr_rsp_flags = Some(PFCPSRRspFlags::decode(curmsg)?); }
				F_SEID::ID => { cp_f_seid = Some(F_SEID::decode(curmsg)?); }
				F_TEID::ID => { n4u_f_teid = Some(F_TEID::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			offending_ie,
			update_bar,
			pfcpsr_rsp_flags,
			cp_f_seid,
			n4u_f_teid,
		})
	}
}


pub struct NodeReportRequest {
	pub node_id: NodeID,
	pub node_report_type: NodeReportType,
	// ...
}
impl PFCPModel for NodeReportRequest {
	const ID: u16 = 12;

	fn encode(&self) -> Vec<u8> {
		let mut ret = vec![];
		ret.append(&mut self.node_id.encode());
		ret.append(&mut self.node_report_type.encode());
		ret
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut node_report_type = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				NodeReportType::ID => { node_report_type = Some(NodeReportType::decode(curmsg)?); }

				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			node_report_type: node_report_type.ok_or(PFCPError::new("Missing mandatory field NodeReportType"))?,

		})
	}
}


pub struct NodeReportResponse {
	pub node_id: NodeID,
	pub cause: Cause,
	pub offending_ie: Option<OffendingIE>
}

impl PFCPModel for NodeReportResponse {
	const ID: u16 = 13;

	fn encode(&self) -> Vec<u8> {
		let mut result = vec![];
		result.append(&mut self.node_id.encode());
		result.append(&mut self.cause.encode());
		self.offending_ie.as_ref().map(|o| result.append(&mut o.encode()));
		result
	}

	fn decode(stream: &[u8]) -> Result<Self, PFCPError> where Self: Sized {
		let mut stream = stream;
		let mut node_id = None;
		let mut cause = None;
		let mut offending_ie = None;
		while stream.len() > 4 {
			let msgtype = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			let length = u16::from_be_bytes(stream[0..2].try_into().unwrap()); stream = &stream[2..];
			if stream.len() < length as usize {
				return Err(PFCPError::new(&format!("Message is of length {}, but remaining octects is {}", length, stream.len())));
			}
			let curmsg = &stream[..length as usize];
			match msgtype {
				NodeID::ID => { node_id = Some(NodeID::decode(curmsg)?); }
				Cause::ID => { cause = Some(Cause::decode(curmsg)?); }

				OffendingIE::ID => { offending_ie = Some(OffendingIE::decode(curmsg)?); }
				_ => { println!("Ignore unknown message type {}", msgtype); }
			}
			stream = &stream[length as usize..];
		}
		Ok(Self {
			node_id: node_id.ok_or(PFCPError::new("Missing mandatory field NodeID"))?,
			cause: cause.ok_or(PFCPError::new("Missing mandatory field Cause"))?,
			
			offending_ie: offending_ie,
		})
	}
}
