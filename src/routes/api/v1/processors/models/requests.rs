use crate::managers::dsp::models::ProcessorType;
use crate::processors::frontline::models::ParameterValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateProcessorRequest {
    pub index: usize,
    pub processor_type: ProcessorType,
}

#[derive(Serialize, Deserialize)]
pub struct SwapProcessorRequest {
    pub processor_type: ProcessorType,
}

#[derive(Serialize, Deserialize)]
pub struct EditParameterRequest {
    pub value: ParameterValue,
}

#[derive(Serialize, Deserialize)]
pub struct MoveProcessorRequest {
    pub destination_index: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SetProcessorBypassedRequest {
    pub bypassed: bool,
}
