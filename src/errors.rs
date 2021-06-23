use std::fmt;

#[derive(Debug)]
pub struct HttpError {
    code: u16,
    details: String,
}

impl HttpError {
    pub fn new(code: u16, details: &str) -> HttpError {
        HttpError {
            code: match code {
                200 => 200,
                401 => 421,
                500 => 432,
                _ => 432,
            },
            details: details.to_string(),
        }
    }
}

impl std::error::Error for HttpError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.code, self.details)
    }
}
