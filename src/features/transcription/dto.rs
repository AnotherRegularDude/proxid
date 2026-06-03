use serde::Serialize;

#[derive(Serialize)]
pub struct TranscriptionResponseBody {
    pub text: String,
}
