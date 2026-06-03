use serde::Deserialize;

use crate::config::AudioConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct TranscoderConfig {
    pub stt_sample_rate: u32,
    pub aac_bitrate_bps: u32,
    pub mp3_bitrate_bps: u32,
    pub opus_bitrate_bps: u32,
    pub pcm_sample_rate: u32,
}

impl From<&AudioConfig> for TranscoderConfig {
    fn from(cfg: &AudioConfig) -> Self {
        Self {
            stt_sample_rate: cfg.stt_sample_rate,
            aac_bitrate_bps: cfg.aac_bitrate_bps,
            mp3_bitrate_bps: cfg.mp3_bitrate_bps,
            opus_bitrate_bps: cfg.opus_bitrate_bps,
            pcm_sample_rate: cfg.pcm_sample_rate,
        }
    }
}
