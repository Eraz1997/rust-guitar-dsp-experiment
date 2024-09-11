use crate::processors::Processor;

pub struct Waveshaper {}

impl Processor for Waveshaper {
    fn new(_: &u32, _: &usize) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        for sample in data {
            *sample = match *sample {
                ..=-1.7 => -1.0,
                -1.7..=-0.3 => {
                    *sample += 0.3;
                    *sample + ((*sample).powi(2)) / (4.0 * (1.0 - 0.3)) - 0.3
                }
                0.9..=1.1 => {
                    *sample -= 0.9;
                    *sample - ((*sample).powi(2)) / (4.0 * (1.0 - 0.9)) + 0.9
                }
                1.1.. => 1.0,
                _ => *sample,
            }
        }
    }
}
