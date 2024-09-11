#[derive(Debug, Clone)]
pub enum Error {
    CardNotFound,
    CannotOpenMixer,
    CannotFindControl,
    CannotSetInputMode,
    Generic,
}

impl From<alsa::Error> for Error {
    fn from(_: alsa::Error) -> Self {
        Error::Generic
    }
}
