use defguard_wireguard_rs::error::WireguardInterfaceError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    #[error("{0}")]
    RestAPI(#[from] RestAPIError),
    #[error("JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("WireGuard interface error: {0}")]
    WireGuardInterface(#[from] WireguardInterfaceError),
    #[error("Invalid public key ('{public_key}') for client '{client}': {error}")]
    InvalidPublicKey {
        public_key: String,
        client: String,
        error: <defguard_wireguard_rs::key::Key as std::str::FromStr>::Err,
    },
    #[error("Could not get the default network interface: {0}")]
    CouldNotGetDefaultInterface(String),
    #[error("Invalid server address: {0}")]
    InvalidServerAddress(String),
}

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("WireGuard interface '{interface:?}' not found, available interfaces: {available_interfaces:?}")]
    WireGuardInterfaceNotFound {
        interface: String,
        available_interfaces: Vec<String>,
    },
}

#[derive(Error, Debug)]
pub enum RestAPIError {
    #[error("Field '{0}' missing from request body")]
    FieldMissing(String),
    #[error("Invalid base64 private key: '{0}'")]
    InvalidPrivateKey(String),
}
