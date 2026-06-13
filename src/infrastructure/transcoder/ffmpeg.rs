use std::io::Write;

use anyhow::{Result, bail};
use bytes::Bytes;
use tempfile::NamedTempFile;
use tokio::process::Command;

use super::Transcoder;
use crate::core::audio::{AudioFormat, OutputFormat};

pub use super::SourceAudio;
pub use super::config::TranscoderConfig;

#[derive(Clone)]
pub struct FfmpegTranscoder {
    cfg: TranscoderConfig,
}

impl FfmpegTranscoder {
    pub fn new(cfg: TranscoderConfig) -> Result<Self> {
        Ok(Self { cfg })
    }
}

struct EncoderParams {
    codec: &'static str,
    sample_rate: u32,
    channels: u16,
    bitrate: i64,
    ext: &'static str,
    container_fmt: Option<&'static str>,
}

impl Transcoder for FfmpegTranscoder {
    async fn convert(&self, src: SourceAudio, target: OutputFormat) -> Result<Bytes> {
        let input_tmp = write_source_to_tempfile(&src)?;
        let input_str = input_tmp.path().to_string_lossy().to_string();

        let params = encoder_params(&self.cfg, target);

        let output_tmp = tempfile::Builder::new()
            .prefix("proxid_out_")
            .suffix(&format!(".{}", params.ext))
            .tempfile()?;
        let output_str = output_tmp.path().to_string_lossy().to_string();

        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-hide_banner").arg("-loglevel").arg("warning");
        if *src.format() == AudioFormat::Pcm {
            cmd.arg("-f").arg("s16le").arg("-ar").arg("24000").arg("-ac").arg("1");
        }
        cmd.arg("-i")
            .arg(&input_str)
            .arg("-acodec")
            .arg(params.codec)
            .arg("-ar")
            .arg(params.sample_rate.to_string())
            .arg("-ac")
            .arg(params.channels.to_string());
        if params.bitrate > 0 {
            cmd.arg("-b:a").arg(params.bitrate.to_string());
        }
        if let Some(fmt) = params.container_fmt {
            cmd.arg("-f").arg(fmt);
        }
        cmd.arg("-y").arg(&output_str);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("ffmpeg exited with status {}: {}", output.status, stderr.trim());
        }

        let data = tokio::fs::read(output_tmp.path()).await?;

        tracing::debug!("Data length: {}", data.len());

        drop(input_tmp);
        drop(output_tmp);

        Ok(Bytes::from(data))
    }
}

fn write_source_to_tempfile(src: &SourceAudio) -> Result<NamedTempFile> {
    let ext = src.format().as_ref();
    let mut tmp = tempfile::Builder::new().suffix(&format!(".{ext}")).tempfile()?;
    tmp.write_all(src.bytes())?;
    tmp.flush()?;
    Ok(tmp)
}

fn encoder_params(cfg: &TranscoderConfig, target: OutputFormat) -> EncoderParams {
    match target {
        OutputFormat::Mp3 => EncoderParams {
            codec: "libmp3lame",
            sample_rate: 44_100,
            channels: 1,
            bitrate: *cfg.mp3_bitrate_bps() as i64,
            ext: "mp3",
            container_fmt: None,
        },
        OutputFormat::Opus => EncoderParams {
            codec: "libopus",
            sample_rate: 48_000,
            channels: 1,
            bitrate: *cfg.opus_bitrate_bps() as i64,
            ext: "ogg",
            container_fmt: None,
        },
        OutputFormat::Aac => EncoderParams {
            codec: "aac",
            sample_rate: *cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: *cfg.aac_bitrate_bps() as i64,
            ext: "aac",
            container_fmt: None,
        },
        OutputFormat::Flac => EncoderParams {
            codec: "flac",
            sample_rate: *cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "flac",
            container_fmt: None,
        },
        OutputFormat::Wav => EncoderParams {
            codec: "pcm_s16le",
            sample_rate: *cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "wav",
            container_fmt: None,
        },
        OutputFormat::Pcm => EncoderParams {
            codec: "pcm_s16le",
            sample_rate: *cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "pcm",
            container_fmt: Some("s16le"),
        },
        OutputFormat::SttWav => EncoderParams {
            codec: "pcm_s16le",
            sample_rate: *cfg.stt_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "wav",
            container_fmt: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;
    use crate::core::audio::OutputFormat;

    static SILENT_PCM: LazyLock<Vec<u8>> = LazyLock::new(|| vec![0u8; 24_000]);

    fn make_silent_pcm() -> Bytes {
        Bytes::from_static(SILENT_PCM.as_slice())
    }

    fn test_cfg() -> TranscoderConfig {
        TranscoderConfig::new(16_000, 160_000, 128_000, 64_000, 24_000)
    }

    async fn convert(target: OutputFormat) -> Bytes {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let src = SourceAudio::new(make_silent_pcm(), AudioFormat::Pcm);
        transcoder.convert(src, target).await.expect("conversion should succeed")
    }

    #[tokio::test]
    async fn pcm_to_wav_has_riff_header() {
        let result = convert(OutputFormat::Wav).await;
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"RIFF", "WAV should start with RIFF");
    }

    #[tokio::test]
    async fn pcm_to_mp3_has_sync_byte() {
        let result = convert(OutputFormat::Mp3).await;
        assert!(!result.is_empty());
        let first = result[0];
        assert!(
            first == 0xFF || first == 0x49,
            "MP3 should start with sync byte (0xFF) or ID3 (0x49), got 0x{first:02x}"
        );
    }

    #[tokio::test]
    async fn pcm_to_opus_has_ogg_magic() {
        let result = convert(OutputFormat::Opus).await;
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"OggS", "Opus container should start with OggS");
    }

    #[tokio::test]
    async fn pcm_to_flac_has_flac_magic() {
        let result = convert(OutputFormat::Flac).await;
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"fLaC", "FLAC should start with fLaC");
    }

    #[tokio::test]
    async fn pcm_to_aac_has_adts_sync() {
        let result = convert(OutputFormat::Aac).await;
        assert!(!result.is_empty());
        assert_eq!(result[0], 0xFF, "AAC should start with ADTS sync byte 0xFF");
    }

    #[tokio::test]
    async fn pcm_to_pcm_preserves_expected_length() {
        let input_len = make_silent_pcm().len();
        let result = convert(OutputFormat::Pcm).await;
        assert_eq!(result.len(), input_len, "PCM→PCM same sr should preserve byte count");
    }

    #[tokio::test]
    async fn wav_to_sttwav_has_16khz_mono() {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let wav_bytes = {
            let src = SourceAudio::new(make_silent_pcm(), AudioFormat::Pcm);
            transcoder.convert(src, OutputFormat::Wav).await.unwrap()
        };

        let result = {
            let src = SourceAudio::new(wav_bytes, AudioFormat::Wav);
            transcoder.convert(src, OutputFormat::SttWav).await.unwrap()
        };

        assert!(!result.is_empty());
        let sr = u32::from_le_bytes([result[24], result[25], result[26], result[27]]);
        assert_eq!(sr, 16_000, "SttWav sample rate must be 16000");
        let ch = u16::from_le_bytes([result[22], result[23]]);
        assert_eq!(ch, 1, "SttWav channels must be 1");
    }

    #[test]
    fn encoder_params_returns_correct_values() {
        let cfg = test_cfg();

        let params = encoder_params(&cfg, OutputFormat::Mp3);
        assert_eq!(params.codec, "libmp3lame");
        assert_eq!(params.sample_rate, 44_100);
        assert_eq!(params.channels, 1);
        assert_eq!(params.bitrate, 128_000);
        assert_eq!(params.ext, "mp3");
        assert!(params.container_fmt.is_none());

        let params = encoder_params(&cfg, OutputFormat::Opus);
        assert_eq!(params.codec, "libopus");
        assert_eq!(params.sample_rate, 48_000);
        assert_eq!(params.bitrate, 64_000);
        assert_eq!(params.ext, "ogg");

        let params = encoder_params(&cfg, OutputFormat::Pcm);
        assert_eq!(params.codec, "pcm_s16le");
        assert_eq!(params.sample_rate, 24_000);
        assert_eq!(params.container_fmt, Some("s16le"));

        let params = encoder_params(&cfg, OutputFormat::SttWav);
        assert_eq!(params.codec, "pcm_s16le");
        assert_eq!(params.sample_rate, 16_000);
        assert_eq!(params.ext, "wav");
        assert!(params.container_fmt.is_none());
    }
}
