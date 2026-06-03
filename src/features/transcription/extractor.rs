use std::str::FromStr;

use axum::extract::Multipart;
use bytes::Bytes;

use crate::app::AppState;
use crate::core::audio::AudioFormat;
use crate::core::error::{AppError, AppResult};
use crate::infrastructure::transcoder::Transcoder;

pub(super) struct UploadedAudio {
    pub bytes: Bytes,
    pub format: AudioFormat,
    #[allow(dead_code)]
    pub filename: String,
}

pub(super) struct TranscriptionForm {
    pub audio: UploadedAudio,
    pub model: String,
    pub temperature: Option<f32>,
}

pub(super) async fn parse_transcription_form<T: Transcoder>(
    mut multipart: Multipart,
    state: &AppState<T>,
) -> AppResult<TranscriptionForm> {
    let mut audio: Option<UploadedAudio> = None;
    let model = state.provider().default_transcription_model().to_string();
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
                    .ok_or_else(|| AppError::Internal("Can't parse filename".to_string()))?;

                let format = AudioFormat::from_str(ext).map_err(|_| {
                    tracing::warn!(ext = %ext, filename = %filename, "unsupported audio format");
                    AppError::UnsupportedFormat(ext.to_string())
                })?;

                let data = field.bytes().await.map_err(|e| {
                    tracing::error!(filename = %filename, error = %e, "failed to read file bytes");
                    AppError::BadRequest(format!("failed to read file: {e}"))
                })?;

                audio = Some(UploadedAudio { bytes: data, format, filename });
            }
            "temperature" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!(error = %e, "failed to read temperature field");
                    AppError::BadRequest(format!("failed to read temperature: {e}"))
                })?;
                let Ok(temp) = text.parse::<f32>() else {
                    return Err(AppError::BadRequest(format!("invalid temperature: {text}")));
                };
                let _ = temperature.insert(temp);
            }
            _ => {}
        }
    }

    let audio = audio.ok_or(AppError::MissingField("file"))?;

    Ok(TranscriptionForm { audio, model, temperature })
}
