import { ThemeProvider, createTheme } from "@mui/material";
import { ToastQueue } from "./ToastQueue";
import { AppContents } from "./AppContents";

function App() {
  const theme = createTheme({
    colorSchemes: {
      light: {
        palette: {
          primary: { main: "#474e3d" },
        },
      },
      dark: {
        palette: {
          primary: { main: "#a3b793" },
        },
      },
    },
    shape: { borderRadius: 16 },
    typography: {
      fontFamily: [
        "Roboto",
        "Noto Sans JP",
        "Hiragino Kaku Gothic ProN",
        "Hiragino Sans",
        "Meiryo",
        "system-ui",
        "sans-serif",
      ].join(","),
      h6: { fontWeight: 700 },
      h5: { fontWeight: 700 },
      button: { textTransform: "none", fontWeight: 600 },
    },
    components: {
      MuiButton: {
        styleOverrides: {
          root: { borderRadius: 999, padding: "8px 20px" },
        },
      },
      MuiToggleButton: {
        styleOverrides: {
          root: {
            borderRadius: "14px !important",
            padding: "8px 12px",
            textTransform: "none",
          },
        },
      },
      MuiPaper: {
        styleOverrides: { root: { backgroundImage: "none" } },
      },
      MuiSnackbarContent: {
        styleOverrides: {
          root: {
            borderRadius: 16,
            padding: "12px 20px",
          },
        },
      },
      MuiFormControlLabel: {
        styleOverrides: { label: { lineHeight: 1.4 } },
      },
    },
  });
  return (
    <ThemeProvider theme={theme}>
      <ToastQueue>
        <AppContents />
      </ToastQueue>
    </ThemeProvider>
  );
}

export default App;
