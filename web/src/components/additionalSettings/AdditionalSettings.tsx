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
  return (
    <Stack spacing="1">
      <Typography component="h2" variant="h6">
        {t("additionalSettings.additionalSettings")}
      </Typography>
      {features.hasGamingMode && deviceState.gamingMode != null && (
        <FormControlLabel
          control={
            <Switch
              checked={deviceState.gamingMode}
              onChange={(event) => setGamingMode(event.target.checked)}
            />
          }
          label={t("additionalSettings.gamingMode")}
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
          label={t("additionalSettings.surroundSound")}
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
          label={t("additionalSettings.lowBatteryPrompt")}
        />
      )}
    </Stack>
  );
});
