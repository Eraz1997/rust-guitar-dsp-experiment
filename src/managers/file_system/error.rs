use std::io;

#[derive(Debug)]
pub enum Error {
    Conversion,
    Generic(io::Error),
    HomeDirectoryNotFound,
    NotFound,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Generic(value)
    }
}
