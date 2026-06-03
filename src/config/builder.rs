use crate::config::settings::Settings;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};

const DEFAULT_CONFIG_FILE: &str = include_str!("../../config/default.toml");

pub enum MergedData {
    File(String),
    Content(String),
}

pub struct ConfigBuilder {
    merged_data: Option<MergedData>,
    env_prefix: String,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_custom_config(mut self, path: Option<MergedData>) -> Self {
        self.merged_data = path;
        self
    }

    pub fn with_env_prefix(mut self, prefix: String) -> Self {
        self.env_prefix = prefix;
        self
    }

    pub fn load(self) -> anyhow::Result<Settings> {
        let figment = figment_with_merged_custom_config(self.merged_data);
        let figment = figment.merge(Env::prefixed(&self.env_prefix).split("__"));

        figment.extract::<Settings>().map_err(|err| anyhow::anyhow!(err))
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self { merged_data: None, env_prefix: "PROXID__".to_string() }
    }
}

fn figment_with_merged_custom_config(merged_data: Option<MergedData>) -> Figment {
    let figment = Figment::new().merge(Toml::string(DEFAULT_CONFIG_FILE));

    let Some(data) = merged_data else {
        return figment;
    };

    match data {
        MergedData::File(path) => {
            tracing::debug!(path = %path, "merging custom config file");
            figment.merge(Toml::file(path))
        }
        MergedData::Content(content) => figment.merge(Toml::string(&content)),
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;

    fn config_file_path() -> String {
        let mut path = current_dir().expect("Get current directory");
        path.push("config/test.toml");
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn load_fails_without_api_key() {
        let settings: Result<Settings, _> = ConfigBuilder::new().load();
        assert!(settings.is_err());
    }

    #[test]
    fn load_succeeds_with_api_key_from_toml() {
        let toml_str = r#"
[server]
host = "127.0.0.1"
port = 9999

[provider]
base_url = "https://api.example.com"
api_key = "sk-test-key"
default_transcription_model = "test-model"
default_speech_model = "test-tts-model"
request_timeout_secs = 30

[logging]
filter = "proxid=warn"
"#;
        let settings = ConfigBuilder::new()
            .with_custom_config(Some(MergedData::Content(toml_str.to_string())))
            .load()
            .unwrap();

        assert_eq!(settings.server.port, 9999);
        assert_eq!(settings.provider.default_transcription_model, "test-model");
    }

    #[test]
    fn defaults_applied_when_missing() {
        let toml_str = r#"
[provider]
base_url = "https://api.example.com"
api_key = "sk-test-key"
"#;
        let settings = ConfigBuilder::new()
            .with_custom_config(Some(MergedData::Content(toml_str.to_string())))
            .load()
            .expect("Properly load custom config from TOML string");

        assert_eq!(settings.server.port, 8800);
        assert_eq!(settings.provider.default_transcription_model, "openai/whisper-large-v3");
        assert_eq!(settings.provider.request_timeout_secs, 60);
    }

    #[test]
    fn toml_string_with_invalid_port_fails() {
        let toml_str = r#"
[server]
port = "not_a_number"

[provider]
api_key = "sk-test-key"
"#;
        let result = ConfigBuilder::new()
            .with_custom_config(Some(MergedData::Content(toml_str.to_string())))
            .load();

        assert!(result.is_err());
    }

    #[test]
    fn redefine_settings_from_custom_toml_file() {
        let settings = ConfigBuilder::new()
            .with_custom_config(Some(MergedData::File(config_file_path())))
            .load()
            .expect("Properly load custom config from TOML file");

        assert_eq!(settings.provider.api_key(), "test-key");
        assert_eq!(settings.server.port, 8899);
    }
}
