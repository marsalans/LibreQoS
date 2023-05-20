use crate::web::wss::queries::{
    omnisearch, root_heat_map, send_circuit_info, send_packets_for_all_nodes,
    send_packets_for_node, send_perf_for_node, send_rtt_for_all_nodes, send_rtt_for_all_nodes_site,
    send_rtt_for_node, send_site_info, send_site_parents, send_throughput_for_all_nodes,
    send_throughput_for_all_nodes_by_site, send_throughput_for_node, site_heat_map,
    site_tree::send_site_tree, send_throughput_for_all_nodes_by_circuit, send_rtt_for_all_nodes_circuit, send_site_stack_map, time_period::InfluxTimePeriod,
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::IntoResponse,
};
use pgdb::sqlx::{Pool, Postgres};
use wasm_pipe_types::{WasmRequest, WasmResponse};
mod login;
mod nodes;
mod queries;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Pool<Postgres>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |sock| handle_socket(sock, state))
}

async fn handle_socket(mut socket: WebSocket, cnn: Pool<Postgres>) {
    log::info!("WebSocket Connected");
    let mut credentials: Option<login::LoginResult> = None;
    while let Some(msg) = socket.recv().await {
        let cnn = cnn.clone();
        let msg = msg.unwrap();

        // Get the binary message and decompress it
        log::info!("Received a message: {:?}", msg);
        let raw = msg.into_data();
        let uncompressed = miniz_oxide::inflate::decompress_to_vec(&raw).unwrap();
        let msg = lts_client::cbor::from_slice::<WasmRequest>(&uncompressed).unwrap();
        log::info!("{msg:?}");

        // Update the token credentials (if there are any)
        if let Some(credentials) = &credentials {
            let _ = pgdb::refresh_token(cnn.clone(), &credentials.token).await;
        }

        // Handle the message by type
        match msg {
            // Handle login with just a token
            WasmRequest::Auth { token } => {
                let result = login::on_token_auth(&token, &mut socket, cnn).await;
                if let Some(result) = result {
                    credentials = Some(result);
                }
            }
            // Handle login with a username and password
            WasmRequest::Login { license, username, password } => {
                let result = login::on_login(&license, &username, &password, &mut socket, cnn).await;
                if let Some(result) = result {
                    credentials = Some(result);
                }
            }
            // Node status for dashboard
            WasmRequest::GetNodeStatus => {
                if let Some(credentials) = &credentials {
                    nodes::node_status(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                    )
                    .await;
                } else {
                    log::info!("Node status requested but no credentials provided");
                }
            }
            // Packet chart for dashboard
            WasmRequest::PacketChart { period } => {
                if let Some(credentials) = &credentials {
                    let _ = send_packets_for_all_nodes(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            // Packet chart for individual node
            WasmRequest::PacketChartSingle { period, node_id, node_name } => {
                if let Some(credentials) = &credentials {
                    let _ = send_packets_for_node(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                        node_id,
                        node_name,
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            // Throughput chart for the dashboard
            WasmRequest::ThroughputChart { period } => {
                if let Some(credentials) = &credentials {
                    let _ = send_throughput_for_all_nodes(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            // Throughput chart for a single shaper node
            WasmRequest::ThroughputChartSingle { period, node_id, node_name } => {
                if let Some(credentials) = &credentials {
                    let _ = send_throughput_for_node(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                        node_id,
                        node_name,
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            },
            WasmRequest::ThroughputChartSite { period, site_id } => {
                if let Some(credentials) = &credentials {
                    let _ = send_throughput_for_all_nodes_by_site(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        site_id,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            WasmRequest::ThroughputChartCircuit { period, circuit_id } => {
                if let Some(credentials) = &credentials {
                    let _ = send_throughput_for_all_nodes_by_circuit(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        circuit_id,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            // Rtt Chart
            WasmRequest::RttChart { period } => {
                if let Some(credentials) = &credentials {
                    let _ = send_rtt_for_all_nodes(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            WasmRequest::RttChartSite { period, site_id } => {
                if let Some(credentials) = &credentials {
                    let _ = send_rtt_for_all_nodes_site(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        site_id,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            WasmRequest::RttChartSingle { period, node_id, node_name } => {
                if let Some(credentials) = &credentials {
                    let _ = send_rtt_for_node(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                        node_id,
                        node_name,
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            WasmRequest::RttChartCircuit { period, circuit_id } => {
                if let Some(credentials) = &credentials {
                    let _ = send_rtt_for_all_nodes_circuit(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        circuit_id,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            // Site Stack
            WasmRequest::SiteStack { period, site_id } => {
                if let Some(credentials) = &credentials {
                    let _ = send_site_stack_map(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                        site_id,
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            },
            WasmRequest::RootHeat { period } => {
                if let Some(credentials) = &credentials {
                    let _ = root_heat_map(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            },
            WasmRequest::SiteHeat { period, site_id } => {
                if let Some(credentials) = &credentials {
                    let _ = site_heat_map(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &site_id,
                        InfluxTimePeriod::new(&period),
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            }
            WasmRequest::NodePerfChart { period, node_id, node_name } => {
                if let Some(credentials) = &credentials {
                    let _ = send_perf_for_node(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        InfluxTimePeriod::new(&period),
                        node_id,
                        node_name,
                    )
                    .await;
                } else {
                    log::info!("Throughput requested but no credentials provided");
                }
            },
            WasmRequest::Tree { parent } => {
                if let Some(credentials) = &credentials {
                    send_site_tree(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &parent,
                    )
                    .await;
                }
            },
            WasmRequest::SiteInfo { site_id } => {
                if let Some(credentials) = &credentials {
                    send_site_info(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &site_id,
                    )
                    .await;
                }
            }
            WasmRequest::SiteParents { site_id } => {
                if let Some(credentials) = &credentials {
                    send_site_parents(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &site_id,
                    )
                    .await;
                }
            }
            WasmRequest::Search { term } => {
                if let Some(credentials) = &credentials {
                    let _ = omnisearch(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &term,
                    )
                    .await;
                }
            }
            WasmRequest::CircuitInfo { circuit_id } => {
                if let Some(credentials) = &credentials {
                    send_circuit_info(
                        cnn.clone(),
                        &mut socket,
                        &credentials.license_key,
                        &circuit_id,
                    )
                    .await;
                }
            }
        }

        /*if let Ok(text) = msg.into_text() {
            let json = serde_json::from_str::<Value>(&text);
            if json.is_err() {
                log::warn!("Unable to parse JSON: {}", json.err().unwrap());
            } else if let Ok(json) = json {
                log::info!("Received a JSON: {:?}", json);

                if let Some(credentials) = &credentials {
                    let _ = pgdb::refresh_token(cnn.clone(), &credentials.token).await;
                }

                let period =
                    queries::time_period::InfluxTimePeriod::new(json.get("period").cloned());

                if let Some(Value::String(msg_type)) = json.get("msg") {
                    match msg_type.as_str() {
                        "login" => {
                            // A full login request
                            let result = login::on_login(&json, &mut socket, cnn).await;
                            if let Some(result) = result {
                                credentials = Some(result);
                            }
                        }
                        "auth" => {
                            // Login with just a token
                            let result = login::on_token_auth(&json, &mut socket, cnn).await;
                            if let Some(result) = result {
                                credentials = Some(result);
                            }
                        }
                        "nodeStatus" => {
                            if let Some(credentials) = &credentials {
                                nodes::node_status(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                )
                                .await;
                            } else {
                                log::info!("Node status requested but no credentials provided");
                            }
                        }
                        "packetChart" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_packets_for_all_nodes(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "packetChartSingle" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_packets_for_node(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                    json.get("node_id").unwrap().as_str().unwrap().to_string(),
                                    json.get("node_name").unwrap().as_str().unwrap().to_string(),
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "throughputChart" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_throughput_for_all_nodes(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "throughputChartSite" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_throughput_for_all_nodes_by_site(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("site_id").unwrap().as_str().unwrap().to_string(),
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "throughputStackSite" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_site_stack_map(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                    json.get("site_id").unwrap().as_str().unwrap().to_string(),
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "throughputChartSingle" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_throughput_for_node(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                    json.get("node_id").unwrap().as_str().unwrap().to_string(),
                                    json.get("node_name").unwrap().as_str().unwrap().to_string(),
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "throughputChartCircuit" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_throughput_for_all_nodes_by_circuit(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("circuit_id").unwrap().as_str().unwrap().to_string(),
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "rttChart" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_rtt_for_all_nodes(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "rttChartSite" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_rtt_for_all_nodes_site(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("site_id").unwrap().as_str().unwrap().to_string(),
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "rttChartCircuit" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_rtt_for_all_nodes_circuit(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("circuit_id").unwrap().as_str().unwrap().to_string(),
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "rttChartSingle" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_rtt_for_node(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                    json.get("node_id").unwrap().as_str().unwrap().to_string(),
                                    json.get("node_name").unwrap().as_str().unwrap().to_string(),
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "nodePerf" => {
                            if let Some(credentials) = &credentials {
                                let _ = send_perf_for_node(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                    json.get("node_id").unwrap().as_str().unwrap().to_string(),
                                    json.get("node_name").unwrap().as_str().unwrap().to_string(),
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "search" => {
                            if let Some(credentials) = &credentials {
                                let _ = omnisearch(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("term").unwrap().as_str().unwrap(),
                                )
                                .await;
                            }
                        }
                        "siteRootHeat" => {
                            if let Some(credentials) = &credentials {
                                let _ = root_heat_map(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "siteHeat" => {
                            if let Some(credentials) = &credentials {
                                let _ = site_heat_map(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("site_id").unwrap().as_str().unwrap(),
                                    period,
                                )
                                .await;
                            } else {
                                log::info!("Throughput requested but no credentials provided");
                            }
                        }
                        "siteTree" => {
                            if let Some(credentials) = &credentials {
                                send_site_tree(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("parent").unwrap().as_str().unwrap(),
                                )
                                .await;
                            }
                        }
                        "siteInfo" => {
                            if let Some(credentials) = &credentials {
                                send_site_info(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("site_id").unwrap().as_str().unwrap(),
                                )
                                .await;
                            }
                        }
                        "circuitInfo" => {
                            if let Some(credentials) = &credentials {
                                send_circuit_info(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("circuit_id").unwrap().as_str().unwrap(),
                                )
                                .await;
                            }
                        }
                        "siteParents" => {
                            if let Some(credentials) = &credentials {
                                send_site_parents(
                                    cnn.clone(),
                                    &mut socket,
                                    &credentials.license_key,
                                    json.get("site_id").unwrap().as_str().unwrap(),
                                )
                                .await;
                            }
                        }
                        _ => {
                            log::warn!("Unknown message type: {msg_type}");
                        }
                    }
                }
            }
        }*/
    }
}

fn serialize_resposne(response: WasmResponse) -> Vec<u8> {
    let cbor = lts_client::cbor::to_vec(&response).unwrap();
    miniz_oxide::deflate::compress_to_vec(&cbor, 8)
}

pub async fn send_response(socket: &mut WebSocket, response: WasmResponse) {
    let serialized = serialize_resposne(response);
    socket.send(Message::Binary(serialized)).await.unwrap();
}