use crate::processors::frontline::models::Parameter;
use clap::ValueEnum;
use cpal::HostId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessorSettings {
    pub bypassed: bool,
}

#[derive(Clone, Debug)]
pub enum Device {
    Default,
    Named(String),
}

impl FromStr for Device {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(if input == "default" {
            Device::Default
        } else {
            Device::Named(input.to_string())
        })
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Driver {
    Default,
    #[cfg(target_os = "windows")]
    Asio,
    #[cfg(target_os = "linux")]
    Jack,
}

impl Driver {
    pub fn host_id(&self) -> HostId {
        match &self {
            #[cfg(target_os = "linux")]
            Driver::Default => HostId::Alsa,
            #[cfg(target_os = "windows")]
            Driver::Default => HostId::Wasapi,
            #[cfg(target_os = "macos")]
            Driver::Default => HostId::CoreAudio,
            #[cfg(target_os = "linux")]
            Driver::Jack => HostId::Jack,
            #[cfg(target_os = "windows")]
            Driver::Asio => HostId::Asio,
        }
    }
}

#[derive(EnumString, AsRefStr, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessorType {
    SimpleDistortion,
    IR,
    Clone,
    CleanBooster,
    Ocd,
}

impl From<String> for ProcessorType {
    fn from(value: String) -> Self {
        value
            .as_str()
            .parse()
            .expect("processor type conversion failed for {value}")
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessorParameters {
    pub numeric: HashMap<Parameter, f32>,
    pub string: HashMap<Parameter, String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessorInfo {
    pub processor_type: ProcessorType,
    pub settings: ProcessorSettings,
    pub parameters: ProcessorParameters,
}
