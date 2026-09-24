import { GitHub } from "@mui/icons-material";
import { Box, Link, Paper, Stack, Typography } from "@mui/material";
import { useTranslation } from "react-i18next";

export function HomePage() {
  const { t } = useTranslation();
  return (
    <Stack textAlign="center" spacing={3} alignItems="center">
      {navigator.bluetooth == undefined && (
        <Typography>{t("application.webBluetoothNotSupported")}</Typography>
      )}
      <Paper
        elevation={0}
        sx={{
          padding: { xs: 3, sm: 5 },
          borderRadius: 8,
          maxWidth: 620,
          border: "1px solid",
          borderColor: "divider",
          boxShadow: "0 2px 16px rgb(0 0 0 / 0.08)",
        }}
      >
        <Stack spacing={1.5}>
          <Typography variant="h5">{t("home.title")}</Typography>
          <Typography color="text.secondary">{t("home.description")}</Typography>
          <Typography color="text.secondary">{t("home.howto")}</Typography>
          <Box paddingTop={1}>
            <Link
              href="https://github.com/oppzippy/OpenSCQ30"
              color="inherit"
              aria-label={t("github").toString()}
            >
              <GitHub />
            </Link>
          </Box>
        </Stack>
      </Paper>
    </Stack>
  );
}
