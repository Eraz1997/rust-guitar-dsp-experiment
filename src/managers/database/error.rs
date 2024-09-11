use crate::managers::database::error::Error::Generic;

#[derive(Debug)]
pub enum Error {
    Generic,
}

impl From<mongodb::error::Error> for Error {
    fn from(_: mongodb::error::Error) -> Self {
        Generic
    }
}
