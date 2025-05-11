use std::net::Ipv4Addr;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wireguard_keys::{Privkey, Secret};

use crate::error::{AppError, RestAPIError};
use crate::WireGuardAppValues;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardClientData {
    pub name: String,
    #[serde(serialize_with = "uuid::serde::simple::serialize")]
    pub uuid: Uuid,
    pub enabled: bool,
    // stored in server & client configs
    pub preshared_key: Option<String>,
    // stored in server config
    pub public_key: String,
    pub server_allowed_ips: Vec<String>,
    pub persistent_keep_alive: Option<u16>,
    // stored in client config
    pub private_key: String,
    pub address: String,
    pub client_allowed_ips: Vec<String>,
    pub dns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardOptionalClientData {
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
    pub enabled: Option<bool>,
    pub generate_preshared_key: Option<bool>,
    pub preshared_key: Option<String>,
    pub server_allowed_ips: Option<Vec<String>>,
    pub persistent_keep_alive: Option<u16>,
    pub private_key: Option<String>,
    pub address: Option<String>,
    pub client_allowed_ips: Option<Vec<String>>,
    pub dns: Option<Vec<String>>,
}

impl WireGuardOptionalClientData {
    pub fn to_wireguard_client_data(
        &self,
        default_name: Option<String>,
        app_values: &WireGuardAppValues,
    ) -> Result<WireGuardClientData, AppError> {
        let config = &app_values.config;
        let data = &app_values.wireguard_data;
        let server = &data.server;
        let private_key = self
            .private_key
            .to_owned()
            .unwrap_or_else(|| Privkey::generate().to_base64());

        let mut beginning_address = match server.clone() {
            Some(server) => {
                let mut addr = server.address[0].clone();
                addr = addr
                    .rsplit_once('/')
                    .map(|(ip, _)| ip.to_string())
                    .unwrap_or(addr);
                match Ipv4Addr::from_str(addr.as_str()) {
                    Ok(ip) => ip,
                    Err(_) => {
                        return Err(AppError::InvalidServerAddress(server.address[0].clone()));
                    }
                }
            }
            None => config
                .clone()
                .get_wireguard_network_interface()
                .map(|i| i.ipv4[0].addr)?,
        };
        // increment
        let mut octets = beginning_address.octets();
        if octets[3] < 255 {
            octets[3] += 1;
        } else {
            octets[2] += 1;
            octets[3] = 0;
        }
        beginning_address = Ipv4Addr::from(octets);
        let ip = format!("{beginning_address}/32");

        Ok(WireGuardClientData {
            name: match self.name.to_owned().or(default_name) {
                Some(name) => name,
                None => {
                    return Err(AppError::RestAPI(RestAPIError::FieldMissing(
                        "name".to_string(),
                    )))
                }
            },
            uuid: self.uuid.unwrap_or_else(Uuid::new_v4),
            enabled: self.enabled.unwrap_or(false),
            preshared_key: self.preshared_key.to_owned().or_else(|| {
                match self.generate_preshared_key.unwrap_or(true) {
                    true => Some(Secret::generate().to_base64()),
                    false => None,
                }
            }),
            public_key: Privkey::parse(private_key.as_str())
                .map_err(|_| {
                    AppError::RestAPI(RestAPIError::InvalidPrivateKey(private_key.to_owned()))
                })?
                .pubkey()
                .to_base64(),
            server_allowed_ips: self
                .server_allowed_ips
                .to_owned()
                .unwrap_or_else(|| vec![ip.clone()]),
            persistent_keep_alive: self.persistent_keep_alive,
            private_key,
            address: self.address.to_owned().unwrap_or(ip),
            client_allowed_ips: self
                .client_allowed_ips
                .to_owned()
                .unwrap_or_else(|| vec!["0.0.0.0".to_string()]),
            dns: self.dns.to_owned().unwrap_or_default(),
        })
    }
}

impl WireGuardClientData {
    pub fn get_server_peer_config(&self) -> String {
        let mut result = format!("# Name: {}", self.name);
        result += &format!("\n# UUID: {}", self.uuid);
        let prefix = if self.enabled { "\n" } else { "\n# " };
        result += &format!("{}[Peer]", prefix);
        result += &format!("{}PublicKey = {}", prefix, self.public_key);
        if let Some(preshared_key) = &self.preshared_key {
            result += &format!("{}PresharedKey = {preshared_key}", prefix);
        }
        result += &format!(
            "{}AllowedIPs = {}",
            prefix,
            self.server_allowed_ips.join(",")
        );
        if let Some(persistent_keep_alive) = self.persistent_keep_alive {
            result += &format!("{}PersistentKeepAlive = {persistent_keep_alive}", prefix);
        }
        result
    }

    pub fn get_client_config(
        &self,
        server_public_key: &String,
        server_endpoint: &String,
    ) -> String {
        let mut result = format!("# Name: {}", self.name);
        result += "[Interface]";
        result += &format!("PrivateKey = {}", self.private_key);
        result += &format!("Address = {}", self.address);
        if !self.dns.is_empty() {
            result += &format!("DNS = {}", self.dns.join(","));
        }
        result += "[Peer]";
        result += &format!("PublicKey = {server_public_key}");
        if let Some(preshared_key) = &self.preshared_key {
            result += &format!("PresharedKey = {preshared_key}");
        }
        result += &format!("AllowedIPs = {}", self.client_allowed_ips.join(","));
        result += &format!("Endpoint = {server_endpoint}");
        result
    }
}
