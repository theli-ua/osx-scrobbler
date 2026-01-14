use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    NotPlaying,
    NoData,
    DeserializationFailed,
    AppCommandFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let msg = match self {
            Error::NotPlaying => Cow::Borrowed("Failed to convert values"),
            Error::NoData => Cow::Borrowed("Failed to get current_track Data!"),
            Error::DeserializationFailed => {
                Cow::Borrowed("Failed to deserialize current_track Data!")
            }
            Error::AppCommandFailed => Cow::Borrowed("Failed to execute AppCommand"),
        };

        f.write_str(&msg)
    }
}
