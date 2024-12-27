pub mod settings {
    use config::builder::DefaultState;
    use config::{Config, ConfigBuilder, Environment, File};
    use log::{log, Level};
    use serde::{Deserialize, Serialize};
    use serde_json::to_string_pretty;
    use std::path::Path;

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct Server {
        pub host: String,
        pub port: i32,
    }

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct Logging {
        pub log_level: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(default)]
    pub struct VadSettings {
        pub model_path: String,
        pub sessions_num: u8,
        pub frame_size: usize,
        pub threshold: f32,
        pub min_silence_duration_ms: usize,
        pub speech_pad_ms: usize,
        pub min_speech_duration_ms: usize,
        pub max_speech_duration_s: f32,
    }

    impl Default for VadSettings {
        fn default() -> Self {
            Self {
                model_path: "model/silero_vad.onnx".to_string(),
                sessions_num: 1,
                frame_size: 64,
                threshold: 0.5,
                min_silence_duration_ms: 0,
                speech_pad_ms: 64,
                min_speech_duration_ms: 64,
                max_speech_duration_s: f32::INFINITY,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct Settings {
        pub server: Server,
        pub logging: Logging,
        pub vad: VadSettings,
    }

    impl Settings {
        pub fn new(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
            let mut builder = ConfigBuilder::<DefaultState>::default();

            if Path::new(location).exists() {
                builder = builder.add_source(File::with_name(location));
            } else {
                log!(Level::Warn, "Config file not found")
            }

            builder = builder.add_source(
                Environment::with_prefix(env_prefix)
                    .separator("__")
                    .prefix_separator("__"),
            );

            let settings = builder.build()?.try_deserialize()?;
            Ok(settings)
        }

        pub fn json_pretty(&self) -> String {
            to_string_pretty(&self).unwrap()
        }
    }
}
