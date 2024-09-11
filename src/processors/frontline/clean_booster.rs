use crate::processors::frontline::models::{Parameter, ParameterValue};
use crate::processors::frontline::FrontlineProcessor;
use crate::processors::internal::gain::Gain;
use crate::processors::Processor;

pub struct CleanBooster {
    drive: Gain,
}

impl Processor for CleanBooster {
    fn new(sample_rate: &u32, block_size: &usize) -> Self {
        let mut drive = Gain::new(sample_rate, block_size);
        drive.set_db_range(0.0, 20.0);
        drive.decimal = 0.25;
        Self { drive }
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        self.drive.process(data)
    }
}

impl FrontlineProcessor for CleanBooster {
    fn get_parameter(&self, parameter: Parameter) -> Option<ParameterValue> {
        match parameter {
            Parameter::Drive => Some(self.drive.decimal.into()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter: Parameter, value: ParameterValue) {
        if let (Parameter::Drive, ParameterValue::Numeric(wrapped_value)) = (parameter, value) {
            self.drive.decimal = wrapped_value
        }
    }
}
