import {
  ArrowBack,
  BatteryChargingFull,
  BatteryFull,
  ChevronRight,
  GraphicEq,
  Hearing,
  InfoOutlined,
  SyncAlt,
  TouchApp,
  Tune,
} from "@mui/icons-material";
import {
  Box,
  IconButton,
  Paper,
  Stack,
  Typography,
} from "@mui/material";
import { useCallback, useState } from "react";
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

type Screen =
  | "home"
  | "sound"
  | "eq"
  | "controls"
  | "settings"
  | "info"
  | "profiles";

/**
 * Companion app style navigation: a home screen with a device status header and
 * section cards, and one screen per section. Original design (not a copy of any
 * vendor app), reusing the existing setting components.
 */
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
      // a rejected feature (e.g. band count mismatch) is not a connection loss;
      // dropping the session for it would look like the headphones disconnected
      if (String(err).includes("FeatureNotSupported")) {
        return;
      }
      disconnect();
    },
    [errorHandler, disconnect],
  );

  const [displayState, setDisplayState] = useDisplayState(
    device,
    onBluetoothError,
  );
  const [screen, setScreen] = useState<Screen>("home");

  const [isCreateCustomProfileDialogOpen, setCreateCustomProfileDialogOpen] =
    useState(false);
  const customEqualizerProfiles = useCustomEqualizerProfiles();
  const openCreateCustomProfileDialog = useCallback(
    () => setCreateCustomProfileDialogOpen(true),
    [],
  );
  const closeCreateCustomProfileDialog = useCallback(
    () => setCreateCustomProfileDialogOpen(false),
    [],
  );
  const createCustomProfile = useCreateCustomProfileWithName([
    ...displayState.equalizerConfiguration.volumeAdjustments,
  ]);
  const deleteCustomProfile = useDeleteCustomProfile();

  const setEqualizerValue = useCallback(
    (changedIndex: number, newVolume: number) => {
      setDisplayState((state) => {
        const volumeAdjustments =
          state.equalizerConfiguration.volumeAdjustments.map(
            (volume, index) => (index == changedIndex ? newVolume : volume),
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

  const setSelectedPresetProfile = useCallback(
    (presetProfile: PresetEqualizerProfile | "custom") => {
      // preset curves are stored with 8 bands; devices with more bands (A3959 has 10)
      // need the curve padded, otherwise the band count check rejects the update
      const adjustments =
        presetProfile != "custom"
          ? [...EqualizerHelper.getPresetProfileVolumeAdjustments(presetProfile)]
          : displayState.equalizerConfiguration.volumeAdjustments;
      const targetBands = displayState.deviceFeatures.numEqualizerBands;
      while (adjustments.length < targetBands) {
        adjustments.push(0);
      }
      adjustments.length = Math.min(adjustments.length, targetBands);
      const newEqualizerConfiguration: EqualizerConfiguration =
        presetProfile != "custom"
          ? {
              presetProfile,
              volumeAdjustments: adjustments,
            }
          : {
              presetProfile: null,
              volumeAdjustments: adjustments,
            };
      setDisplayState((state) => ({
        ...state,
        equalizerConfiguration: newEqualizerConfiguration,
      }));
    },
    [
      displayState.equalizerConfiguration.volumeAdjustments,
      displayState.deviceFeatures.numEqualizerBands,
      setDisplayState,
    ],
  );

  const setSoundModes = useCallback(
    (soundModes: SoundModes) => {
      setDisplayState((state) => ({ ...state, soundModes }));
    },
    [setDisplayState],
  );
  const setSoundModesTypeTwo = useCallback(
    (soundModes: SoundModesTypeTwo) => {
      setDisplayState((state) => ({ ...state, soundModesTypeTwo: soundModes }));
    },
    [setDisplayState],
  );
  const setSoundModesTypeThree = useCallback(
    (soundModes: SoundModesTypeThree) => {
      setDisplayState((state) => ({
        ...state,
        soundModesTypeThree: soundModes,
      }));
    },
    [setDisplayState],
  );
  const setMultiButtonConfiguration = useCallback(
    (buttons: MultiButtonConfiguration) => {
      setDisplayState((state) => ({ ...state, buttonConfiguration: buttons }));
    },
    [setDisplayState],
  );

  const features = displayState.deviceFeatures;
  const hasSoundModes =
    displayState.soundModes != null ||
    displayState.soundModesTypeTwo != null ||
    displayState.soundModesTypeThree != null;
  const hasEqualizer = features.numEqualizerBands > 0;
  const hasControls =
    features.hasButtonConfiguration && displayState.buttonConfiguration != null;
  const hasAdditional =
    features.hasGamingMode ||
    features.hasSurroundSound ||
    features.hasLowBatteryPrompt;

  const currentSoundModes =
    displayState.soundModes ??
    displayState.soundModesTypeTwo ??
    displayState.soundModesTypeThree;
  const presetProfile = displayState.equalizerConfiguration.presetProfile;

  const cards: {
    screen: Screen;
    icon: React.ReactNode;
    title: string;
    subtitle?: string;
  }[] = [];
  if (hasSoundModes) {
    cards.push({
      screen: "sound",
      icon: <Hearing />,
      title: t("soundModes.soundModes"),
      subtitle: currentSoundModes
        ? t(`ambientSoundMode.${currentSoundModes.ambientSoundMode}`)
        : undefined,
    });
  }
  if (hasEqualizer) {
    cards.push({
      screen: "eq",
      icon: <GraphicEq />,
      title: t("equalizer.equalizer"),
      subtitle:
        presetProfile != null
          ? t(
              `presetEqualizerProfile.${presetProfile.charAt(0).toLowerCase()}${presetProfile.slice(1)}`,
            )
          : t("equalizer.custom"),
    });
  }
  if (hasControls) {
    cards.push({
      screen: "controls",
      icon: <TouchApp />,
      title: t("buttons.Buttons"),
    });
  }
  if (hasAdditional) {
    cards.push({
      screen: "settings",
      icon: <Tune />,
      title: t("additionalSettings.additionalSettings"),
    });
  }
  cards.push({ screen: "info", icon: <InfoOutlined />, title: t("deviceInfo.deviceInfo") });
  cards.push({ screen: "profiles", icon: <SyncAlt />, title: t("application.importExport") });

  return (
    <Box sx={{ maxWidth: 800, marginX: "auto" }}>
      {screen == "home" ? (
        <Stack spacing={2}>
          <DeviceHeaderCard deviceState={displayState} />
          <Stack spacing={1.5}>
            {cards.map((card) => (
              <NavCard
                key={card.screen}
                icon={card.icon}
                title={card.title}
                subtitle={card.subtitle}
                onClick={() => setScreen(card.screen)}
              />
            ))}
          </Stack>
        </Stack>
      ) : (
        <Stack spacing={2}>
          <IconButton
            onClick={() => setScreen("home")}
            aria-label={t("application.back").toString()}
            sx={{ alignSelf: "flex-start" }}
          >
            <ArrowBack />
          </IconButton>
          {screen == "sound" && (
            <>
              {displayState.soundModes &&
                displayState.deviceFeatures.availableSoundModes && (
                  <SoundModeSelection
                    soundModes={displayState.soundModes}
                    setSoundModes={setSoundModes}
                    availableModes={
                      displayState.deviceFeatures.availableSoundModes
                    }
                  />
                )}
              {displayState.soundModesTypeTwo && (
                <SoundModeTypeTwoSelection
                  soundModes={displayState.soundModesTypeTwo}
                  setSoundModes={setSoundModesTypeTwo}
                />
              )}
              {displayState.soundModesTypeThree && (
                <SoundModeTypeThreeSelection
                  soundModes={displayState.soundModesTypeThree}
                  setSoundModes={setSoundModesTypeThree}
                />
              )}
            </>
          )}
          {screen == "eq" && (
            <EqualizerSettings
              profile={presetProfile ?? "custom"}
              customProfiles={customEqualizerProfiles}
              onProfileSelected={setSelectedPresetProfile}
              values={[
                ...displayState.equalizerConfiguration.volumeAdjustments,
              ]}
              onValueChange={setEqualizerValue}
              onAddCustomProfile={openCreateCustomProfileDialog}
              onDeleteCustomProfile={deleteCustomProfile}
            />
          )}
          {screen == "controls" && displayState.buttonConfiguration && (
            <ButtonSettings
              buttonConfiguration={displayState.buttonConfiguration}
              setMultiButtonConfiguration={setMultiButtonConfiguration}
            />
          )}
          {screen == "settings" && (
            <AdditionalSettings device={device} deviceState={displayState} />
          )}
          {screen == "info" && <DeviceInfo deviceState={displayState} />}
          {screen == "profiles" && <ImportExport />}
        </Stack>
      )}
      <NewCustomProfileDialog
        isOpen={isCreateCustomProfileDialogOpen}
        existingProfiles={customEqualizerProfiles}
        onClose={closeCreateCustomProfileDialog}
        onCreate={createCustomProfile}
      />
    </Box>
  );
}

function DeviceHeaderCard({ deviceState }: { deviceState: DeviceState }) {
  const { t } = useTranslation();
  const batteries =
    deviceState.battery.type == "dualBattery"
      ? [
          { label: t("battery.left"), battery: deviceState.battery.left },
          { label: t("battery.right"), battery: deviceState.battery.right },
        ]
      : [{ label: t("battery.level"), battery: deviceState.battery }];
  return (
    <Paper
      elevation={0}
      sx={{
        padding: 2.5,
        borderRadius: 6,
        border: "1px solid",
        borderColor: "divider",
        background: "linear-gradient(135deg, rgba(71,78,61,0.16), rgba(71,78,61,0.02))",
      }}
    >
      <Stack spacing={1.5}>
        <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
          {batteries.map(({ label, battery }) => (
            <Paper
              key={label}
              elevation={0}
              sx={{
                display: "flex",
                alignItems: "center",
                gap: 0.75,
                borderRadius: 999,
                paddingX: 1.5,
                paddingY: 0.5,
                border: "1px solid",
                borderColor: "divider",
              }}
            >
              {battery.isCharging ? (
                <BatteryChargingFull color="primary" fontSize="small" />
              ) : (
                <BatteryFull color="primary" fontSize="small" />
              )}
              <Typography variant="body2" fontWeight={600}>
                {label} {battery.level}%
              </Typography>
            </Paper>
          ))}
        </Stack>
        {deviceState.firmwareVersion && (
          <Typography variant="body2" color="text.secondary">
            {t("deviceInfo.firmwareVersion")}:{" "}
            {deviceState.firmwareVersion.major}.
            {String(deviceState.firmwareVersion.minor).padStart(2, "0")}
            {deviceState.serialNumber ? ` ・ ${deviceState.serialNumber}` : ""}
          </Typography>
        )}
      </Stack>
    </Paper>
  );
}

function NavCard({
  icon,
  title,
  subtitle,
  onClick,
}: {
  icon: React.ReactNode;
  title: string;
  subtitle?: string;
  onClick: () => void;
}) {
  return (
    <Paper
      elevation={0}
      component="button"
      onClick={onClick}
      sx={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        gap: 2,
        padding: 1.75,
        borderRadius: 5,
        border: "1px solid",
        borderColor: "divider",
        cursor: "pointer",
        textAlign: "left",
        background: "transparent",
        "&:hover": { borderColor: "primary.main" },
      }}
    >
      <Box
        sx={{
          padding: 1.25,
          borderRadius: 4,
          backgroundColor: "primary.main",
          color: "primary.contrastText",
          display: "flex",
        }}
      >
        {icon}
      </Box>
      <Box sx={{ flexGrow: 1 }}>
        <Typography fontWeight={600}>{title}</Typography>
        {subtitle && (
          <Typography variant="body2" color="text.secondary">
            {subtitle}
          </Typography>
        )}
      </Box>
      <ChevronRight color="disabled" />
    </Paper>
  );
}
