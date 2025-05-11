use std::io;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use defguard_wireguard_rs::key::Key;
use defguard_wireguard_rs::WireguardInterfaceApi;

use crate::data::wireguard_peer::WireGuardPeer;
use crate::error::AppError;
use crate::WireGuardAppValues;

pub fn get_peers(
    app_values: Arc<Mutex<WireGuardAppValues>>,
) -> Result<Vec<WireGuardPeer>, AppError> {
    let app_values = app_values.lock().unwrap();
    let raw_peers = &app_values.wg_api.read_interface_data()?.peers;
    let mut peers = Vec::<WireGuardPeer>::new();

    for client in &app_values.wireguard_data.clients {
        let key = match Key::from_str(&client.public_key) {
            Ok(key) => key,
            Err(error) => {
                return Err(AppError::InvalidPublicKey {
                    public_key: client.public_key.clone(),
                    client: client.name.clone(),
                    error,
                });
            }
        };
        let raw_peer_option = &raw_peers.get(&key);
        if let Some(raw_peer) = raw_peer_option {
            peers.push(WireGuardPeer {
                name: client.name.clone(),
                uuid: client.uuid,
                server_allowed_ips: raw_peer.allowed_ips.clone(),
                address: client.address.clone(),
                protocol_version: raw_peer.protocol_version,
                endpoint: raw_peer.endpoint,
                dns: client.dns.clone(),
                transmitted_bytes: raw_peer.tx_bytes,
                received_bytes: raw_peer.rx_bytes,
                last_handshake: raw_peer.last_handshake,
            })
        }
    }

    Ok(peers)
}

pub fn restart_wireguard(interface: &String) -> Result<(), RestartWireGuardErrorType> {
    if let Err(error) = stop_wireguard(interface) {
        return Err(RestartWireGuardErrorType::StopFailed(error));
    };
    match start_wireguard(interface) {
        Err(error) => Err(RestartWireGuardErrorType::StartFailed(error)),
        _ => Ok(()),
    }
}

pub fn reload_wireguard(interface: &String) -> Result<(), io::Error> {
    match Command::new("sudo")
        .arg("systemctl")
        .arg("reload")
        .arg(format!("wg-quick@{}", interface))
        .status()
    {
        Err(error) => Err(error),
        Ok(_) => Ok(())
    }
}

pub fn start_wireguard(interface: &String) -> Result<(), io::Error> {
    match Command::new("wg-quick").arg("up").arg(interface).output() {
        Err(error) => Err(error),
        _ => Ok(()),
    }
}

pub fn stop_wireguard(interface: &String) -> Result<(), io::Error> {
    match Command::new("wg-quick").arg("down").arg(interface).output() {
        Err(error) => Err(error),
        _ => Ok(()),
    }
}

pub enum RestartWireGuardErrorType {
    StopFailed(io::Error),
    StartFailed(io::Error),
}
