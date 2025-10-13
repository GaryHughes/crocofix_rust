use std::io;

#[derive(Debug, PartialEq)]
pub enum Error {
    IoError,
    DataFieldWithNoPrecedingSizeField,
    DataFieldWithNonNumericPreviousField,
    DataFieldWithNoTrailingSeparator,
    InvalidUtf8(std::str::Utf8Error),
    TagParseFailed
}

impl From<io::Error> for Error {
    fn from(_value: io::Error) -> Self {
        Error::IoError
    }
}