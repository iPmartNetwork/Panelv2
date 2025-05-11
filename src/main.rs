#![cfg(target_os = "linux")]
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

use defguard_wireguard_rs::WGApi;
use nix::unistd::Uid;

use crate::data::config::AppConfig;
use crate::data::wireguard_data::WireGuardData;

mod data;
mod error;
mod server;
mod wireguard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !Uid::effective().is_root() {
        panic!("This must be run as root!");
    }

    println!("Reading config file");
    let config = data::data_manager::read_config_file()?;
    data::data_manager::save_config_file(&config)?;

    println!("Reading data file");
    let data = data::data_manager::read_json_file()?;
    data::data_manager::save_json_file(&data)?;

    println!("Preparing WireGuard");
    let wg_api = WGApi::new(config.wireguard_interface.to_owned(), false)?;

    let app_values = Arc::new(Mutex::new(WireGuardAppValues {
        wg_api,
        config,
        wireguard_data: data,
    }));

    println!("Starting server");
    server::start_server(app_values.clone()).await;

    // add something else later?

    loop {
        thread::park();
    }
}

pub struct WireGuardAppValues {
    pub wg_api: WGApi,
    pub config: AppConfig,
    pub wireguard_data: WireGuardData,
}
