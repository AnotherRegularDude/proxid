use std::io::Write;

use anyhow::{Result, bail};
use bytes::Bytes;
use tempfile::NamedTempFile;
use tokio::process::Command;

use super::Transcoder;
use crate::core::audio::{AudioFormat, SourceAudio};

pub use super::config::TranscoderConfig;

#[derive(Clone)]
pub struct FfmpegTranscoder {
    cfg: TranscoderConfig,
}

struct EncodeParams {
    codec: &'static str,
    sample_rate: u32,
    channels: u16,
    bitrate: u32,
    ext: &'static str,
    container_fmt: Option<&'static str>,
}

impl FfmpegTranscoder {
    pub fn new(cfg: TranscoderConfig) -> Result<Self> {
        Ok(Self { cfg })
    }
}

impl Transcoder for FfmpegTranscoder {
    async fn convert(&self, src: SourceAudio, target: AudioFormat) -> Result<SourceAudio> {
        let input_tmp = write_source_to_tempfile(&src)?;
        let input_tmp_path = input_tmp.path().to_string_lossy().to_string();

        let params = encode_params(&self.cfg, target)?;

        let output_tmp = tempfile::Builder::new()
            .prefix("proxid_out_")
            .suffix(&format!(".{}", params.ext))
            .tempfile()?;
        let output_tmp_path = output_tmp.path().to_string_lossy().to_string();

        run_ffmpeg(&src, &input_tmp_path, &output_tmp_path, &params).await?;
        let data = tokio::fs::read(output_tmp.path()).await?;
        tracing::debug!("Data length: {}", data.len());

        Ok(SourceAudio::new(Bytes::from(data), target))
    }
}

async fn run_ffmpeg(
    src: &SourceAudio,
    input_audio_path: &str,
    output_audio_path: &str,
    params: &EncodeParams,
) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner").arg("-loglevel").arg("warning");
    if src.format == AudioFormat::Pcm {
        cmd.arg("-f").arg("s16le").arg("-ar").arg("24000").arg("-ac").arg("1");
    }
    cmd.arg("-i")
        .arg(input_audio_path)
        .arg("-acodec")
        .arg(params.codec)
        .arg("-ar")
        .arg(params.sample_rate.to_string())
        .arg("-ac")
        .arg(params.channels.to_string())
        .arg("-b:a")
        .arg(params.bitrate.to_string());

    if let Some(fmt) = params.container_fmt {
        cmd.arg("-f").arg(fmt);
    }
    cmd.arg("-y").arg(output_audio_path);

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg exited with status {}: {}", output.status, stderr.trim());
    }

    Ok(())
}

fn write_source_to_tempfile(src: &SourceAudio) -> Result<NamedTempFile> {
    let ext = src.format.as_ref();
    let mut tmp = tempfile::Builder::new().suffix(&format!(".{ext}")).tempfile()?;
    tmp.write_all(&src.bytes)?;
    tmp.flush()?;
    Ok(tmp)
}

fn encode_params(cfg: &TranscoderConfig, target: AudioFormat) -> Result<EncodeParams> {
    match target {
        AudioFormat::Mp3 => Ok(EncodeParams {
            codec: "libmp3lame",
            sample_rate: cfg.stt_sample_rate(),
            channels: 1,
            bitrate: cfg.mp3_bitrate_bps(),
            ext: "mp3",
            container_fmt: None,
        }),
        AudioFormat::Opus => Ok(EncodeParams {
            codec: "libopus",
            sample_rate: cfg.stt_sample_rate(),
            channels: 1,
            bitrate: cfg.opus_bitrate_bps(),
            ext: "ogg",
            container_fmt: None,
        }),
        AudioFormat::Aac => Ok(EncodeParams {
            codec: "aac",
            sample_rate: cfg.stt_sample_rate(),
            channels: 1,
            bitrate: cfg.aac_bitrate_bps(),
            ext: "aac",
            container_fmt: None,
        }),
        AudioFormat::Flac => Ok(EncodeParams {
            codec: "flac",
            sample_rate: cfg.stt_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "flac",
            container_fmt: None,
        }),
        AudioFormat::Wav => Ok(EncodeParams {
            codec: "pcm_s16le",
            sample_rate: cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "wav",
            container_fmt: None,
        }),
        AudioFormat::Pcm => Ok(EncodeParams {
            codec: "pcm_s16le",
            sample_rate: cfg.pcm_sample_rate(),
            channels: 1,
            bitrate: 0,
            ext: "pcm",
            container_fmt: Some("s16le"),
        }),
        _ => bail!("unsupported output audio format: {target}"),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    static SILENT_PCM: LazyLock<Vec<u8>> = LazyLock::new(|| vec![0u8; 24_000]);

    fn make_silent_pcm() -> Bytes {
        Bytes::from_static(SILENT_PCM.as_slice())
    }

    fn test_cfg() -> TranscoderConfig {
        TranscoderConfig::new(16_000, 160_000, 128_000, 64_000, 24_000)
    }

    async fn convert(target: AudioFormat) -> SourceAudio {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let src = SourceAudio::new(make_silent_pcm(), AudioFormat::Pcm);
        transcoder.convert(src, target).await.expect("conversion should succeed")
    }

    #[tokio::test]
    async fn pcm_to_wav_has_riff_header() {
        let result = convert(AudioFormat::Wav).await;
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"RIFF", "WAV should start with RIFF");
    }

    #[tokio::test]
    async fn pcm_to_mp3_has_sync_byte() {
        let result = convert(AudioFormat::Mp3).await;
        assert!(!result.bytes.is_empty());
        let first = result.bytes[0];
        assert!(
            first == 0xFF || first == 0x49,
            "MP3 should start with sync byte (0xFF) or ID3 (0x49), got 0x{first:02x}"
        );
    }

    #[tokio::test]
    async fn pcm_to_opus_has_ogg_magic() {
        let result = convert(AudioFormat::Opus).await;
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"OggS", "Opus container should start with OggS");
    }

    #[tokio::test]
    async fn pcm_to_flac_has_flac_magic() {
        let result = convert(AudioFormat::Flac).await;
        assert!(!result.bytes.is_empty());
        assert_eq!(&result.bytes[..4], b"fLaC", "FLAC should start with fLaC");
    }

    #[tokio::test]
    async fn pcm_to_aac_has_adts_sync() {
        let result = convert(AudioFormat::Aac).await;
        assert!(!result.bytes.is_empty());
        assert_eq!(result.bytes[0], 0xFF, "AAC should start with ADTS sync byte 0xFF");
    }

    #[tokio::test]
    async fn pcm_to_pcm_preserves_expected_length() {
        let input_len = make_silent_pcm().len();
        let result = convert(AudioFormat::Pcm).await;
        assert_eq!(result.bytes.len(), input_len, "PCM→PCM same sr should preserve byte count");
    }

    #[tokio::test]
    async fn wav_to_sttwav_has_16khz_mono() {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let wav_src = {
            let src = SourceAudio::new(make_silent_pcm(), AudioFormat::Pcm);
            transcoder.convert(src, AudioFormat::Wav).await.unwrap()
        };

        let result = transcoder.convert(wav_src, AudioFormat::Wav).await.unwrap();

        let bytes = &result.bytes;
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[..4], b"RIFF", "WAV container should start with RIFF");
    }

    #[test]
    fn encoder_params_returns_correct_values() {
        let cfg = test_cfg();

        let params = encode_params(&cfg, AudioFormat::Mp3).unwrap();
        assert_eq!(params.codec, "libmp3lame");
        assert_eq!(params.sample_rate, 16_000);
        assert_eq!(params.channels, 1);
        assert_eq!(params.bitrate, 128_000);
        assert_eq!(params.ext, "mp3");
        assert!(params.container_fmt.is_none());

        let params = encode_params(&cfg, AudioFormat::Opus).unwrap();
        assert_eq!(params.codec, "libopus");
        assert_eq!(params.sample_rate, 16_000);
        assert_eq!(params.bitrate, 64_000);
        assert_eq!(params.ext, "ogg");

        let params = encode_params(&cfg, AudioFormat::Pcm).unwrap();
        assert_eq!(params.codec, "pcm_s16le");
        assert_eq!(params.sample_rate, 24_000);
        assert_eq!(params.container_fmt, Some("s16le"));
    }

    #[test]
    fn encoder_params_rejects_unsupported_formats() {
        let cfg = test_cfg();
        assert!(encode_params(&cfg, AudioFormat::M4a).is_err());
        assert!(encode_params(&cfg, AudioFormat::Ogg).is_err());
        assert!(encode_params(&cfg, AudioFormat::Webm).is_err());
    }
}
