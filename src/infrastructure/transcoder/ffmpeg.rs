use std::{io::Write, process::Stdio};

pub use super::SourceAudio;
pub use super::config::TranscoderConfig;

use anyhow::{Result, bail};
use bytes::Bytes;
use tempfile::TempPath;
use tokio::process::Command;

use super::Transcoder;
use crate::core::audio::{AudioFormat, OutputFormat};

pub struct FfmpegTranscoder {
    cfg: TranscoderConfig,
}

impl Clone for FfmpegTranscoder {
    fn clone(&self) -> Self {
        Self { cfg: self.cfg.clone() }
    }
}

impl FfmpegTranscoder {
    pub fn new(cfg: TranscoderConfig) -> Result<Self> {
        Ok(Self { cfg })
    }
}

impl Transcoder for FfmpegTranscoder {
    async fn convert(&self, src: SourceAudio, target: OutputFormat) -> Result<Bytes> {
        let input_path = write_source_to_tempfile(&src)?;

        let (encoder_name, output_sr, output_channels, bitrate, output_ext, output_fmt) =
            encoder_params(&self.cfg, target);

        let output_tmp = tempfile::Builder::new()
            .prefix("proxid_out_")
            .suffix(&format!(".{output_ext}"))
            .tempfile()?;
        let output_path = output_tmp.into_temp_path();

        let mut input_args: Vec<String> = Vec::new();
        if src.format == AudioFormat::Pcm {
            // Hardcoded: 24 kHz / 16-bit / mono (OpenRouter TTS format)
            input_args.push("-f".to_string());
            input_args.push("s16le".to_string());
            input_args.push("-ar".to_string());
            input_args.push("24000".to_string());
            input_args.push("-ac".to_string());
            input_args.push("1".to_string());
        }

        let mut output_args = vec![
            "-acodec".to_string(),
            encoder_name.to_string(),
            "-ar".to_string(),
            output_sr.to_string(),
            "-ac".to_string(),
            output_channels.to_string(),
        ];
        if bitrate > 0 {
            output_args.push("-b:a".to_string());
            output_args.push(bitrate.to_string());
        }
        if let Some(fmt) = output_fmt {
            output_args.push("-f".to_string());
            output_args.push(fmt.to_string());
        }
        output_args.push("-y".to_string());

        let input_str = input_path.to_string_lossy().to_string();
        let output_str = output_path.to_string_lossy().to_string();

        let status = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("warning")
            .args(&input_args)
            .arg("-i")
            .arg(&input_str)
            .args(&output_args)
            .arg(&output_str)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if !status.success() {
            bail!("ffmpeg exited with non-zero status: {status}");
        }

        let data = tokio::fs::read(&output_path).await?;

        tracing::debug!("Data length: {}", data.len());

        Ok(Bytes::from(data))
    }
}

fn write_source_to_tempfile(src: &SourceAudio) -> Result<TempPath> {
    let ext = match src.format {
        AudioFormat::Pcm => "pcm",
        other => other.into(),
    };
    let mut tmp = tempfile::Builder::new().suffix(&format!(".{ext}")).tempfile()?;
    tmp.write_all(&src.bytes)?;
    tmp.flush()?;
    Ok(tmp.into_temp_path())
}

fn encoder_params(
    cfg: &TranscoderConfig,
    target: OutputFormat,
) -> (&'static str, u32, u16, i64, &'static str, Option<&'static str>) {
    match target {
        OutputFormat::Mp3 => ("libmp3lame", 44_100, 1, cfg.mp3_bitrate_bps as i64, "mp3", None),
        OutputFormat::Opus => ("libopus", 48_000, 1, cfg.opus_bitrate_bps as i64, "ogg", None),
        OutputFormat::Aac => {
            ("aac", cfg.pcm_sample_rate, 1, cfg.aac_bitrate_bps as i64, "aac", None)
        }
        OutputFormat::Flac => ("flac", cfg.pcm_sample_rate, 1, 0, "flac", None),
        OutputFormat::Wav => ("pcm_s16le", cfg.pcm_sample_rate, 1, 0, "wav", None),
        OutputFormat::Pcm => ("pcm_s16le", cfg.pcm_sample_rate, 1, 0, "pcm", Some("s16le")),
        OutputFormat::SttWav => ("pcm_s16le", cfg.stt_sample_rate, 1, 0, "wav", None),
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
        TranscoderConfig {
            stt_sample_rate: 16_000,
            aac_bitrate_bps: 160_000,
            mp3_bitrate_bps: 128_000,
            opus_bitrate_bps: 64_000,
            pcm_sample_rate: 24_000,
        }
    }

    fn convert(target: OutputFormat) -> Bytes {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let src = SourceAudio { bytes: make_silent_pcm(), format: AudioFormat::Pcm };
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(transcoder.convert(src, target))
            .expect("conversion should succeed")
    }

    #[test]
    fn pcm_to_wav_has_riff_header() {
        let result = convert(OutputFormat::Wav);
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"RIFF", "WAV should start with RIFF");
    }

    #[test]
    fn pcm_to_mp3_has_sync_byte() {
        let result = convert(OutputFormat::Mp3);
        assert!(!result.is_empty());
        let first = result[0];
        assert!(
            first == 0xFF || first == 0x49,
            "MP3 should start with sync byte (0xFF) or ID3 (0x49), got 0x{first:02x}"
        );
    }

    #[test]
    fn pcm_to_opus_has_ogg_magic() {
        let result = convert(OutputFormat::Opus);
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"OggS", "Opus container should start with OggS");
    }

    #[test]
    fn pcm_to_flac_has_flac_magic() {
        let result = convert(OutputFormat::Flac);
        assert!(!result.is_empty());
        assert_eq!(&result[..4], b"fLaC", "FLAC should start with fLaC");
    }

    #[test]
    fn pcm_to_aac_has_adts_sync() {
        let result = convert(OutputFormat::Aac);
        assert!(!result.is_empty());
        assert_eq!(result[0], 0xFF, "AAC should start with ADTS sync byte 0xFF");
    }

    #[test]
    fn pcm_to_pcm_preserves_expected_length() {
        let input_len = make_silent_pcm().len();
        let result = convert(OutputFormat::Pcm);
        assert_eq!(result.len(), input_len, "PCM→PCM same sr should preserve byte count");
    }

    #[test]
    fn wav_to_sttwav_has_16khz_mono() {
        let transcoder = FfmpegTranscoder::new(test_cfg()).unwrap();
        let wav_bytes = {
            let src = SourceAudio { bytes: make_silent_pcm(), format: AudioFormat::Pcm };
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(transcoder.convert(src, OutputFormat::Wav))
                .unwrap()
        };

        let result = {
            let src = SourceAudio { bytes: wav_bytes, format: AudioFormat::Wav };
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(transcoder.convert(src, OutputFormat::SttWav))
                .unwrap()
        };

        assert!(!result.is_empty());
        let sr = u32::from_le_bytes([result[24], result[25], result[26], result[27]]);
        assert_eq!(sr, 16_000, "SttWav sample rate must be 16000");
        let ch = u16::from_le_bytes([result[22], result[23]]);
        assert_eq!(ch, 1, "SttWav channels must be 1");
    }
}
