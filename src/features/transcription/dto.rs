use serde::Serialize;

#[derive(Serialize)]
pub struct TranscriptionResponseBody {
    pub text: String,
}

impl TranscriptionResponseBody {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}
