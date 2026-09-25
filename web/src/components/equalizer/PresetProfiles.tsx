import { Check } from "@mui/icons-material";
import { Grid, Paper, Stack, Typography } from "@mui/material";
import React from "react";
import { useTranslation } from "react-i18next";
import { usePresetEqualizerProfiles } from "../../hooks/usePresetEqualizerProfiles";
import { EqualizerLine } from "./EqualizerLine";
import { PresetEqualizerProfile } from "../../libTypes/DeviceState";

interface Props {
  profile: PresetEqualizerProfile | "custom";
  onProfileSelected: (presetProfile: PresetEqualizerProfile | "custom") => void;
}

/**
 * Preset picker as a grid of cards with the frequency curve preview,
 * in the style of common headphone companion apps (original design).
 */
export const PresetProfiles = React.memo(function (props: Props) {
  const { t } = useTranslation();
  const presetProfiles = usePresetEqualizerProfiles();
  const { onProfileSelected } = props;

  return (
    <Grid container spacing={1}>
      <Grid item xs={6} sm={4} md={3}>
        <PresetCard
          selected={props.profile == "custom"}
          name={t("equalizer.custom")}
          onClick={() => onProfileSelected("custom")}
        />
      </Grid>
      {presetProfiles.map((profile) => (
        <Grid item xs={6} sm={4} md={3} key={profile.id}>
          <PresetCard
            selected={props.profile == profile.id}
            name={profile.name}
            volumeAdjustments={profile.values}
            onClick={() => onProfileSelected(profile.id)}
          />
        </Grid>
      ))}
    </Grid>
  );
});

function PresetCard({
  selected,
  name,
  volumeAdjustments,
  onClick,
}: {
  selected: boolean;
  name: string;
  volumeAdjustments?: readonly number[];
  onClick: () => void;
}) {
  return (
    <Paper
      elevation={0}
      component="button"
      onClick={onClick}
      sx={{
        width: "100%",
        padding: 1.25,
        borderRadius: 4,
        border: "2px solid",
        borderColor: selected ? "primary.main" : "divider",
        backgroundColor: selected ? "action.selected" : "transparent",
        cursor: "pointer",
        textAlign: "left",
        position: "relative",
      }}
    >
      <Stack spacing={0.5}>
        <Typography
          variant="body2"
          fontWeight={600}
          noWrap
          sx={{ paddingRight: 3 }}
        >
          {name}
        </Typography>
        {volumeAdjustments && (
          <EqualizerLine volumeAdjustments={[...volumeAdjustments]} />
        )}
      </Stack>
      {selected && (
        <Check
          color="primary"
          fontSize="small"
          sx={{ position: "absolute", top: 8, right: 8 }}
        />
      )}
    </Paper>
  );
}
