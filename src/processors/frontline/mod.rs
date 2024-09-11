use crate::managers::dsp::models::ProcessorType;
use crate::processors::frontline::clean_booster::CleanBooster;
use crate::processors::frontline::models::{Parameter, ParameterValue};
use crate::processors::frontline::ocd::Ocd;
use crate::processors::frontline::simple_distortion::SimpleDistortion;
use crate::processors::Processor;
use regex::Regex;
use std::any::type_name_of_val;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub mod clean_booster;
pub mod models;
pub mod ocd;
pub mod simple_distortion;

pub type BoxedProcessor = Box<dyn FrontlineProcessor + Send>;

pub trait FrontlineProcessor: Processor {
    fn get_parameter(&self, parameter: Parameter) -> Option<ParameterValue>;
    fn set_parameter(&mut self, parameter: Parameter, value: ParameterValue);

    fn get_numeric_parameters(&self) -> HashMap<Parameter, f32> {
        Parameter::iter()
            .filter_map(|parameter| {
                self.get_parameter(parameter)
                    .and_then(|value| match value {
                        ParameterValue::Numeric(wrapped_value) => Some(wrapped_value),
                        ParameterValue::String(_) => None,
                    })
                    .map(|value| (parameter, value))
            })
            .collect()
    }

    fn get_string_parameters(&self) -> HashMap<Parameter, String> {
        Parameter::iter()
            .filter_map(|parameter| {
                self.get_parameter(parameter)
                    .and_then(|value| match value {
                        ParameterValue::String(wrapped_value) => Some(wrapped_value),
                        ParameterValue::Numeric(_) => None,
                    })
                    .map(|value| (parameter, value))
            })
            .collect()
    }

    fn get_type(&self) -> ProcessorType {
        let object_type = type_name_of_val(self);
        let regex = Regex::new(r"^.*::").unwrap();
        regex.replace_all(object_type, "").to_string().into()
    }
}

pub fn create_processor_from_type(
    processor_type: &ProcessorType,
    sample_rate: &u32,
    buffer_size: &usize,
) -> BoxedProcessor {
    match processor_type {
        ProcessorType::SimpleDistortion => {
            Box::new(SimpleDistortion::new(sample_rate, buffer_size))
        }
        ProcessorType::CleanBooster => Box::new(CleanBooster::new(sample_rate, buffer_size)),
        ProcessorType::Ocd => Box::new(Ocd::new(sample_rate, buffer_size)),
        ProcessorType::Clone => todo!("Implement CloneProcessor"),
        ProcessorType::IR => todo!("Implement IRProcessor"),
    }
}
