use crate::managers::dsp::models::{Device, Driver};
use clap::Parser;
use cpal::{BufferSize, SampleRate, StreamConfig};

#[derive(Parser, Debug)]
pub struct Settings {
    #[arg(long, default_value = "512")]
    pub buffer_size: usize,
    #[arg(long, default_value = "mongodb://localhost:27017")]
    pub database_connection_string: String,
    #[arg(long, default_value = "default")]
    pub driver: Driver,
    #[arg(long, default_value = "false")]
    pub hifiberry_enabled: bool,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value = "1")]
    pub input_channels: u16,
    #[arg(long, default_value = "default", value_parser = clap::value_parser!(Device))]
    pub input_device: Device,
    #[arg(long, default_value = "96000")]
    pub input_sample_rate: u32,
    #[arg(long, default_value = "info")]
    pub log_level: tracing::Level,
    #[arg(long, default_value = "2000")]
    pub max_latency_in_samples: usize,
    #[arg(long, default_value = "1")]
    pub output_channels: u16,
    #[arg(long, default_value = "default", value_parser = clap::value_parser!(Device))]
    pub output_device: Device,
    #[arg(long, default_value = "96000")]
    pub output_sample_rate: u32,
    #[arg(long, default_value = "3000")]
    port: i32,
}

impl Settings {
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn audio_input_stream_config(&self) -> StreamConfig {
        StreamConfig {
            buffer_size: BufferSize::Fixed(self.buffer_size as u32),
            channels: self.input_channels,
            sample_rate: SampleRate(self.input_sample_rate),
        }
    }

    pub fn audio_output_stream_config(&self) -> StreamConfig {
        StreamConfig {
            buffer_size: BufferSize::Fixed(self.buffer_size as u32),
            channels: self.output_channels,
            sample_rate: SampleRate(self.output_sample_rate),
        }
    }
}
