use serde::Serialize;

/// Standard ADR 0007 machine-readable response envelope.
#[derive(Debug, Serialize)]
pub struct CliEnvelope<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<CliErrorPayload>,
}

#[derive(Debug, Serialize)]
pub struct CliErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl<T: Serialize> CliEnvelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(CliErrorPayload {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
        }
    }
}

/// Helper to render data based on the `--json` flag.
pub struct OutputRenderer {
    pub json: bool,
}

impl OutputRenderer {
    pub fn new(json: bool) -> Self {
        Self { json }
    }

    pub fn success<T: Serialize>(&self, data: &T, human_message: impl FnOnce() -> String) {
        if self.json {
            let envelope = CliEnvelope::ok(data);
            println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
        } else {
            println!("{}", human_message());
        }
    }

    pub fn error(&self, code: impl Into<String>, message: impl Into<String>) {
        let code = code.into();
        let message = message.into();
        if self.json {
            let envelope: CliEnvelope<()> = CliEnvelope::err(code, message);
            println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
        } else {
            eprintln!("Error [{}]: {}", code, message);
        }
    }
}
