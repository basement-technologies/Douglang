#[derive(Debug, Clone)]
pub struct ParseErr {
    pub message: String,
}

impl ParseErr {
    pub fn new(line: usize, column: usize, msg: &str) -> Self {
        ParseErr {
            message: format!("You are literally trolling. {msg} on line {line}, column {column}"),
        }
    }
}

impl std::fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseErr {}
