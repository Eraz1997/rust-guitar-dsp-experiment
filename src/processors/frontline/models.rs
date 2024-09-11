use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, EnumIter, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Parameter {
    Drive,
    Volume,
    FilePath,
    Tone,
    FilterSwitch,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterValue {
    Numeric(f32),
    String(String),
}

impl From<f32> for ParameterValue {
    fn from(value: f32) -> Self {
        ParameterValue::Numeric(value)
    }
}

impl From<String> for ParameterValue {
    fn from(value: String) -> Self {
        ParameterValue::String(value)
    }
}
