use crate::processors::Processor;
use rubato::{FftFixedInOut, Resampler as RubatoResampler};

pub struct Resampler {
    buffer_size: usize,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
    resampler: FftFixedInOut<f32>,
    sample_rate: u32,
}

impl Resampler {
    fn create_from_settings(
        input_sample_rate: u32,
        target_sample_rate: u32,
        buffer_size: usize,
    ) -> Self {
        // The buffer size is doubled because the output delay is half of the buffer length
        let resampler = FftFixedInOut::<f32>::new(
            input_sample_rate as usize,
            target_sample_rate as usize,
            buffer_size * 2,
            1,
        )
        .unwrap();
        let input_buffer = resampler.input_buffer_allocate(true);
        let output_buffer = resampler.output_buffer_allocate(true);

        Self {
            buffer_size,
            input_buffer,
            output_buffer,
            resampler,
            sample_rate: input_sample_rate,
        }
    }

    pub fn set_target_sample_rate(&mut self, target_sample_rate: u32) {
        *self = Self::create_from_settings(self.sample_rate, target_sample_rate, self.buffer_size)
    }
}

impl Processor for Resampler {
    fn new(sample_rate: &u32, buffer_size: &usize) -> Self
    where
        Self: Sized,
    {
        Self::create_from_settings(*sample_rate, *sample_rate, *buffer_size)
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        self.input_buffer[0].resize(data.len(), 0.0);
        self.input_buffer[0].copy_from_slice(data);
        self.input_buffer[0].resize(self.resampler.input_frames_max(), 0.0);

        let _ = self.resampler.process_into_buffer(
            self.input_buffer.as_slice(),
            self.output_buffer.as_mut_slice(),
            None,
        );

        let cleaned_output_size = self.resampler.output_frames_max() / 2;
        let first_sample = data.first().copied().unwrap_or(0.0);
        let last_sample = data.last().copied().unwrap_or(0.0);
        data.resize(cleaned_output_size, 0.0);
        data.fill(0.0);
        data.copy_from_slice(&self.output_buffer[0][cleaned_output_size..]);
        if let Some(sample) = data.first_mut() {
            *sample = first_sample
        }
        if let Some(sample) = data.last_mut() {
            *sample = last_sample
        }
    }
}
