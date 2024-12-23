use clap::{Arg, Command};

mod controller;
mod pb;
mod service;
mod settings;

use crate::controller::VadServiceController;
use crate::pb::vad_grpc::vad_recognizer_server;
use crate::settings::settings::Settings;
pub(crate) use service::vad::VadService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::new("vad_grpc_server")
        .version("1.0")
        .about("Voice Activity Detection gRPC server")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file location")
                .default_value("config.yaml"),
        );

    let matches = command.get_matches();
    let config_location = matches
        .get_one::<String>("config")
        .unwrap_or(&"".to_string())
        .to_string();
    let settings = Settings::new(&config_location, "VAD_GRPC")?;

    println!("Settings:\n{}", settings.json_pretty());

    env_logger::init();

    // log::set_max_level(LevelFilter::from_str(settings.logging.log_level.as_str()).unwrap_or(LevelFilter::Info));

    let vad_service = VadServiceController::new(&settings)?;

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    println!("Server listening on {}", addr);

    let vad_server =
        vad_recognizer_server::VadRecognizerServer::new(vad_service).max_decoding_message_size(100 * 1024 * 1024);

    tonic::transport::Server::builder()
        .add_service(vad_server)
        .serve(addr.parse()?)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::controller::VadServiceController;
    use crate::pb::vad_grpc::vad_recognizer_client::VadRecognizerClient;
    use crate::pb::vad_grpc::{vad_recognizer_server, VadRequest};
    use crate::settings::settings::Settings;
    use std::sync::{Arc, LazyLock, OnceLock};

    static SETTINGS: LazyLock<Settings> = LazyLock::new(|| Settings {
        server: crate::settings::settings::Server {
            host: "0.0.0.0".to_string(),
            port: 9091,
        },
        logging: crate::settings::settings::Logging {
            log_level: "debug".to_string(),
        },
        vad: crate::settings::settings::VadSettings {
            model_path: "../silero_vad/model/silero_vad.onnx".to_string(),
            sessions_num: 1,
        },
    });

    static SERVER_HANDLE: OnceLock<()> = OnceLock::new();

    async fn start_server() {
        SERVER_HANDLE.get_or_init(|| {
            let vad_service = VadServiceController::new(&SETTINGS).expect("Failed to create VadService");

            let addr = format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port);
            println!("Server listening on {}", addr);

            tokio::spawn(async move {
                tonic::transport::Server::builder()
                    .add_service(
                        vad_recognizer_server::VadRecognizerServer::new(vad_service)
                            .max_decoding_message_size(100 * 1024 * 1024),
                    )
                    .serve(addr.parse().expect("Failed to parse address"))
                    .await
                    .expect("Failed to start server");
            });
            ()
        });
    }

    #[tokio::test]
    async fn test_vad() {
        start_server().await;

        let content = vec![32532i16; 16000 * 10];

        let addr = format!("http://localhost:{}", SETTINGS.server.port);
        let mut client = VadRecognizerClient::connect(addr).await.expect("Failed to connect");

        let request = tonic::Request::new(VadRequest {
            audio: content.iter().map(|x| x.to_ne_bytes().to_vec()).flatten().collect(),
            request_id: None,
        });

        let response = client.detect(request).await.expect("Failed to call RPC");

        let response = response.into_inner();
        response
            .intervals
            .iter()
            .for_each(|ts| println!("{} - {}", ts.start_s, ts.end_s));
        assert_eq!(response.intervals.len(), 0);
    }

    #[tokio::test]
    async fn test_vad_multithread() {
        start_server().await;

        let content = vec![32532i16; 16000 * 10];

        let addr = format!("http://localhost:{}", SETTINGS.server.port);
        let client = Arc::new(VadRecognizerClient::connect(addr).await.expect("Failed to connect"));

        let message = Arc::new(VadRequest {
            audio: content.iter().map(|x| x.to_ne_bytes().to_vec()).flatten().collect(),
            request_id: None,
        });

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let client = client.clone();
                let message = Arc::clone(&message);
                tokio::spawn(async move {
                    let response = (*client)
                        .clone()
                        .detect(tonic::Request::new(VadRequest::clone(&message)))
                        .await
                        .expect("Failed to call RPC");

                    let response = response.into_inner();
                    response
                        .intervals
                        .iter()
                        .for_each(|ts| println!("{} - {}", ts.start_s, ts.end_s));
                    assert_eq!(response.intervals.len(), 0);
                })
            })
            .collect();

        for handle in handles {
            handle.await.expect("Task failed");
        }
    }
}
