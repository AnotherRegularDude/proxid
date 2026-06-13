pub mod config;
pub mod ffmpeg;

use crate::core::audio::{AudioFormat, OutputFormat};
use bytes::Bytes;

pub use config::TranscoderConfig;
pub use ffmpeg::FfmpegTranscoder;

pub trait Transcoder: Clone + Send + Sync + 'static {
    fn convert(
        &self,
        src: SourceAudio,
        target: OutputFormat,
    ) -> impl Future<Output = anyhow::Result<Bytes>> + Send + '_;
}

#[derive(Debug, Clone, accessory::Accessors)]
#[access(get)]
pub struct SourceAudio {
    bytes: Bytes,
    format: AudioFormat,
}

impl SourceAudio {
    pub fn new(bytes: Bytes, format: AudioFormat) -> Self {
        Self { bytes, format }
    }
}
