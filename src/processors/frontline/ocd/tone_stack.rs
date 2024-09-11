use crate::processors::frontline::ocd::models::FilterMode;
use crate::processors::internal::filter::models::FirstOrderFilterType;
use crate::processors::internal::filter::Filter;
use crate::processors::internal::gain::Gain;
use crate::processors::Processor;

pub struct ToneStack {
    filter_mode: FilterMode,
    tone: f32,
    buffer: Vec<f32>,
    high_pass_filter: Filter,
    low_pass_filter: Filter,
    high_pass_gain: Gain,
}

impl ToneStack {
    pub fn set_filter_mode(&mut self, filter_mode: FilterMode) {
        let frequency = match filter_mode {
            FilterMode::HighPass => 358.0,
            FilterMode::LowPass => 160.0,
        };
        self.low_pass_filter
            .make_first_order(FirstOrderFilterType::HighPass, frequency);
        self.high_pass_filter
            .make_first_order(FirstOrderFilterType::LowPass, frequency);

        self.filter_mode = filter_mode;
        self.set_tone(self.tone);
    }

    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone;
        let gain = match self.filter_mode {
            FilterMode::HighPass => 0.7,
            FilterMode::LowPass => 0.2,
        };
        self.high_pass_gain.set_linear_gain(self.tone * gain);
    }

    pub fn get_tone(&self) -> f32 {
        self.tone
    }
}

impl Processor for ToneStack {
    fn new(sample_rate: &u32, block_size: &usize) -> Self
    where
        Self: Sized,
    {
        let high_pass_filter = Filter::new(sample_rate, block_size);
        let low_pass_filter = Filter::new(sample_rate, block_size);
        let high_pass_gain = Gain::new(sample_rate, block_size);
        let filter_mode = FilterMode::HighPass;

        let mut processor = Self {
            filter_mode,
            buffer: vec![0.0; *block_size],
            low_pass_filter,
            high_pass_filter,
            high_pass_gain,
            tone: 0.2,
        };
        processor.set_filter_mode(filter_mode);
        processor
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        // This truncates or extends the buffer without reallocating memory
        self.buffer.resize(data.len(), 0.0);
        self.buffer.copy_from_slice(data);
        self.high_pass_filter.process(&mut self.buffer);
        self.high_pass_gain.process(&mut self.buffer);
        self.low_pass_filter.process(data);
        for (sample, buffer_sample) in data.iter_mut().zip(&self.buffer) {
            *sample += *buffer_sample;
        }
    }
}
