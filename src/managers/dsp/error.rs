use cpal::{
    BuildStreamError, DeviceNameError, DevicesError, HostUnavailable, PauseStreamError,
    PlayStreamError,
};

#[derive(Debug)]
pub enum Error {
    BuildStream,
    DefaultDeviceNotFound,
    Device,
    DeviceName,
    Host(HostUnavailable),
    NamedDeviceNotFound,
    PauseStream,
    PlayStream,
}

pub enum TransformProcessorError {
    NotFound,
}

impl From<HostUnavailable> for Error {
    fn from(value: HostUnavailable) -> Self {
        Self::Host(value)
    }
}

impl From<DevicesError> for Error {
    fn from(_: DevicesError) -> Self {
        Self::Device
    }
}

impl From<DeviceNameError> for Error {
    fn from(_: DeviceNameError) -> Self {
        Self::DeviceName
    }
}

impl From<BuildStreamError> for Error {
    fn from(_: BuildStreamError) -> Self {
        Self::BuildStream
    }
}

impl From<PlayStreamError> for Error {
    fn from(_: PlayStreamError) -> Self {
        Self::PlayStream
    }
}

impl From<PauseStreamError> for Error {
    fn from(_: PauseStreamError) -> Self {
        Self::PauseStream
    }
}
