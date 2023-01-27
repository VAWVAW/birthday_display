use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// A wrapper that map the Debug trait to Display
pub struct ErrorDisplayWrapper {
    error: Box<dyn Error>,
}

impl Debug for ErrorDisplayWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(debug_assertions))]
        return Display::fmt(&self.error, f);

        #[cfg(debug_assertions)]
        return Debug::fmt(&self.error, f);
    }
}

impl Display for ErrorDisplayWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl From<Box<dyn Error>> for ErrorDisplayWrapper {
    fn from(value: Box<dyn Error>) -> Self {
        Self { error: value }
    }
}
