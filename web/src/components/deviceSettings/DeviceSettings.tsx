import { Masonry } from "@mui/lab";
import { Paper } from "@mui/material";
import { Dispatch, SetStateAction, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { EqualizerHelper } from "../../../wasm/pkg/openscq30_web_wasm";
import { Device } from "../../bluetooth/Device";
import { useToastErrorHandler } from "../../hooks/useToastErrorHandler";
import {
  MultiButtonConfiguration,
  DeviceState,
  EqualizerConfiguration,
  PresetEqualizerProfile,
  SoundModes,
  SoundModesTypeTwo,
  SoundModesTypeThree,
} from "../../libTypes/DeviceState";
import { SoundModeSelection as SoundModeTypeTwoSelection } from "../soundModeTypeTwo/SoundModeSelection";
import { SoundModeSelection as SoundModeTypeThreeSelection } from "../soundModeTypeThree/SoundModeSelection";
import { AdditionalSettings } from "../additionalSettings/AdditionalSettings";
import { ButtonSettings } from "../buttonSettings/ButtonSettings";
import { DeviceInfo } from "../deviceInfo/DeviceInfo";
import { EqualizerSettings } from "../equalizer/EqualizerSettings";
import { NewCustomProfileDialog } from "../equalizer/NewCustomProfileDialog";
import { ImportExport } from "../importExport/ImportExport";
import { SoundModeSelection } from "./SoundModeSelection";
import { useCreateCustomProfileWithName } from "./hooks/useCreateCustomProfileWithName";
import { useCustomEqualizerProfiles } from "./hooks/useCustomEqualizerProfiles";
import { useDeleteCustomProfile } from "./hooks/useDeleteCustomProfile";
import { useDisplayState } from "./hooks/useDisplayState";

export function DeviceSettings({
  device,
  disconnect,
}: {
  device: Device;
  disconnect: () => void;
}) {
  const { t } = useTranslation();
  const errorHandler = useToastErrorHandler(t("errors.disconnected"));
  const onBluetoothError = useCallback(
    (err: Error) => {
      errorHandler(err);
      disconnect();
    },
    [errorHandler, disconnect],
  );

  const [displayState, setDisplayState] = useDisplayState(
    device,
    onBluetoothError,
  );

  return (
    <Masonry columns={{ sx: 1, lg: 2 }}>
      {[
        // Regular function call so we can filter out nulls
        SoundModeSelectionSection({ displayState, setDisplayState }),
        SoundModeTypeTwoSelectionSection({ displayState, setDisplayState }),
        SoundModeTypeThreeSelectionSection({ displayState, setDisplayState }),
        AdditionalSettingsSection({ device, displayState }),
        EqualizerSection({ displayState, setDisplayState }),
        ButtonSettingsSection({ displayState, setDisplayState }),

        <DeviceInfo deviceState={displayState} />,
        <ImportExport />,
      ]
        .filter((component) => component)
        .map((component, index) => (
          <Paper
            elevation={0}
            key={index}
            sx={{
              padding: { xs: 2, sm: 3 },
              marginBottom: 3,
              borderRadius: 6,
              border: "1px solid",
              borderColor: "divider",
              boxShadow: "0 2px 12px rgb(0 0 0 / 0.06)",
            }}
          >
            {component}
          </Paper>
        ))}
    </Masonry>
  );
}

function SoundModeSelectionSection({
  displayState,
  setDisplayState,
}: {
  displayState: DeviceState;
  setDisplayState: Dispatch<SetStateAction<DeviceState>>;
}) {
  const setSoundModes = useCallback(
    (soundModes: SoundModes) => {
      setDisplayState((state) => ({
        ...state,
        soundModes: soundModes,
      }));
    },
    [setDisplayState],
  );

  if (
    displayState.deviceFeatures.availableSoundModes != null &&
    displayState.soundModes
  ) {
    return (
      <SoundModeSelection
        soundModes={displayState.soundModes}
        setSoundModes={setSoundModes}
        availableModes={displayState.deviceFeatures.availableSoundModes}
      />
    );
  }
}

function SoundModeTypeTwoSelectionSection({
  displayState,
  setDisplayState,
}: {
  displayState: DeviceState;
  setDisplayState: Dispatch<SetStateAction<DeviceState>>;
}) {
  const setSoundModes = useCallback(
    (soundModes: SoundModesTypeTwo) => {
      setDisplayState((state) => ({
        ...state,
        soundModesTypeTwo: soundModes,
      }));
    },
    [setDisplayState],
  );

  if (displayState.soundModesTypeTwo) {
    return (
      <SoundModeTypeTwoSelection
        soundModes={displayState.soundModesTypeTwo}
        setSoundModes={setSoundModes}
      />
    );
  }
}

function SoundModeTypeThreeSelectionSection({
  displayState,
  setDisplayState,
}: {
  displayState: DeviceState;
  setDisplayState: Dispatch<SetStateAction<DeviceState>>;
}) {
  const setSoundModes = useCallback(
    (soundModes: SoundModesTypeThree) => {
      setDisplayState((state) => ({
        ...state,
        soundModesTypeThree: soundModes,
      }));
    },
    [setDisplayState],
  );

  if (displayState.soundModesTypeThree) {
    return (
      <SoundModeTypeThreeSelection
        soundModes={displayState.soundModesTypeThree}
        setSoundModes={setSoundModes}
      />
    );
  }
}

function AdditionalSettingsSection({
  device,
  displayState,
}: {
  device: Device;
  displayState: DeviceState;
}) {
  if (
    !displayState.deviceFeatures.hasGamingMode &&
    !displayState.deviceFeatures.hasSurroundSound &&
    !displayState.deviceFeatures.hasLowBatteryPrompt
  ) {
    return undefined;
  }
  return <AdditionalSettings device={device} deviceState={displayState} />;
}

function EqualizerSection({
  displayState,
  setDisplayState,
}: {
  displayState: DeviceState;
  setDisplayState: Dispatch<SetStateAction<DeviceState>>;
}) {
  const setSelectedPresetProfile = useCallback(
    (presetProfile: PresetEqualizerProfile | "custom") => {
      const newEqualizerConfiguration: EqualizerConfiguration =
        presetProfile != "custom"
          ? {
              presetProfile,
              volumeAdjustments: [
                ...EqualizerHelper.getPresetProfileVolumeAdjustments(
                  presetProfile,
                ),
              ],
            }
          : {
              presetProfile: null,
              volumeAdjustments:
                displayState.equalizerConfiguration.volumeAdjustments,
            };
      setDisplayState((state) => ({
        ...state,
        equalizerConfiguration: newEqualizerConfiguration,
      }));
    },
    [displayState.equalizerConfiguration.volumeAdjustments, setDisplayState],
  );

  const setEqualizerValue = useCallback(
    (changedIndex: number, newVolume: number) => {
      setDisplayState((state) => {
        const volumeAdjustments =
          state.equalizerConfiguration.volumeAdjustments.map((volume, index) =>
            index == changedIndex ? newVolume : volume,
          );
        return {
          ...state,
          equalizerConfiguration: {
            presetProfile: null,
            volumeAdjustments: volumeAdjustments,
          },
        };
      });
    },
    [setDisplayState],
  );

  const openCreateCustomProfileDialog = useCallback(
    () => setCreateCustomProfileDialogOpen(true),
    [],
  );

  const [isCreateCustomProfileDialogOpen, setCreateCustomProfileDialogOpen] =
    useState(false);
  const customEqualizerProfiles = useCustomEqualizerProfiles();

  const closeCreateCustomProfileDialog = useCallback(
    () => setCreateCustomProfileDialogOpen(false),
    [setCreateCustomProfileDialogOpen],
  );

  const createCustomProfileWithName = useCreateCustomProfileWithName(
    displayState.equalizerConfiguration.volumeAdjustments,
  );

  const deleteCustomProfile = useDeleteCustomProfile();
  if (displayState.deviceFeatures.numEqualizerBands > 0) {
    return (
      <>
        <EqualizerSettings
          profile={
            displayState.equalizerConfiguration.presetProfile ?? "custom"
          }
          onProfileSelected={setSelectedPresetProfile}
          values={displayState.equalizerConfiguration.volumeAdjustments}
          onValueChange={setEqualizerValue}
          customProfiles={customEqualizerProfiles}
          onAddCustomProfile={openCreateCustomProfileDialog}
          onDeleteCustomProfile={deleteCustomProfile}
        />
        <NewCustomProfileDialog
          isOpen={isCreateCustomProfileDialogOpen}
          existingProfiles={customEqualizerProfiles}
          onClose={closeCreateCustomProfileDialog}
          onCreate={createCustomProfileWithName}
        />
      </>
    );
  }
}

function ButtonSettingsSection({
  displayState,
  setDisplayState,
}: {
  displayState: DeviceState;
  setDisplayState: Dispatch<SetStateAction<DeviceState>>;
}) {
  const setMultiButtonConfiguration = useCallback(
    (buttonConfiguration: MultiButtonConfiguration) => {
      setDisplayState((state) => {
        return {
          ...state,
          buttonConfiguration,
        };
      });
    },
    [setDisplayState],
  );
  if (
    displayState.deviceFeatures.hasButtonConfiguration &&
    displayState.buttonConfiguration != null
  ) {
    return (
      <ButtonSettings
        buttonConfiguration={displayState.buttonConfiguration}
        setMultiButtonConfiguration={setMultiButtonConfiguration}
      />
    );
  }
}
