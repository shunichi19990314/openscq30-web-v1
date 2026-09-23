use nom::{
    combinator::map,
    error::{context, ContextError, ParseError},
    number::complete::le_u8,
    sequence::tuple,
    IResult,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, FromRepr, IntoStaticStr};

use super::AmbientSoundMode;

/// Sound modes layout used by A3959 (Soundcore P30i / R50i NC), ported from the v2
/// implementation (`devices/soundcore/a3959/structures/sound_modes.rs`).
///
/// Wire format (7 bytes):
/// `[ambient, (manual << 4) | adaptive, ambient (duplicated), nc mode, wind, adaptive sensitivity, multi scene]`
///
/// Unlike type two, manual/adaptive noise canceling are raw 0-5 levels rather than enums, and a
/// multi-scene ANC selector is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SoundModesTypeThree {
    pub ambient_sound_mode: AmbientSoundMode,
    /// 0-5
    pub manual_noise_canceling: u8,
    /// 0-5, read-only on the device
    pub adaptive_noise_canceling: u8,
    pub noise_canceling_mode: NoiseCancelingModeTypeThree,
    pub wind_noise_suppression: bool,
    pub wind_noise_detected: bool,
    pub noise_canceling_adaptive_sensitivity_level: u8,
    pub multi_scene_noise_canceling: MultiSceneNoiseCanceling,
}

impl SoundModesTypeThree {
    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], SoundModesTypeThree, E> {
        context(
            "sound modes type three",
            map(
                tuple((
                    AmbientSoundMode::take,
                    le_u8,
                    AmbientSoundMode::take,
                    NoiseCancelingModeTypeThree::take,
                    le_u8,
                    le_u8,
                    MultiSceneNoiseCanceling::take,
                )),
                |(
                    ambient_sound_mode,
                    noise_canceling_settings,
                    _duplicated_ambient_sound_mode,
                    noise_canceling_mode,
                    wind_noise,
                    noise_canceling_adaptive_sensitivity_level,
                    multi_scene_noise_canceling,
                )| {
                    SoundModesTypeThree {
                        ambient_sound_mode,
                        manual_noise_canceling: (noise_canceling_settings & 0xF0) >> 4,
                        adaptive_noise_canceling: noise_canceling_settings & 0x0F,
                        noise_canceling_mode,
                        wind_noise_suppression: wind_noise & 0b01 != 0,
                        wind_noise_detected: wind_noise & 0b10 != 0,
                        noise_canceling_adaptive_sensitivity_level,
                        multi_scene_noise_canceling,
                    }
                },
            ),
        )(input)
    }

    pub(crate) fn bytes(&self) -> [u8; 7] {
        [
            self.ambient_sound_mode.id(),
            (self.manual_noise_canceling << 4) | (self.adaptive_noise_canceling & 0x0F),
            // the device repeats the ambient sound mode in this byte
            self.ambient_sound_mode.id(),
            self.noise_canceling_mode.id(),
            (u8::from(self.wind_noise_detected) << 1) | u8::from(self.wind_noise_suppression),
            self.noise_canceling_adaptive_sensitivity_level,
            self.multi_scene_noise_canceling.id(),
        ]
    }
}

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromRepr, Display, AsRefStr, IntoStaticStr,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum NoiseCancelingModeTypeThree {
    #[default]
    Manual = 0,
    Adaptive = 1,
    MultiScene = 2,
}

impl NoiseCancelingModeTypeThree {
    pub fn id(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "noise canceling mode type three",
            map(le_u8, |id| Self::from_repr(id).unwrap_or_default()),
        )(input)
    }
}

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromRepr, Display, AsRefStr, IntoStaticStr,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum MultiSceneNoiseCanceling {
    #[default]
    Transport = 0,
    Outdoor = 1,
    Indoor = 2,
}

impl MultiSceneNoiseCanceling {
    pub fn id(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "multi scene noise canceling",
            map(le_u8, |id| Self::from_repr(id).unwrap_or_default()),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_and_reserializes() {
        // bytes 64-70 of the a3959 state update packet from tools/soundcore-device-faker/devices/a3959.toml
        const INPUT: &[u8] = &[0, 0x55, 0, 0, 1, 255, 1];
        let (_, sound_modes) = SoundModesTypeThree::take::<nom::error::VerboseError<_>>(INPUT)
            .expect("should parse");
        assert_eq!(
            sound_modes,
            SoundModesTypeThree {
                ambient_sound_mode: AmbientSoundMode::NoiseCanceling,
                manual_noise_canceling: 5,
                adaptive_noise_canceling: 5,
                noise_canceling_mode: NoiseCancelingModeTypeThree::Manual,
                wind_noise_suppression: true,
                wind_noise_detected: false,
                noise_canceling_adaptive_sensitivity_level: 255,
                multi_scene_noise_canceling: MultiSceneNoiseCanceling::Outdoor,
            }
        );
        assert_eq!(INPUT, sound_modes.bytes().as_slice());
    }
}
