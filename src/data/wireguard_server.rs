use crate::data::config::AppConfig;
use crate::error::{AppError, RestAPIError};
use crate::WireGuardAppValues;
use serde::{Deserialize, Serialize};
use wireguard_keys::Privkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardServerData {
    pub endpoint: String,
    pub address: Vec<String>,
    pub dns: Vec<String>,
    pub listen_port: u16,
    pub private_key: String,
    pub public_key: String,
    pub pre_up: Option<String>,
    pub post_up: Option<String>,
    pub pre_down: Option<String>,
    pub post_down: Option<String>,
    pub table: Option<String>,
    pub mtu: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardOptionalServerData {
    pub endpoint: Option<String>,
    pub address: Option<Vec<String>>,
    pub dns: Option<Vec<String>>,
    pub listen_port: Option<u16>,
    pub private_key: Option<String>,
    pub pre_up: Option<String>,
    pub post_up: Option<String>,
    pub pre_down: Option<String>,
    pub post_down: Option<String>,
    pub table: Option<String>,
    pub mtu: Option<u16>,
}

impl WireGuardOptionalServerData {
    pub fn to_wireguard_server_data(
        &self,
        default_endpoint: Option<String>,
        app_values: &WireGuardAppValues,
    ) -> Result<WireGuardServerData, AppError> {
        let config = &app_values.config;

        let private_key = self
            .private_key
            .to_owned()
            .unwrap_or_else(|| Privkey::generate().to_base64());

        Ok(WireGuardServerData {
            endpoint: match self.endpoint.to_owned().or(default_endpoint) {
                Some(endpoint) => endpoint,
                None => return Err(AppError::RestAPI(RestAPIError::FieldMissing("endpoint".to_string()))),
            },
            address: match &self.address {
                Some(address) => address.to_owned(),
                None => vec![config.get_wireguard_network_interface()?.ipv4[0].addr.to_string()]
            },
            dns: self.dns.to_owned().unwrap_or_default(),
            listen_port: self.listen_port.unwrap_or(51820),
            public_key: Privkey::parse(private_key.as_str())
                .map_err(|_| AppError::RestAPI(RestAPIError::InvalidPrivateKey(private_key.to_owned())))?
                .pubkey()
                .to_base64(),
            private_key: private_key.to_owned(),
            pre_up: self.pre_up.to_owned(),
            post_up: self.post_up.to_owned().or(Some("iptables -A FORWARD -i {WIREGUARD_INTERFACE} -j ACCEPT; iptables -t nat -A POSTROUTING -o {NETWORK_INTERFACE} -j MASQUERADE".to_string())),
            pre_down: self.pre_down.to_owned(),
            post_down: self.post_down.to_owned().or(Some("iptables -D FORWARD -i {WIREGUARD_INTERFACE} -j ACCEPT; iptables -t nat -D POSTROUTING -o {NETWORK_INTERFACE} -j MASQUERADE".to_string())),
            table: self.table.to_owned(),
            mtu: self.mtu,
        })
    }
}

impl WireGuardServerData {
    pub fn get_interface_config(&self, app_config: &AppConfig) -> String {
        let mut result = String::from("[Interface]");
        result += &format!("\nAddress = {}", self.address.join(","));
        result += &format!("\nListenPort = {}", self.listen_port);
        result += &format!("\nPrivateKey = {}", self.private_key);
        if !self.dns.is_empty() {
            result += &format!("\nDNS = {}", self.dns.join(","));
        }
        let mut second_part = String::new();
        if let Some(table) = &self.table {
            second_part += &format!("\nTable = {}", table);
        }
        if let Some(mtu) = self.mtu {
            second_part += &format!("\nMTU = {}", mtu);
        }

        fn replace_interface_vars(str: &str, app_config: &AppConfig) -> String {
            str.replace(
                "{WIREGUARD_INTERFACE}",
                app_config.wireguard_interface.as_str(),
            )
            .replace("{NETWORK_INTERFACE}", app_config.network_interface.as_str())
        }
        if let Some(pre_up) = &self.pre_up {
            second_part += &format!("\nPreUp = {}", replace_interface_vars(pre_up, app_config));
        }
        if let Some(post_up) = &self.post_up {
            second_part += &format!("\nPostUp = {}", replace_interface_vars(post_up, app_config));
        }
        if let Some(pre_down) = &self.pre_down {
            second_part += &format!(
                "\nPreDown = {}",
                replace_interface_vars(pre_down, app_config)
            );
        }
        if let Some(post_down) = &self.post_down {
            second_part += &format!(
                "\nPostDown = {}",
                replace_interface_vars(post_down, app_config)
            );
        }
        if !second_part.is_empty() {
            result += &format!("\n{second_part}");
        }
        result
    }
}
