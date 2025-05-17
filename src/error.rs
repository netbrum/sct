#[derive(Debug)]

pub enum Error {
    TermNotSet,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::TermNotSet => "$TERM not set or empty",
        };

        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {}
