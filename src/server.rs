use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::data::data_manager;
use crate::data::wireguard_client::{WireGuardClientData, WireGuardOptionalClientData};
use crate::data::wireguard_data::WireGuardOptionalData;
use crate::data::wireguard_server::WireGuardOptionalServerData;
use crate::error::AppError;
use crate::wireguard::RestartWireGuardErrorType;
use crate::{wireguard, WireGuardAppValues};

pub async fn start_server(app_values: Arc<Mutex<WireGuardAppValues>>) {
    let address = SocketAddr::from_str(app_values.lock().unwrap().config.address.as_str())
        .expect("Could not parse address");
    tokio::spawn(async move {
        let listener = match TcpListener::bind(address).await {
            Ok(listener) => listener,
            #[allow(unused_variables)] // bugged
            Err(error) => {
                panic!("Could not bind to address {address}: {error}");
            }
        };
        let server = axum::serve(
            listener,
            Router::new()
                .route(
                    "/wireguard/server",
                    axum::routing::get(get_wireguard_server),
                )
                .route(
                    "/wireguard/server",
                    axum::routing::put(put_wireguard_server),
                )
                .route(
                    "/wireguard/server",
                    axum::routing::delete(delete_wireguard_server),
                )
                .route(
                    "/wireguard/clients",
                    axum::routing::get(get_wireguard_clients),
                )
                .route(
                    "/wireguard/clients",
                    axum::routing::put(put_wireguard_clients),
                )
                .route(
                    "/wireguard/clients",
                    axum::routing::post(post_wireguard_clients),
                )
                .route(
                    "/wireguard/clients/:uuid",
                    axum::routing::get(get_wireguard_client),
                )
                .route(
                    "/wireguard/clients/:uuid",
                    axum::routing::put(put_wireguard_client),
                )
                .route("/wireguard/peers", axum::routing::get(get_wireguard_peers))
                .route("/wireguard/restart", axum::routing::post(wireguard_restart)) // also saves into file
                .route("/wireguard/reload", axum::routing::post(wireguard_reload)) // also saves into file
                .route("/wireguard/start", axum::routing::post(wireguard_start))
                .route("/wireguard/stop", axum::routing::post(wireguard_stop))
                .route("/sample", axum::routing::get(sample))
                .with_state(app_values)
                .into_make_service_with_connect_info::<SocketAddr>(),
        );
        server.await.unwrap();
        panic!("Server stopped unexpectedly");
    });

    println!("Server started on {}", address);
}

async fn get_wireguard_server(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(app_values.lock().unwrap().wireguard_data.server.clone()),
    )
}

async fn put_wireguard_server(
    State(app_values_arc): State<Arc<Mutex<WireGuardAppValues>>>,
    Json(body): Json<Option<WireGuardOptionalServerData>>,
) -> Response<Body> {
    let mut app_values = app_values_arc.lock().unwrap();
    let server = match body {
        Some(server) => match server.to_wireguard_server_data(
            app_values
                .wireguard_data
                .server
                .clone()
                .map(|server| server.endpoint),
            &app_values,
        ) {
            Ok(server) => Some(server),
            Err(error) => {
                return ErrorResponse::from((
                    if let AppError::RestAPI(_) = error {
                        StatusCode::BAD_REQUEST
                    } else {
                        StatusCode::INTERNAL_SERVER_ERROR
                    },
                    format!("Could not create server: {error}"),
                ))
                .into();
            }
        },
        None => None,
    };
    app_values.wireguard_data.server.clone_from(&server);
    match data_manager::save_json_file(&app_values.wireguard_data) {
        Ok(_) => (StatusCode::OK, Json(server)).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save data: {error}"),
        ))
        .into(),
    }
}

async fn delete_wireguard_server(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> impl IntoResponse {
    let mut app_values = app_values.lock().unwrap();
    app_values.wireguard_data.server = None;
    match data_manager::save_json_file(&app_values.wireguard_data) {
        Ok(_) => (StatusCode::OK, String::new()).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save data: {error}"),
        ))
        .into(),
    }
}

async fn get_wireguard_clients(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(app_values.lock().unwrap().wireguard_data.clients.clone()),
    )
}

async fn put_wireguard_clients(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
    Json(body): Json<Vec<WireGuardClientData>>,
) -> impl IntoResponse {
    let mut app_values = app_values.lock().unwrap();
    app_values.wireguard_data.clients = body;
    match data_manager::save_json_file(&app_values.wireguard_data) {
        Ok(_) => (StatusCode::OK, String::new()).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save data: {error}"),
        ))
        .into(),
    }
}

async fn get_wireguard_client(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
    Path(uuid): Path<Uuid>,
) -> Response<Body> {
    let app_values = app_values.lock().unwrap();
    match app_values.wireguard_data.get_client_config(&uuid) {
        Some(client) => (StatusCode::OK, Json(client)).into_response(),
        None => ErrorResponse::from((
            StatusCode::NOT_FOUND,
            format!("Client config for uuid {} not found", uuid),
        ))
        .into(),
    }
}

async fn put_wireguard_client(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
    Path(uuid): Path<Uuid>,
    Json(body): Json<WireGuardClientData>,
) -> Response<Body> {
    let mut app_values = app_values.lock().unwrap();
    let client_index = app_values
        .wireguard_data
        .clients
        .iter()
        .position(|client| client.uuid == uuid);
    match client_index {
        Some(index) => app_values.wireguard_data.clients[index] = body,
        None => {
            return ErrorResponse::from((
                StatusCode::NOT_FOUND,
                format!("Client config for uuid {} not found", uuid),
            ))
            .into()
        }
    }

    match data_manager::save_json_file(&app_values.wireguard_data) {
        Ok(_) => (StatusCode::OK, String::new()).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save data: {error}"),
        ))
        .into(),
    }
}

async fn post_wireguard_clients(
    State(app_values_arc): State<Arc<Mutex<WireGuardAppValues>>>,
    Json(body): Json<WireGuardOptionalClientData>,
) -> Response<Body> {
    let mut app_values = app_values_arc.lock().unwrap();
    let new_client = match body.to_wireguard_client_data(None, &app_values) {
        Ok(client) => client,
        Err(error) => {
            return ErrorResponse::from((
                if let AppError::RestAPI(_) = error {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                },
                format!("Could not create client: {error}"),
            ))
            .into();
        }
    };
    if app_values
        .wireguard_data
        .clients
        .iter()
        .any(|client| client.uuid == new_client.uuid)
    {
        return ErrorResponse::from((
            StatusCode::CONFLICT,
            format!("Client with uuid {} already exists", new_client.uuid),
        ))
        .into();
    }
    app_values.wireguard_data.clients.push(new_client.clone());

    match data_manager::save_json_file(&app_values.wireguard_data) {
        Ok(_) => (StatusCode::OK, Json(new_client)).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save data: {error}"),
        ))
        .into(),
    }
}

async fn get_wireguard_peers(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> Response<Body> {
    match wireguard::get_peers(app_values.clone()) {
        Ok(peers) => (StatusCode::OK, Json(peers)).into_response(),
        Err(error) => ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not get peers: {error}"),
        ))
        .into(),
    }
}

async fn wireguard_restart(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> Response<Body> {
    let app_values = app_values.lock().unwrap();
    if let Err(error) =
        data_manager::save_wireguard_config(&app_values.wireguard_data, &app_values.config)
    {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save config: {error}"),
        ))
        .into();
    };
    if let Err(error) = wireguard::restart_wireguard(&app_values.config.wireguard_interface) {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            match error {
                RestartWireGuardErrorType::StopFailed(err) => {
                    format!("{}: {}", "Could not stop WireGuard", err)
                }
                RestartWireGuardErrorType::StartFailed(err) => {
                    format!("{}: {}", "Could not start WireGuard", err)
                }
            },
        ))
        .into();
    }
    (StatusCode::OK, String::new()).into_response()
}

async fn wireguard_reload(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> Response<Body> {
    let app_values = app_values.lock().unwrap();
    if let Err(error) =
        data_manager::save_wireguard_config(&app_values.wireguard_data, &app_values.config)
    {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not save config: {error}"),
        ))
        .into();
    };
    if let Err(error) = wireguard::reload_wireguard(&app_values.config.wireguard_interface) {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", "Could not reload WireGuard", error),
        ))
        .into();
    };
    (StatusCode::OK, String::new()).into_response()
}

async fn wireguard_start(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> Response<Body> {
    let app_values = app_values.lock().unwrap();
    if let Err(error) = wireguard::start_wireguard(&app_values.config.wireguard_interface) {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not start WireGuard: {error}"),
        ))
        .into();
    }
    (StatusCode::OK, String::new()).into_response()
}

async fn wireguard_stop(
    State(app_values): State<Arc<Mutex<WireGuardAppValues>>>,
) -> Response<Body> {
    let app_values = app_values.lock().unwrap();
    if let Err(error) = wireguard::stop_wireguard(&app_values.config.wireguard_interface) {
        return ErrorResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not stop WireGuard: {error}"),
        ))
        .into();
    }
    (StatusCode::OK, String::new()).into_response()
}

async fn sample() -> impl IntoResponse {
    (
        StatusCode::OK,
        serde_json::to_string(
            &WireGuardOptionalData {
                server: Some(WireGuardOptionalServerData {
                    endpoint: Some("endpoint.com:51820".into()),
                    address: Some(vec!["10.8.0.1/24".into()]),
                    dns: Some(vec!["1.1.1.1".into()]),
                    listen_port: Some(51820),
                    private_key: Some("oL5cNL2cZQVNLYEfg4LIEEfS6KaFN1YSmOlq5rRJjlI=".to_string()),
                    pre_up: None,
                    post_up: Some("iptables -A FORWARD -i {WIREGUARD_INTERFACE} -j ACCEPT; iptables -t nat -A POSTROUTING -o {NETWORK_INTERFACE} -j MASQUERADE".into()),
                    pre_down: None,
                    post_down: Some("iptables -D FORWARD -i {WIREGUARD_INTERFACE} -j ACCEPT; iptables -t nat -D POSTROUTING -o {NETWORK_INTERFACE} -j MASQUERADE".into()),
                    table: None,
                    mtu: None,
                }),
                clients: vec![
                    WireGuardOptionalClientData {
                        name: Some("Sample Client".into()),
                        uuid: Some(Uuid::new_v4()),
                        enabled: Some(true),
                        generate_preshared_key: Some(true),
                        preshared_key: Some("KS4xysNuixRcArtY/iNph8dQyhXv/W1rxc0QOiDlhzs=".into()),
                        server_allowed_ips: Some(vec!["10.8.0.2/32".into()]),
                        persistent_keep_alive: None,
                        private_key: Some("qD+418LUGssYC/V6ZHJQz2YQO8PCWv9gmX4QWtKEMHg=".to_string()),
                        address: Some("10.8.0.2/32".to_string()),
                        client_allowed_ips: Some(vec!["0.0.0.0/0".into()]),
                        dns: Some(vec![]),
                    }
                ],
            }
        ).unwrap()
    )
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    pub error: ErrorResponseData,
}

impl From<(u16, String)> for ErrorResponse {
    fn from(value: (u16, String)) -> Self {
        ErrorResponse {
            error: value.into(),
        }
    }
}

impl From<(StatusCode, String)> for ErrorResponse {
    fn from(value: (StatusCode, String)) -> Self {
        ErrorResponse {
            error: value.into(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.error.code).unwrap(), Json(self)).into_response()
    }
}

impl From<ErrorResponse> for Response<Body> {
    fn from(value: ErrorResponse) -> Self {
        value.into_response()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponseData {
    pub code: u16,
    pub message: String,
}

impl From<(u16, String)> for ErrorResponseData {
    fn from(value: (u16, String)) -> Self {
        ErrorResponseData {
            code: value.0,
            message: value.1,
        }
    }
}

impl From<(StatusCode, String)> for ErrorResponseData {
    fn from(value: (StatusCode, String)) -> Self {
        ErrorResponseData {
            code: value.0.as_u16(),
            message: value.1,
        }
    }
}
