use crate::processors::frontline::models::{Parameter, ParameterValue};
use crate::processors::frontline::ocd::models::FilterMode;
use crate::processors::frontline::ocd::tone_stack::ToneStack;
use crate::processors::frontline::ocd::waveshaper::Waveshaper;
use crate::processors::frontline::FrontlineProcessor;
use crate::processors::internal::filter::models::FirstOrderFilterType;
use crate::processors::internal::filter::Filter;
use crate::processors::internal::gain::Gain;
use crate::processors::internal::resampler::Resampler;
use crate::processors::Processor;

mod models;
mod tone_stack;
mod waveshaper;

/* Copied from https://github.com/JanosGit/Schrammel_OJD */
pub struct Ocd {
    // Parameters
    filter_mode: FilterMode,
    drive: f32,

    // Chain
    high_pass_filter: Filter,
    pre_drive_boost_filter: Filter,
    pre_drive_notch_filter: Filter,
    gain: Gain,
    oversampler: Resampler,
    waveshaper: Waveshaper,
    downsampler: Resampler,
    post_drive_boost_1: Filter,
    post_drive_boost_2: Filter,
    post_drive_boost_3: Filter,
    low_pass_filter: Filter,
    tone_stack: ToneStack,
    volume: Gain,
}

impl Ocd {
    fn set_drive(&mut self, drive: f32) {
        self.drive = drive;
        let drive_squared = drive.powi(2);

        self.pre_drive_boost_filter.make_peak(
            -1400.0 * drive_squared + 500.0 * drive + 1600.0,
            -0.1 * drive + 0.15,
            32.0 * drive + 4.0,
        );
        self.pre_drive_notch_filter
            .make_peak(8_000.0, 0.8, -5.0 * drive_squared);

        let post_drive_boost_1_frequency = match self.filter_mode {
            FilterMode::HighPass => 2052.0,
            FilterMode::LowPass => 2781.0,
        };
        let post_drive_boost_1_gain = match self.filter_mode {
            FilterMode::HighPass => 4.6,
            FilterMode::LowPass => 4.38,
        };
        self.post_drive_boost_1.make_peak(
            post_drive_boost_1_frequency,
            0.5,
            post_drive_boost_1_gain,
        );
        self.post_drive_boost_2
            .make_peak(74.0, 0.2, 7.38 * drive + 8.12);
        let post_drive_boost_3_gain = match self.filter_mode {
            FilterMode::HighPass => 10.0,
            FilterMode::LowPass => 16.9,
        };
        self.post_drive_boost_3
            .make_peak(2935.0, 0.1, post_drive_boost_3_gain);
    }
}

impl Processor for Ocd {
    fn new(sample_rate: &u32, block_size: &usize) -> Self {
        let filter_mode = FilterMode::HighPass;
        let drive = 0.5;

        let mut gain = Gain::new(sample_rate, block_size);
        let mut volume = Gain::new(sample_rate, block_size);
        let mut high_pass_filter = Filter::new(sample_rate, block_size);
        let mut low_pass_filter = Filter::new(sample_rate, block_size);
        let waveshaper = Waveshaper::new(sample_rate, block_size);
        let mut tone_stack = ToneStack::new(sample_rate, block_size);

        let mut oversampler = Resampler::new(sample_rate, block_size);
        oversampler.set_target_sample_rate(sample_rate * 16);
        let downsampler = Resampler::new(&(sample_rate * 16), &(block_size * 16));
        oversampler.set_target_sample_rate(*sample_rate);

        gain.set_linear_gain(11.0);
        volume.set_db_range(-60.0, -20.0);
        volume.decimal = 0.5;

        high_pass_filter.make_first_order(FirstOrderFilterType::HighPass, 30.0);
        low_pass_filter.make_first_order(FirstOrderFilterType::LowPass, 6_300.0);

        tone_stack.set_tone(0.5);
        tone_stack.set_filter_mode(filter_mode);

        let mut processor = Self {
            // Parameters
            drive,
            filter_mode,

            // Chain
            gain,
            oversampler,
            waveshaper,
            downsampler,
            volume,
            high_pass_filter,
            low_pass_filter,
            pre_drive_boost_filter: Filter::new(sample_rate, block_size),
            pre_drive_notch_filter: Filter::new(sample_rate, block_size),
            post_drive_boost_1: Filter::new(sample_rate, block_size),
            post_drive_boost_2: Filter::new(sample_rate, block_size),
            post_drive_boost_3: Filter::new(sample_rate, block_size),
            tone_stack,
        };
        processor.set_drive(drive);
        processor
    }

    #[inline(always)]
    fn process(&mut self, data: &mut Vec<f32>) {
        // TODO: filters clearly don't work well, as the resulting sound is very highs-only
        // Plus, the resampler is too much expensive for the CPU.. Check if you can tweak that, otherwise I guess it's game over
        self.high_pass_filter.process(data);
        self.pre_drive_boost_filter.process(data);
        self.pre_drive_notch_filter.process(data);
        self.gain.process(data);
        self.oversampler.process(data);
        self.waveshaper.process(data);
        self.downsampler.process(data);
        self.post_drive_boost_1.process(data);
        self.post_drive_boost_2.process(data);
        self.post_drive_boost_3.process(data);
        self.low_pass_filter.process(data);
        self.tone_stack.process(data);
        self.volume.process(data);
    }
}

impl FrontlineProcessor for Ocd {
    fn get_parameter(&self, parameter: Parameter) -> Option<ParameterValue> {
        match parameter {
            Parameter::Drive => Some(self.drive.into()),
            Parameter::Volume => Some(self.volume.decimal.into()),
            Parameter::Tone => Some(self.tone_stack.get_tone().into()),
            Parameter::FilterSwitch => Some(match self.filter_mode {
                FilterMode::HighPass => 0.0.into(),
                FilterMode::LowPass => 1.0.into(),
            }),
            _ => None,
        }
    }

    fn set_parameter(&mut self, parameter: Parameter, value: ParameterValue) {
        match (parameter, value) {
            (Parameter::Drive, ParameterValue::Numeric(wrapped_value)) => {
                self.set_drive(wrapped_value)
            }
            (Parameter::Volume, ParameterValue::Numeric(wrapped_value)) => {
                self.volume.decimal = wrapped_value
            }
            (Parameter::Tone, ParameterValue::Numeric(wrapped_value)) => {
                self.tone_stack.set_tone(wrapped_value)
            }
            (Parameter::FilterSwitch, ParameterValue::Numeric(0.0..=0.5)) => {
                self.filter_mode = FilterMode::HighPass;
                self.set_drive(self.drive);
                self.tone_stack.set_filter_mode(self.filter_mode);
            }
            (Parameter::FilterSwitch, ParameterValue::Numeric(0.5..=1.0)) => {
                self.filter_mode = FilterMode::LowPass;
                self.set_drive(self.drive);
                self.tone_stack.set_filter_mode(self.filter_mode);
            }
            _ => {}
        }
    }
}
