use std::error::Error;
use std::fmt::{Debug, Display, Formatter};


/// A wrapper the map the Debug trait to Display
pub struct ErrorDisplayWrapper {
    error: Box<dyn Error>
}

impl Debug for ErrorDisplayWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl Display for ErrorDisplayWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl From<Box<dyn Error>> for ErrorDisplayWrapper {
    fn from(value: Box<dyn Error>) -> Self {
        Self {
            error: value
        }
    }
}

impl From<iced::Error> for ErrorDisplayWrapper {
    fn from(value: iced::Error) -> Self {
        Self {
            error: Box::new(value)
        }
    }
}