use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub provider: OpenRouterConfig,
    pub audio: AudioConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterConfig {
    pub base_url: Url,
    api_key: SecretString,
    pub default_transcription_model: String,
    pub default_speech_model: String,
    pub request_timeout_secs: u64,
    pub app_name: Option<String>,
    pub app_referer: Option<Url>,
}

impl OpenRouterConfig {
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub filter: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioConfig {
    pub stt_sample_rate: u32,
    pub aac_bitrate_bps: u32,
    pub mp3_bitrate_bps: u32,
    pub opus_bitrate_bps: u32,
    pub pcm_sample_rate: u32,
}
