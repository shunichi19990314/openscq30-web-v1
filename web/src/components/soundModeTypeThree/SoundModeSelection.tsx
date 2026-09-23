import { FormControlLabel, Slider, Stack, Switch, Typography } from "@mui/material";
import React, { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useFilterNulls } from "../../hooks/useFilterNulls";
import { SoundModesTypeThree } from "../../libTypes/DeviceState";
import { AmbientSoundModeSelection } from "../soundModeTypeTwo/AmbientSoundModeSelection";
import {
  ToggleButtonRow,
  ToggleButtonValues,
} from "../soundModeTypeTwo/ToggleButtonRow";

interface Props {
  soundModes: SoundModesTypeThree;
  setSoundModes: (soundModes: SoundModesTypeThree) => void;
}

type TFunction = (key: string) => string;

function noiseCancelingModeValues(
  t: TFunction,
): ToggleButtonValues<SoundModesTypeThree["noiseCancelingMode"]> {
  return [
    { value: "manual", label: t("soundModeTypeThree.manual") },
    { value: "adaptive", label: t("soundModeTypeThree.adaptive") },
    { value: "multiScene", label: t("soundModeTypeThree.multiScene") },
  ];
}

function multiSceneValues(
  t: TFunction,
): ToggleButtonValues<SoundModesTypeThree["multiSceneNoiseCanceling"]> {
  return [
    {
      value: "transport",
      label: t("soundModeTypeThree.multiSceneOptions.transport"),
    },
    { value: "outdoor", label: t("soundModeTypeThree.multiSceneOptions.outdoor") },
    { value: "indoor", label: t("soundModeTypeThree.multiSceneOptions.indoor") },
  ];
}

function useSetter<K extends keyof SoundModesTypeThree>(
  key: K,
  soundModes: SoundModesTypeThree,
  setSoundModes: (soundModes: SoundModesTypeThree) => void,
) {
  return useCallback(
    (value: SoundModesTypeThree[K]) => {
      setSoundModes({
        ...soundModes,
        [key]: value,
      });
    },
    [key, setSoundModes, soundModes],
  );
}

/**
 * Sound mode selection for devices using the type three layout (A3959 / Soundcore P30i):
 * manual/adaptive noise canceling are 0-5 levels and a multi-scene ANC mode exists.
 */
export const SoundModeSelection = React.memo(function ({
  soundModes,
  setSoundModes,
}: Props) {
  const { t } = useTranslation();
  const setAmbientSoundMode = useSetter(
    "ambientSoundMode",
    soundModes,
    setSoundModes,
  );
  const setNoiseCancelingMode = useFilterNulls(
    useSetter("noiseCancelingMode", soundModes, setSoundModes),
  );
  const setManualNoiseCanceling = useSetter(
    "manualNoiseCanceling",
    soundModes,
    setSoundModes,
  );
  const setAdaptiveSensitivity = useSetter(
    "noiseCancelingAdaptiveSensitivityLevel",
    soundModes,
    setSoundModes,
  );
  const setMultiSceneNoiseCanceling = useFilterNulls(
    useSetter("multiSceneNoiseCanceling", soundModes, setSoundModes),
  );
  const setWindNoiseSuppression = useSetter(
    "windNoiseSuppression",
    soundModes,
    setSoundModes,
  );

  return (
    <Stack spacing="2">
      <Typography component="h2" variant="h6">
        {t("soundModes.soundModes")}
      </Typography>
      <AmbientSoundModeSelection
        value={soundModes.ambientSoundMode}
        hasNoiseCancelingMode={true}
        onValueChanged={setAmbientSoundMode}
      />
      <div>
        <Typography>{t("noiseCancelingMode.noiseCancelingMode")}</Typography>
        <ToggleButtonRow
          value={soundModes.noiseCancelingMode}
          onValueChanged={setNoiseCancelingMode}
          values={noiseCancelingModeValues(t)}
        />
      </div>
      {soundModes.noiseCancelingMode == "manual" && (
        <div>
          <Typography>
            {t("manualNoiseCanceling.manualNoiseCanceling")}:{" "}
            {Math.min(5, soundModes.manualNoiseCanceling)}/5
          </Typography>
          <Slider
            min={0}
            max={5}
            step={1}
            marks
            valueLabelDisplay="auto"
            value={Math.min(5, soundModes.manualNoiseCanceling)}
            onChange={(_, value) => setManualNoiseCanceling(value as number)}
          />
        </div>
      )}
      {soundModes.noiseCancelingMode == "adaptive" && (
        <Stack spacing="1">
          <div>
            <Typography>
              {t("adaptiveNoiseCanceling.adaptiveNoiseCanceling")}:{" "}
              {soundModes.adaptiveNoiseCanceling}/5
            </Typography>
          </div>
          <div>
            <Typography>
              {t("soundModeTypeThree.adaptiveSensitivity")}:{" "}
              {Math.min(10, soundModes.noiseCancelingAdaptiveSensitivityLevel)}/10
            </Typography>
            <Slider
              min={0}
              max={10}
              step={1}
              marks
              valueLabelDisplay="auto"
              value={Math.min(10, soundModes.noiseCancelingAdaptiveSensitivityLevel)}
              onChange={(_, value) => setAdaptiveSensitivity(value as number)}
            />
          </div>
        </Stack>
      )}
      {soundModes.noiseCancelingMode == "multiScene" && (
        <div>
          <Typography>
            {t("soundModeTypeThree.multiSceneNoiseCanceling")}
          </Typography>
          <ToggleButtonRow
            value={soundModes.multiSceneNoiseCanceling}
            onValueChanged={setMultiSceneNoiseCanceling}
            values={multiSceneValues(t)}
          />
        </div>
      )}
      <FormControlLabel
        control={
          <Switch
            checked={soundModes.windNoiseSuppression}
            onChange={(event) =>
              setWindNoiseSuppression(event.target.checked)
            }
          />
        }
        label={t("soundModeTypeThree.windNoiseSuppression")}
      />
    </Stack>
  );
});
