use crate::managers::audio_device_settings::constants::{
    HIFIBERRY_CARD_NAME, HIFIBERRY_INPUT_GAIN_CONTROL_NAME,
    HIFIBERRY_INPUT_SELECTOR_UNBALANCED_VALUE, HIFIBERRY_LEFT_INPUT_SELECTOR_CONTROL_NAME,
    HIFIBERRY_MIC_BIAS_CONTROL_NAME, HIFIBERRY_OUTPUT_CONTROL_NAME,
    HIFIBERRY_RIGHT_INPUT_SELECTOR_CONTROL_NAME,
};
use crate::managers::audio_device_settings::error::Error;
use crate::managers::audio_device_settings::error::Error::CannotFindControl;
use alsa::mixer::{Selem, SelemChannelId, SelemId};
use alsa::{card, Mixer};

mod constants;
pub mod error;

pub struct AudioDeviceSettingsManager {
    mixer: Mixer,
}

unsafe impl Sync for AudioDeviceSettingsManager {}

impl AudioDeviceSettingsManager {
    pub fn new() -> Result<Self, Error> {
        let card_name = card::Iter::new()
            .find(|any_card| {
                any_card
                    .ok()
                    .and_then(|card| {
                        card.get_longname()
                            .ok()
                            .map(|card_name| card_name.to_lowercase().contains(HIFIBERRY_CARD_NAME))
                    })
                    .unwrap_or(false)
            })
            .and_then(|card| card.ok().and_then(|card| card.get_name().ok()))
            .map(Ok)
            .unwrap_or(Err(Error::CardNotFound))?;

        let mixer = Mixer::new(card_name.as_str(), true).map_err(|_| Error::CannotOpenMixer)?;
        let left_input_selector_control =
            find_control(&mixer, HIFIBERRY_LEFT_INPUT_SELECTOR_CONTROL_NAME)?;
        let right_input_selector_control =
            find_control(&mixer, HIFIBERRY_RIGHT_INPUT_SELECTOR_CONTROL_NAME)?;

        left_input_selector_control
            .set_enum_item(
                SelemChannelId::mono(),
                HIFIBERRY_INPUT_SELECTOR_UNBALANCED_VALUE,
            )
            .map_err(|_| Error::CannotSetInputMode)?;
        right_input_selector_control
            .set_enum_item(
                SelemChannelId::mono(),
                HIFIBERRY_INPUT_SELECTOR_UNBALANCED_VALUE,
            )
            .map_err(|_| Error::CannotSetInputMode)?;

        Ok(AudioDeviceSettingsManager { mixer })
    }

    pub fn get_input_gain(&self) -> Result<f32, Error> {
        let input_gain_control = find_control(&self.mixer, HIFIBERRY_INPUT_GAIN_CONTROL_NAME)?;

        let (min_value, max_value) = input_gain_control.get_capture_volume_range();
        let value = input_gain_control.get_capture_volume(SelemChannelId::mono())?;

        Ok((value - min_value) as f32 / (max_value - min_value) as f32)
    }

    pub fn set_input_gain(&self, value: f32) -> Result<(), Error> {
        let input_gain_control = find_control(&self.mixer, HIFIBERRY_INPUT_GAIN_CONTROL_NAME)?;

        let (min_value, max_value) = input_gain_control.get_capture_volume_range();
        Ok(input_gain_control.set_capture_volume(
            SelemChannelId::mono(),
            ((max_value - min_value) as f32 * value) as i64 + min_value,
        )?)
    }

    pub fn get_volume(&self) -> Result<f32, Error> {
        let output_control = find_control(&self.mixer, HIFIBERRY_OUTPUT_CONTROL_NAME)?;

        let (min_value, max_value) = output_control.get_playback_volume_range();
        let value = output_control.get_playback_volume(SelemChannelId::mono())?;

        Ok((value - min_value) as f32 / (max_value - min_value) as f32)
    }

    pub fn set_volume(&self, value: f32) -> Result<(), Error> {
        let output_control = find_control(&self.mixer, HIFIBERRY_OUTPUT_CONTROL_NAME)?;

        let (min_value, max_value) = output_control.get_playback_volume_range();
        output_control.set_playback_volume(
            SelemChannelId::mono(),
            ((max_value - min_value) as f32 * value) as i64 + min_value,
        )?;
        Ok(())
    }

    pub fn is_mic_bias_on(&self) -> Result<bool, Error> {
        let mic_bias_control = find_control(&self.mixer, HIFIBERRY_MIC_BIAS_CONTROL_NAME)?;

        Ok(mic_bias_control.get_enum_item(SelemChannelId::mono())? != 0)
    }

    pub fn set_mic_bias(&self, on: bool) -> Result<(), Error> {
        let mic_bias_control = find_control(&self.mixer, HIFIBERRY_MIC_BIAS_CONTROL_NAME)?;

        mic_bias_control.set_enum_item(SelemChannelId::mono(), if on { 1 } else { 0 })?;
        Ok(())
    }
}

fn find_control<'a>(mixer: &'a Mixer, control_name: &'a str) -> Result<Selem<'a>, Error> {
    mixer
        .find_selem(&SelemId::new(control_name, 0))
        .map(Ok)
        .unwrap_or(Err(CannotFindControl))
}
