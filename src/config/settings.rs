use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Clone, Deserialize, accessory::Accessors)]
#[access(get)]
pub struct Settings {
    server: ServerConfig,
    provider: OpenRouterConfig,
    audio: AudioConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, accessory::Accessors)]
#[access(get)]
pub struct ServerConfig {
    host: IpAddr,
    port: u16,
}

#[derive(Debug, Clone, Deserialize, accessory::Accessors)]
#[access(get)]
pub struct OpenRouterConfig {
    base_url: Url,
    #[access(skip)]
    api_key: SecretString,
    default_transcription_model: String,
    default_speech_model: String,
    #[access(get(cp))]
    request_timeout_secs: u64,
    app_name: Option<String>,
    app_referer: Option<Url>,
}

#[derive(Debug, Clone, Deserialize, accessory::Accessors)]
#[access(get)]
pub struct LoggingConfig {
    filter: String,
}

#[derive(Debug, Clone, Deserialize, accessory::Accessors)]
#[access(get, defaults(get(cp)))]
pub struct AudioConfig {
    stt_sample_rate: u32,
    aac_bitrate_bps: u32,
    mp3_bitrate_bps: u32,
    opus_bitrate_bps: u32,
    pcm_sample_rate: u32,
}

impl OpenRouterConfig {
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}
