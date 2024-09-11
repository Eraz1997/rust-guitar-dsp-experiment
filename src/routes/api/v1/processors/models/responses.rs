use crate::managers::dsp::models::ProcessorParameters;
use crate::processors::frontline::models::Parameter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct CreateProcessorResponse {
    pub parameters: ProcessorParameters,
}

#[derive(Serialize, Deserialize)]
pub struct SwapProcessorResponse {
    pub parameters: ProcessorParameters,
}

#[derive(Serialize, Deserialize)]
pub struct GetStringParameterValuesResponse {
    pub values: HashMap<Parameter, Vec<String>>,
}
