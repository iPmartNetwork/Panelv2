use crate::error::{AppError, ConfigurationError};
use netdev::Interface;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_wireguard_interface")]
    pub wireguard_interface: String,
    #[serde(default = "default_network_interface")]
    pub network_interface: String,
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_wireguard_config_path")]
    pub wireguard_config_path: String,
}

impl AppConfig {
    pub fn get_network_interface_name(&self) -> Result<String, AppError> {
        if !self.network_interface.is_empty() {
            return Ok(self.network_interface.to_owned());
        }
        netdev::get_default_interface()
            .map_err(AppError::CouldNotGetDefaultInterface)
            .map(|interface| interface.name.to_owned())
    }

    pub fn get_wireguard_network_interface(&self) -> Result<Interface, ConfigurationError> {
        let mut interfaces = netdev::get_interfaces()
            .into_iter()
            .map(|interface| (interface.to_owned(), interface.name));
        if let Some((interface, _)) =
            &interfaces.find(|(_, name)| name == &self.wireguard_interface)
        {
            return Ok(interface.to_owned().to_owned());
        };
        Err(ConfigurationError::WireGuardInterfaceNotFound {
            interface: self.wireguard_interface.to_owned(),
            available_interfaces: interfaces.map(|(_, name)| name.to_owned()).collect(),
        })
    }
}

fn default_wireguard_interface() -> String {
    if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        "wg0".into()
    } else {
        "utun3".into()
    }
}

fn default_network_interface() -> String {
    "".to_string()
}

fn default_address() -> String {
    "0.0.0.0:6252".to_string()
}

fn default_wireguard_config_path() -> String {
    "/etc/wireguard/wg0.conf".to_string()
}
