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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    SttWav,
    Mp3,
    Opus,
    Aac,
    Flac,
    Wav,
    Pcm,
}

impl OutputFormat {
    pub fn for_speech(format: AudioFormat) -> Option<Self> {
        match format {
            AudioFormat::Mp3 => Some(Self::Mp3),
            AudioFormat::Opus => Some(Self::Opus),
            AudioFormat::Aac => Some(Self::Aac),
            AudioFormat::Flac => Some(Self::Flac),
            AudioFormat::Wav => Some(Self::Wav),
            AudioFormat::Pcm => Some(Self::Pcm),
            _ => None,
        }
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

    #[test]
    fn for_speech_supported_formats() {
        assert_eq!(OutputFormat::for_speech(AudioFormat::Mp3), Some(OutputFormat::Mp3));
        assert_eq!(OutputFormat::for_speech(AudioFormat::Opus), Some(OutputFormat::Opus));
        assert_eq!(OutputFormat::for_speech(AudioFormat::Aac), Some(OutputFormat::Aac));
        assert_eq!(OutputFormat::for_speech(AudioFormat::Flac), Some(OutputFormat::Flac));
        assert_eq!(OutputFormat::for_speech(AudioFormat::Wav), Some(OutputFormat::Wav));
        assert_eq!(OutputFormat::for_speech(AudioFormat::Pcm), Some(OutputFormat::Pcm));
    }

    #[test]
    fn for_speech_unsupported_formats() {
        assert_eq!(OutputFormat::for_speech(AudioFormat::M4a), None);
        assert_eq!(OutputFormat::for_speech(AudioFormat::Ogg), None);
        assert_eq!(OutputFormat::for_speech(AudioFormat::Webm), None);
        assert_eq!(OutputFormat::for_speech(AudioFormat::Mp4), None);
        assert_eq!(OutputFormat::for_speech(AudioFormat::Mpeg), None);
        assert_eq!(OutputFormat::for_speech(AudioFormat::Mpga), None);
    }
}
