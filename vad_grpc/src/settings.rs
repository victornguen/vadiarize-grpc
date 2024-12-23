pub mod settings {
    use config::{Config, Environment, File};
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

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct VadSettings {
        pub model_path: String,
        pub sessions_num: u8,
    }

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct Settings {
        pub server: Server,
        pub logging: Logging,
        pub vad: VadSettings,
    }

    impl Settings {
        pub fn new(location: &str, env_prefix: &str) -> anyhow::Result<Self> {
            let mut builder = Config::builder();

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
