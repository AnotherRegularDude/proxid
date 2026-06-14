pub mod config;
pub mod ffmpeg;

use crate::core::audio::{AudioFormat, SourceAudio};

pub use ffmpeg::FfmpegTranscoder;

pub trait Transcoder: Clone + Send + Sync + 'static {
    fn convert(
        &self,
        src: SourceAudio,
        target: AudioFormat,
    ) -> impl Future<Output = anyhow::Result<SourceAudio>> + Send + '_;
}
