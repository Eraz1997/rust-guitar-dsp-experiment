use crate::processors::internal::filter::models::FirstOrderFilterType;
use crate::processors::Processor;
use std::f32::consts::PI;

pub mod models;

pub struct Filter {
    numerators: Vec<f32>,
    denominators: Vec<f32>,
    input_history: Vec<f32>,
    output_history: Vec<f32>,
    sample_rate: f32,
}

impl Filter {
    #[inline(always)]
    fn get_order(&self) -> usize {
        self.denominators.len() - 1
    }

    fn reset_state(&mut self) {
        let size = self.get_order() + 1;
        self.input_history = vec![0.0; size];
        self.output_history = vec![0.0; size];
    }

    pub fn make_first_order(&mut self, filter_type: FirstOrderFilterType, frequency: f32) {
        let omega = 2.0 * PI * frequency / self.sample_rate;
        let alpha = omega.sin() / (2.0 * omega.tan());

        self.numerators = match filter_type {
            FirstOrderFilterType::LowPass => vec![1.0, alpha - 1.0],
            FirstOrderFilterType::HighPass => vec![1.0, alpha - 1.0],
        };
        self.denominators = match filter_type {
            FirstOrderFilterType::LowPass => vec![alpha, alpha],
            FirstOrderFilterType::HighPass => vec![1.0 - alpha, alpha - 1.0],
        };

        self.reset_state();
    }

    pub fn make_peak(&mut self, frequency: f32, q: f32, gain_db: f32) {
        let linear_gain = 10.0_f32.powf(gain_db / 20.0);
        let omega = 2.0 * PI * frequency / self.sample_rate;
        let alpha = omega.sin() / (2.0 * q);
        let cos_omega = omega.cos();

        self.numerators = vec![
            1.0 + alpha * linear_gain,
            -2.0 * cos_omega,
            1.0 - alpha / linear_gain,
        ];
        self.denominators = vec![
            1.0 + alpha * linear_gain,
            -2.0 * cos_omega,
            1.0 - alpha * linear_gain,
        ];

        self.reset_state();
    }
}

impl Processor for Filter {
    fn new(sample_rate: &u32, _: &usize) -> Self
    where
        Self: Sized,
    {
        Self {
            numerators: vec![1.0],
            denominators: vec![1.0],
            sample_rate: *sample_rate as f32,
            input_history: vec![0.0],
            output_history: vec![0.0],
        }
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        for sample in data {
            for history_index in (1..self.input_history.len()).rev() {
                self.input_history[history_index] = self.input_history[history_index - 1];
                self.output_history[history_index] = self.output_history[history_index - 1];
            }
            self.input_history[0] = *sample;

            *sample = 0.0;
            for coefficient_index in 0..self.denominators.len() {
                *sample +=
                    self.denominators[coefficient_index] * self.input_history[coefficient_index];
            }
            for coefficient_index in 1..self.numerators.len() {
                *sample -=
                    self.numerators[coefficient_index] * self.output_history[coefficient_index];
            }

            *sample /= self.numerators[0];
            self.output_history[0] = *sample;
        }
    }
}
