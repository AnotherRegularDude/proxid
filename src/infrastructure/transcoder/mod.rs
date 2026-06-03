pub mod config;
pub mod ffmpeg;

pub use config::TranscoderConfig;
pub use ffmpeg::FfmpegTranscoder;

use crate::core::audio::{AudioFormat, OutputFormat};
use bytes::Bytes;

pub trait Transcoder: Send + Sync + 'static {
    fn convert(
        &self,
        src: SourceAudio,
        target: OutputFormat,
    ) -> impl Future<Output = anyhow::Result<Bytes>> + Send + '_;
}

#[derive(Debug, Clone)]
pub struct SourceAudio {
    pub bytes: Bytes,
    pub format: AudioFormat,
}
