use bytes::Bytes;
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    M4a,
    Ogg,
    Webm,
    Mp4,
    Mpeg,
    Mpga,
    Aac,
    Opus,
    Pcm,
}

#[derive(Debug, Clone)]
pub struct SourceAudio {
    pub bytes: Bytes,
    pub format: AudioFormat,
}

impl AudioFormat {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Flac => "audio/flac",
            Self::M4a => "audio/mp4",
            Self::Ogg => "audio/ogg",
            Self::Webm => "audio/webm",
            Self::Mp4 => "audio/mp4",
            Self::Mpeg => "audio/mpeg",
            Self::Mpga => "audio/mpeg",
            Self::Aac => "audio/aac",
            Self::Opus => "audio/opus",
            Self::Pcm => "audio/L16",
        }
    }
}

impl SourceAudio {
    pub fn new(bytes: Bytes, format: AudioFormat) -> Self {
        Self { bytes, format }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn from_extension_known() {
        assert_eq!(AudioFormat::from_str("mp3"), Ok(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_str("wav"), Ok(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_str("flac"), Ok(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_str("m4a"), Ok(AudioFormat::M4a));
        assert_eq!(AudioFormat::from_str("ogg"), Ok(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_str("webm"), Ok(AudioFormat::Webm));
        assert_eq!(AudioFormat::from_str("mp4"), Ok(AudioFormat::Mp4));
        assert_eq!(AudioFormat::from_str("mpeg"), Ok(AudioFormat::Mpeg));
        assert_eq!(AudioFormat::from_str("mpga"), Ok(AudioFormat::Mpga));
        assert_eq!(AudioFormat::from_str("aac"), Ok(AudioFormat::Aac));
        assert_eq!(AudioFormat::from_str("opus"), Ok(AudioFormat::Opus));
        assert_eq!(AudioFormat::from_str("pcm"), Ok(AudioFormat::Pcm));
    }

    #[test]
    fn from_extension_case_insensitive() {
        assert_eq!(AudioFormat::from_str("MP3"), Ok(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_str("Wav"), Ok(AudioFormat::Wav));
    }

    #[test]
    fn from_extension_unknown() {
        assert!(AudioFormat::from_str("xyz").is_err());
        assert!(AudioFormat::from_str("exe").is_err());
    }

    #[test]
    fn mime_returns_correct_content_type() {
        assert_eq!(AudioFormat::Mp3.mime(), "audio/mpeg");
        assert_eq!(AudioFormat::Pcm.mime(), "audio/L16");
        assert_eq!(AudioFormat::Wav.mime(), "audio/wav");
    }
}
