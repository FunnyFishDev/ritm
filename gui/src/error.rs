use std::fmt::Display;

#[derive(Debug)]
pub enum RitmError {
    GuiError(String),
    CoreError(String)
}

impl Display for RitmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::GuiError(s) => s,
            Self::CoreError(s) => s,
        };
        writeln!(f, "{}", error)
    }
}