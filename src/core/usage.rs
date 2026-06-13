#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, accessory::Accessors)]
#[access(get)]
pub struct Usage {
    seconds: Option<f64>,
    cost: Option<f64>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

impl Usage {
    pub fn new(
        seconds: Option<f64>,
        cost: Option<f64>,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
    ) -> Self {
        Self { seconds, cost, input_tokens, output_tokens }
    }
}
