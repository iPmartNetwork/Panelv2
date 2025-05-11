use crate::data::config::AppConfig;
use crate::data::wireguard_client::{WireGuardClientData, WireGuardOptionalClientData};
use crate::data::wireguard_server::{WireGuardOptionalServerData, WireGuardServerData};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WireGuardData {
    pub server: Option<WireGuardServerData>,
    #[serde(default = "Vec::new")]
    pub clients: Vec<WireGuardClientData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardOptionalData {
    pub server: Option<WireGuardOptionalServerData>,
    #[serde(default = "Vec::new")]
    pub clients: Vec<WireGuardOptionalClientData>,
}

impl WireGuardData {
    pub fn get_server_config(&self, app_config: &AppConfig) -> Option<String> {
        let server = match self.server {
            Some(ref server) => server,
            None => return None,
        };
        let mut result = String::new();
        result += &String::from("# Generated from WireGuard UI\n");
        result += &String::from("# Do not edit manually!\n\n");

        result += &server.get_interface_config(app_config);
        for client in &self.clients {
            result += &format!("\n\n{}", client.get_server_peer_config());
        }
        Some(result + "\n")
    }

    pub fn get_client_config(&self, uuid: &Uuid) -> Option<WireGuardClientData> {
        for client in &self.clients {
            if &client.uuid == uuid {
                return Some(client.clone());
            }
        }
        None
    }
}
