use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use defguard_wireguard_rs::net::IpAddrMask;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardPeer {
    pub name: String,
    #[serde(serialize_with = "uuid::serde::simple::serialize")]
    pub uuid: Uuid,
    #[serde(serialize_with = "serialize_ip_addr_mask_vec")]
    pub server_allowed_ips: Vec<IpAddrMask>,
    pub address: String,
    pub protocol_version: Option<u32>,
    pub endpoint: Option<SocketAddr>,
    pub dns: Vec<String>,
    pub transmitted_bytes: u64,
    pub received_bytes: u64,
    #[serde(serialize_with = "serialize_system_time_option")]
    pub last_handshake: Option<SystemTime>,
}

fn serialize_ip_addr_mask_vec<S>(vec: &[IpAddrMask], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(
        &vec.iter().map(ToString::to_string).collect::<Vec<String>>(),
        serializer,
    )
}

fn serialize_system_time_option<S>(
    time: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(
        &time.map(|x| x.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64),
        serializer,
    )
}
