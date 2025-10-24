use std::io;

#[derive(Debug, PartialEq)]
pub enum Error {
    IoError,
    DataFieldWithNoPrecedingSizeField,
    DataFieldWithNonNumericPreviousField(String),
    DataFieldWithNoTrailingSeparator,
    InvalidUtf8(std::str::Utf8Error),
    TagParseFailed(String),
    MessageDoesNotContainMsgType,
    UnknownOrder(String),
    MessageDoesNotContainFieldWithTag(u32),
    UnsupportedMsgType(String),
    OrderIndexOutOfRange(usize),
    OrderBookAlreadyContainsOrderWithKey(String),
    OrderBookDoesNotContainOrderWithKey(String)
}

impl From<io::Error> for Error {
    fn from(_value: io::Error) -> Self {
        Error::IoError
    }
}