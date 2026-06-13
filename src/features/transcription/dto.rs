use serde::Serialize;

#[derive(Serialize, accessory::Accessors)]
#[access(get)]
pub struct TranscriptionResponseBody {
    text: String,
}

impl TranscriptionResponseBody {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}
