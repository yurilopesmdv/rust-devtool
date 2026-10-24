pub mod catalog;
pub mod cli;
pub mod config;
pub mod doctor;
pub mod generator;
pub mod planner;

use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub code: u8,
}

impl Error {
    pub fn input(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
        }
    }

    pub fn operation(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
