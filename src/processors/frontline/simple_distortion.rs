use crate::processors::frontline::models::{Parameter, ParameterValue};
use crate::processors::frontline::FrontlineProcessor;
use crate::processors::internal::gain::Gain;
use crate::processors::Processor;

pub struct SimpleDistortion {
    drive: Gain,
    volume: Gain,
}

impl Processor for SimpleDistortion {
    fn new(sample_rate: &u32, block_size: &usize) -> Self {
        let mut drive = Gain::new(sample_rate, block_size);
        let mut volume = Gain::new(sample_rate, block_size);

        drive.set_db_range(2.0, 34.0);
        drive.decimal = 0.3;

        volume.set_db_range(-15.0, 3.0);
        volume.decimal = 0.5;

        Self { drive, volume }
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        self.drive.process(data);
        for sample in data.iter_mut() {
            *sample = (*sample).tanh();
        }
        self.volume.process(data);
    }
}

impl FrontlineProcessor for SimpleDistortion {
    fn get_parameter(&self, parameter: Parameter) -> Option<ParameterValue> {
        match parameter {
            Parameter::Drive => Some(self.drive.decimal.into()),
            Parameter::Volume => Some(self.volume.decimal.into()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter: Parameter, value: ParameterValue) {
        match (parameter, value) {
            (Parameter::Drive, ParameterValue::Numeric(wrapped_value)) => {
                self.drive.decimal = wrapped_value
            }
            (Parameter::Volume, ParameterValue::Numeric(wrapped_value)) => {
                self.volume.decimal = wrapped_value
            }
            _ => {}
        }
    }
}
