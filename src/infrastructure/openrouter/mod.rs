mod client;
mod dto;

#[allow(unused_imports, reason = "re-exported for external use")]
pub use client::{
    OpenRouterClient, OpenRouterClientBuilder, SpeechSynthRequest, SynthesisedSpeech,
    TranscriptionInput, TranscriptionOutput,
};
