use std::str::FromStr;

use axum::extract::Multipart;
use bytes::Bytes;

use crate::app::AppState;
use crate::core::audio::AudioFormat;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::transcoder::Transcoder;

#[derive(accessory::Accessors)]
#[access(get)]
pub(super) struct UploadedAudio {
    bytes: Bytes,
    format: AudioFormat,
}

#[derive(accessory::Accessors)]
#[access(get)]
pub(super) struct TranscriptionForm {
    audio: UploadedAudio,
    model: String,
    temperature: Option<f32>,
}

impl UploadedAudio {
    pub fn new(bytes: Bytes, format: AudioFormat) -> Self {
        Self { bytes, format }
    }
}

impl TranscriptionForm {
    pub fn new(audio: UploadedAudio, model: String, temperature: Option<f32>) -> Self {
        Self { audio, model, temperature }
    }
}

pub(super) async fn parse_transcription_form<T: Transcoder>(
    mut multipart: Multipart,
    state: &AppState<T>,
) -> AppResult<TranscriptionForm> {
    let mut audio: Option<UploadedAudio> = None;
    let mut model = state.provider().default_transcription_model().to_string();
    let mut temperature: Option<f32> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!(error = %e, "failed to read multipart field");
        AppError::BadRequest(format!("failed to read field: {e}"))
    })? {
        let name = field.name().ok_or(AppError::MissingField("field name"))?.to_string();

        match name.as_ref() {
            "file" => {
                let filename =
                    field.file_name().ok_or(AppError::MissingField("file name"))?.to_string();

                let ext = std::path::Path::new(&filename)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .ok_or_else(|| AppError::BadRequest("missing file extension".to_string()))?;

                let format = AudioFormat::from_str(ext).map_err(|_| {
                    tracing::warn!(ext = %ext, filename = %filename, "unsupported audio format");
                    AppError::UnsupportedFormat(ext.to_string())
                })?;

                let data = field.bytes().await.map_err(|e| {
                    tracing::error!(filename = %filename, error = %e, "failed to read file bytes");
                    AppError::BadRequest(format!("failed to read file: {e}"))
                })?;

                tracing::info!(filename = %filename, ext = %ext, format = %format, "parsing completed");
                audio = Some(UploadedAudio::new(data, format));
            }
            "model" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!(error = %e, "failed to read model field");
                    AppError::BadRequest(format!("failed to read model: {e}"))
                })?;
                if !text.is_empty() {
                    model = text;
                }
            }
            "temperature" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!(error = %e, "failed to read temperature field");
                    AppError::BadRequest(format!("failed to read temperature: {e}"))
                })?;
                let Ok(temp) = text.parse::<f32>() else {
                    return Err(AppError::BadRequest(format!("invalid temperature: {text}")));
                };
                temperature = Some(temp);
            }
            _ => {}
        }
    }

    let audio = audio.ok_or(AppError::MissingField("file"))?;

    Ok(TranscriptionForm::new(audio, model, temperature))
}
