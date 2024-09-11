use crate::managers::dsp::error::{Error, TransformProcessorError};
use crate::managers::dsp::models::{
    Device, Driver, ProcessorInfo, ProcessorParameters, ProcessorSettings,
};
use crate::processors::frontline::BoxedProcessor;
use crate::settings::Settings;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, host_from_id, Stream};
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use ringbuf::traits::Split;
use ringbuf::HeapRb;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub mod error;
pub mod models;

type ProcessorsVector = Arc<Mutex<Vec<BoxedProcessor>>>;
type ProcessorSettingsVector = Arc<Mutex<Vec<ProcessorSettings>>>;
type CpuUsage = Arc<Mutex<Option<f32>>>;

pub struct DSPManager {
    pub buffer_size: usize,
    cpu_usage: CpuUsage,
    input_stream: Stream,
    output_stream: Stream,
    processors: ProcessorsVector,
    processors_settings: ProcessorSettingsVector,
    pub sample_rate: u32,
}

unsafe impl Send for DSPManager {}

impl DSPManager {
    pub fn new(settings: &Settings) -> Result<Self, Error> {
        let host = match &settings.driver {
            Driver::Default => Ok(default_host()),
            other => host_from_id(other.host_id()),
        }?;

        let input_device = get_device(
            host.default_input_device(),
            host.input_devices()?.collect(),
            &settings.input_device,
        )?;
        let output_device = get_device(
            host.default_output_device(),
            host.output_devices()?.collect(),
            &settings.output_device,
        )?;

        let input_config = settings.audio_input_stream_config();
        let output_config = settings.audio_output_stream_config();
        let total_buffer_size = settings.max_latency_in_samples + settings.buffer_size;
        let buffer = HeapRb::<f32>::new(total_buffer_size);
        let (mut producer, mut consumer) = buffer.split();
        let mut input_samples_fell_behind = false;
        let mut output_samples_fell_behind = false;

        let processors: ProcessorsVector = Arc::new(Mutex::new(vec![]));
        let consumed_processors = processors.clone();
        let processors_settings: ProcessorSettingsVector = Arc::new(Mutex::new(vec![]));
        let consumed_processors_settings = processors_settings.clone();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for &sample in data {
                if producer.try_push(sample).is_err() && !output_samples_fell_behind {
                    output_samples_fell_behind = true;
                    tracing::error!("output stream fell behind: try increasing latency");
                }
            }
        };

        let mut cpu_usage_monitor = SystemTime::now();
        let cpu_usage: CpuUsage = Arc::new(Mutex::new(None));
        let cpu_usage_producer = cpu_usage.clone();
        let mut data_vector = vec![0.0; settings.buffer_size];

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // This truncates or extends the buffer without reallocating memory
            data_vector.resize(data.len(), 0.0);
            for sample in data_vector.iter_mut() {
                *sample = match consumer.try_pop() {
                    Some(extracted_sample) => extracted_sample,
                    None => {
                        if !input_samples_fell_behind {
                            input_samples_fell_behind = true;
                            tracing::error!("input stream fell behind: try increasing latency");
                        }
                        0.0
                    }
                };
            }

            let cpu_idle_time = cpu_usage_monitor.elapsed();
            cpu_usage_monitor = SystemTime::now();
            if let Ok(unwrapped_processors_settings) = consumed_processors_settings.try_lock() {
                if let Ok(mut unwrapped_processors) = consumed_processors.try_lock() {
                    for (processor, unwrapped_processor_settings) in unwrapped_processors
                        .iter_mut()
                        .zip(unwrapped_processors_settings.iter())
                    {
                        if !unwrapped_processor_settings.bypassed {
                            processor.process(&mut data_vector);
                        }
                    }
                }
            }

            for (processed_sample, output_sample) in data_vector.iter_mut().zip(data) {
                *output_sample = *processed_sample;
            }

            cpu_usage_producer
                .try_lock()
                .map(|mut cpu_usage_producer_unlocked| {
                    *cpu_usage_producer_unlocked = cpu_usage_monitor
                        .elapsed()
                        .and_then(|usage_time| {
                            cpu_idle_time.map(|idle_time| {
                                usage_time.as_millis() as f32
                                    / usage_time.add(idle_time).as_millis() as f32
                            })
                        })
                        .map_err(|err| tracing::error!("error measuring cpu audio time: {}", err))
                        .ok();
                })
                .ok();
        };

        let input_stream =
            input_device.build_input_stream(&input_config, input_data_fn, handle_error, None)?;
        input_stream.pause()?;
        let output_stream = output_device.build_output_stream(
            &output_config,
            output_data_fn,
            handle_error,
            None,
        )?;
        output_stream.pause()?;

        tracing::info!("audio host selected: {}", host.id().name());
        tracing::info!("buffer size: {}", settings.buffer_size);
        tracing::info!("input sample rate: {}", settings.input_sample_rate);
        tracing::info!("output sample rate: {}", settings.output_sample_rate);
        tracing::info!(
            "input audio device selected: {} with {} channels",
            input_device.name()?,
            input_config.channels
        );
        tracing::info!(
            "output audio device selected: {} with {} channels",
            output_device.name()?,
            output_config.channels
        );

        Ok(Self {
            buffer_size: settings.buffer_size,
            cpu_usage,
            input_stream,
            output_stream,
            processors,
            processors_settings,
            sample_rate: settings.input_sample_rate,
        })
    }

    pub fn start(&self) -> Result<(), Error> {
        self.input_stream.play()?;
        self.output_stream.play()?;
        Ok(())
    }

    pub fn get_audio_cpu_usage(&self) -> Option<f32> {
        self.cpu_usage.try_lock().ok().and_then(|value| *value)
    }

    pub fn add_processor(&mut self, index: usize, processor: BoxedProcessor) {
        self.processors.lock().unwrap().insert(index, processor);
        self.processors_settings
            .lock()
            .unwrap()
            .insert(index, ProcessorSettings { bypassed: false });
    }

    pub fn transform_processor<Function>(
        &self,
        index: usize,
        transform_function: Function,
    ) -> Result<(), TransformProcessorError>
    where
        Function: FnOnce(&mut BoxedProcessor),
    {
        match self.processors.lock().unwrap().get_mut(index) {
            Some(processor) => {
                transform_function(processor);
                Ok(())
            }
            None => Err(TransformProcessorError::NotFound),
        }
    }

    pub fn transform_processor_settings<Function>(
        &self,
        index: usize,
        transform_function: Function,
    ) -> Result<(), TransformProcessorError>
    where
        Function: FnOnce(&mut ProcessorSettings),
    {
        match self.processors_settings.lock().unwrap().get_mut(index) {
            Some(processor_settings) => {
                transform_function(processor_settings);
                Ok(())
            }
            None => Err(TransformProcessorError::NotFound),
        }
    }

    pub fn extract_processor(&mut self, index: usize) -> BoxedProcessor {
        self.processors_settings.lock().unwrap().remove(index);
        self.processors.lock().unwrap().remove(index)
    }

    pub fn clear_all_processors(&self) {
        self.processors_settings.lock().unwrap().clear();
        self.processors.lock().unwrap().clear();
    }

    pub fn get_processors_info(&self) -> Vec<ProcessorInfo> {
        self.processors_settings
            .lock()
            .unwrap()
            .iter()
            .zip(self.processors.lock().unwrap().iter())
            .map(|(settings, processor)| ProcessorInfo {
                settings: (*settings).clone(),
                parameters: ProcessorParameters {
                    numeric: processor.get_numeric_parameters(),
                    string: processor.get_string_parameters(),
                },
                processor_type: processor.get_type(),
            })
            .collect()
    }
}

fn get_device(
    default_device: Option<cpal::Device>,
    devices: Vec<cpal::Device>,
    name: &Device,
) -> Result<cpal::Device, Error> {
    match name {
        Device::Default => default_device.ok_or(Error::DefaultDeviceNotFound),
        Device::Named(device_name) => devices
            .into_iter()
            .find(|candidate_device| {
                candidate_device
                    .name()
                    .map_or(false, |candidate_device_name| {
                        candidate_device_name == *device_name
                    })
            })
            .ok_or(Error::NamedDeviceNotFound),
    }
}

fn handle_error(error: cpal::StreamError) {
    tracing::error!("audio processing error: {}", error);
}
