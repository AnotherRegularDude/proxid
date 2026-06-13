use crate::config::AudioConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, accessory::Accessors)]
#[access(get)]
pub struct TranscoderConfig {
    stt_sample_rate: u32,
    aac_bitrate_bps: u32,
    mp3_bitrate_bps: u32,
    opus_bitrate_bps: u32,
    pcm_sample_rate: u32,
}

impl TranscoderConfig {
    pub fn new(
        stt_sample_rate: u32,
        aac_bitrate_bps: u32,
        mp3_bitrate_bps: u32,
        opus_bitrate_bps: u32,
        pcm_sample_rate: u32,
    ) -> Self {
        Self {
            stt_sample_rate,
            aac_bitrate_bps,
            mp3_bitrate_bps,
            opus_bitrate_bps,
            pcm_sample_rate,
        }
    }
}

impl From<&AudioConfig> for TranscoderConfig {
    fn from(cfg: &AudioConfig) -> Self {
        Self {
            stt_sample_rate: cfg.stt_sample_rate(),
            aac_bitrate_bps: cfg.aac_bitrate_bps(),
            mp3_bitrate_bps: cfg.mp3_bitrate_bps(),
            opus_bitrate_bps: cfg.opus_bitrate_bps(),
            pcm_sample_rate: cfg.pcm_sample_rate(),
        }
    }
}
