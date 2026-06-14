use super::dto;

use anyhow::anyhow;
use base64::prelude::*;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::core::audio::{AudioFormat, SourceAudio};
use crate::core::audio_io::{SpeechPayload, TranscribePayload, TranscriptPayload};

pub struct OpenRouterClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    default_transcription_model: String,
    default_speech_model: String,
    app_name: Option<String>,
    app_referer: Option<Url>,
}

#[derive(Default)]
pub struct OpenRouterClientBuilder {
    base_url: Option<Url>,
    api_key: Option<SecretString>,
    default_transcription_model: Option<String>,
    default_speech_model: Option<String>,
    timeout_secs: Option<u64>,
    app_name: Option<String>,
    app_referer: Option<Url>,
}

impl OpenRouterClient {
    pub fn builder() -> OpenRouterClientBuilder {
        OpenRouterClientBuilder::default()
    }

    pub fn default_transcription_model(&self) -> &str {
        &self.default_transcription_model
    }

    pub fn default_speech_model(&self) -> &str {
        &self.default_speech_model
    }

    fn add_app_headers(&self, mut builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref name) = self.app_name {
            builder = builder.header("X-Title", name);
        }
        if let Some(ref referer) = self.app_referer {
            builder = builder.header("HTTP-Referer", referer.as_str());
        }
        builder
    }

    #[tracing::instrument(skip_all, fields(format = %input.format.as_ref(), size = input.audio_bytes.len()))]
    pub async fn transcribe(
        &self,
        input: TranscribePayload,
    ) -> Result<TranscriptPayload, anyhow::Error> {
        let encoded = BASE64_STANDARD.encode(&input.audio_bytes);
        tracing::debug!(encoded_len = encoded.len(), "audio encoded to base64");

        let request = dto::ProviderTranscribeRequest::new(
            input.model.unwrap_or_else(|| self.default_transcription_model.clone()),
            dto::ProviderInputAudio::new(encoded, input.format.into()),
            input.temperature,
        );

        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| anyhow!("failed to set path segments"))?
            .extend(["audio", "transcriptions"]);

        tracing::debug!(url = %url, "transcription URL");

        let mut builder =
            self.http.post(url.clone()).bearer_auth(self.api_key.expose_secret()).json(&request);
        builder = self.add_app_headers(builder);

        tracing::debug!(url = %url, "sending transcription request to provider");

        let response = builder.send().await?;
        let status = response.status();
        tracing::debug!(url = %url, status = %status, "received response from provider");

        if status.is_success() {
            let dto: dto::ProviderTranscribeResponse = response.json().await?;
            tracing::debug!(
                text_len = dto.text.len(),
                has_usage = dto.usage.is_some(),
                "transcription successful"
            );
            return Ok(dto.into());
        }

        Err(map_response_error(response).await)
    }

    #[tracing::instrument(skip_all, fields(model = ?request.model, voice = %request.voice))]
    pub async fn synthesise(&self, request: SpeechPayload) -> Result<SourceAudio, anyhow::Error> {
        let dto = dto::ProviderSpeechRequest::new(
            request.model.as_deref().unwrap_or(self.default_speech_model()),
            &request.input,
            &request.voice,
            "pcm",
            request.speed,
        );

        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| anyhow!("failed to set path segments"))?
            .extend(["audio", "speech"]);

        tracing::debug!(url = %url, "speech synthesis URL");

        let mut builder =
            self.http.post(url.clone()).bearer_auth(self.api_key.expose_secret()).json(&dto);
        builder = self.add_app_headers(builder);

        tracing::debug!(url = %url, "sending speech synthesis request to provider");

        let response = builder.send().await?;
        let status = response.status();
        tracing::debug!(url = %url, status = %status, "received response from provider");

        if status.is_success() {
            let bytes = response.bytes().await?;
            tracing::debug!(len = bytes.len(), "speech synthesis successful");
            return Ok(SourceAudio::new(bytes, AudioFormat::Pcm));
        }

        Err(map_response_error(response).await)
    }
}

impl OpenRouterClientBuilder {
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    pub fn api_key(mut self, key: SecretString) -> Self {
        self.api_key = Some(key);
        self
    }

    pub fn default_transcription_model(mut self, model: String) -> Self {
        self.default_transcription_model = Some(model);
        self
    }

    pub fn default_speech_model(mut self, model: String) -> Self {
        self.default_speech_model = Some(model);
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn app_name(mut self, name: Option<String>) -> Self {
        self.app_name = name;
        self
    }

    pub fn app_referer(mut self, referer: Option<Url>) -> Self {
        self.app_referer = referer;
        self
    }

    pub fn build(self) -> Result<OpenRouterClient, anyhow::Error> {
        let timeout = std::time::Duration::from_secs(self.timeout_secs.unwrap_or(60));

        let http = reqwest::Client::builder().timeout(timeout).build()?;

        let base_url = self.base_url.ok_or_else(|| anyhow!("base_url is required"))?;
        let api_key = self.api_key.ok_or_else(|| anyhow::anyhow!("api_key is required"))?;
        let default_transcription_model = self
            .default_transcription_model
            .unwrap_or_else(|| String::from("openai/whisper-large-v3-turbo"));
        let default_speech_model = self
            .default_speech_model
            .unwrap_or_else(|| String::from("google/gemini-3.1-flash-tts-preview"));

        Ok(OpenRouterClient {
            http,
            base_url,
            api_key,
            default_transcription_model,
            default_speech_model,
            app_name: self.app_name,
            app_referer: self.app_referer,
        })
    }
}

impl std::fmt::Debug for OpenRouterClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenRouterClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"***")
            .field("default_transcription_model", &self.default_transcription_model)
            .field("default_speech_model", &self.default_speech_model)
            .field("app_name", &self.app_name)
            .field("app_referer", &self.app_referer)
            .finish()
    }
}

async fn map_response_error(resp: reqwest::Response) -> anyhow::Error {
    let status = resp.status().as_u16();
    match status {
        401 => anyhow::anyhow!("provider returned unauthorized"),
        429 => anyhow::anyhow!("provider returned rate limit exceeded"),
        _ => {
            let body = resp.text().await.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "failed to read error response body");
                String::new()
            });
            let msg = serde_json::from_str::<dto::ErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);
            anyhow::anyhow!("upstream {status}: {msg}")
        }
    }
}
