import { Tune } from "@mui/icons-material";
import { FormControlLabel, Stack, Switch, Typography } from "@mui/material";
import React, { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Device } from "../../bluetooth/Device";
import { useToastErrorHandler } from "../../hooks/useToastErrorHandler";
import { DeviceState } from "../../libTypes/DeviceState";

interface Props {
  device: Device;
  deviceState: DeviceState;
}

/**
 * Miscellaneous single flag settings of newer devices (A3959 / Soundcore P30i):
 * gaming mode, surround sound and the low battery prompt.
 */
export const AdditionalSettings = React.memo(function ({
  device,
  deviceState,
}: Props) {
  const { t } = useTranslation();
  const errorHandler = useToastErrorHandler(t("errors.disconnected"));

  const setGamingMode = useCallback(
    (enabled: boolean) => {
      device.setGamingMode(enabled).catch(errorHandler);
    },
    [device, errorHandler],
  );
  const setSurroundSound = useCallback(
    (enabled: boolean) => {
      device.setSurroundSound(enabled).catch(errorHandler);
    },
    [device, errorHandler],
  );
  const setLowBatteryPrompt = useCallback(
    (enabled: boolean) => {
      device.setLowBatteryPrompt(enabled).catch(errorHandler);
    },
    [device, errorHandler],
  );

  const features = deviceState.deviceFeatures;
  // v2 only exposes gaming mode on firmware >= 01.60; older firmwares ignore or
  // misinterpret the flag, so hide the toggle there
  const firmware = deviceState.firmwareVersion;
  const gamingModeSupported =
    firmware != null && (firmware.major > 1 || firmware.minor >= 60);
  return (
    <Stack spacing="1">
      <Typography component="h2" variant="h6" sx={{ display: "flex", alignItems: "center", gap: 1, marginBottom: 1 }}>
        <Tune color="primary" />
        {t("additionalSettings.additionalSettings")}
      </Typography>
      {features.hasGamingMode && gamingModeSupported && deviceState.gamingMode != null && (
        <FormControlLabel
          control={
            <Switch
              checked={deviceState.gamingMode}
              onChange={(event) => setGamingMode(event.target.checked)}
            />
          }
          label={
            <Stack spacing={0.25}>
              <Typography>{t("additionalSettings.gamingMode")}</Typography>
              <Typography variant="caption" color="text.secondary">{t("additionalSettings.gamingModeDescription")}</Typography>
            </Stack>
          }
        />
      )}
      {features.hasSurroundSound && deviceState.surroundSound != null && (
        <FormControlLabel
          control={
            <Switch
              checked={deviceState.surroundSound}
              onChange={(event) => setSurroundSound(event.target.checked)}
            />
          }
          label={
            <Stack spacing={0.25}>
              <Typography>{t("additionalSettings.surroundSound")}</Typography>
              <Typography variant="caption" color="text.secondary">{t("additionalSettings.surroundSoundDescription")}</Typography>
            </Stack>
          }
        />
      )}
      {features.hasLowBatteryPrompt && deviceState.lowBatteryPrompt != null && (
        <FormControlLabel
          control={
            <Switch
              checked={deviceState.lowBatteryPrompt}
              onChange={(event) => setLowBatteryPrompt(event.target.checked)}
            />
          }
          label={
            <Stack spacing={0.25}>
              <Typography>{t("additionalSettings.lowBatteryPrompt")}</Typography>
              <Typography variant="caption" color="text.secondary">{t("additionalSettings.lowBatteryPromptDescription")}</Typography>
            </Stack>
          }
        />
      )}
    </Stack>
  );
});
