import { render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BehaviorSubject } from "rxjs";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ToastQueue } from "../../../src/components/ToastQueue";
import { DeviceSettings } from "../../../src/components/deviceSettings/DeviceSettings";
import { useCustomEqualizerProfiles } from "../../../src/components/deviceSettings/hooks/useCustomEqualizerProfiles";
import { upsertCustomEqualizerProfile } from "../../../src/storage/customEqualizerProfiles";
import { Device } from "../../../src/bluetooth/Device";
import {
  DeviceState,
  EqualizerConfiguration,
  SoundModes,
} from "../../../src/libTypes/DeviceState";
import { EqualizerHelper } from "../../../src/../wasm/pkg/openscq30_web_wasm";
import { CustomEqualizerProfile } from "../../../src/storage/db";

vi.mock(
  "../../../src/components/deviceSettings/hooks/useCustomEqualizerProfiles",
  () => {
    const emptyArray: CustomEqualizerProfile[] = [];
    return {
      useCustomEqualizerProfiles: vi.fn(() => emptyArray),
    };
  },
);

vi.mock("../../../src/storage/customEqualizerProfiles", () => {
  return {
    upsertCustomEqualizerProfile: vi.fn(),
  };
});

describe("Device Settings", () => {
  let device: Device;
  let user: ReturnType<typeof userEvent.setup>;
  beforeEach(() => {
    vi.useFakeTimers({
      shouldAdvanceTime: true,
    });
    user = userEvent.setup();
    const mockDevice = {
      state: new BehaviorSubject<DeviceState>({
        deviceFeatures: {
          availableSoundModes: {
            ambientSoundModes: ["normal", "transparency", "noiseCanceling"],
            transparencyModes: ["fullyTransparent", "vocalMode"],
            noiseCancelingModes: ["indoor", "outdoor", "transport", "custom"],
            customNoiseCanceling: false,
          },
          hasHearId: true,
          numEqualizerChannels: 1,
          numEqualizerBands: 8,
          hasDynamicRangeCompression: true,
          hasButtonConfiguration: true,
          hasWearDetection: true,
          hasTouchTone: true,
          hasAutoPowerOff: true,
          dynamicRangeCompressionMinFirmwareVersion: null,
          hasAmbientSoundModeCycle: false,
          hasGamingMode: false,
          hasSurroundSound: false,
          hasDualConnections: false,
          hasLowBatteryPrompt: false,
        },
        twsStatus: null,
        battery: {
          type: "singleBattery",
          isCharging: true,
          level: 5,
        },
        soundModes: {
          ambientSoundMode: "noiseCanceling",
          noiseCancelingMode: "transport",
          transparencyMode: "fullyTransparent",
          customNoiseCanceling: 0,
        },
        soundModesTypeTwo: null,
        equalizerConfiguration: {
          presetProfile: "SoundcoreSignature",
          volumeAdjustments: [
            ...EqualizerHelper.getPresetProfileVolumeAdjustments(
              "SoundcoreSignature",
            ),
          ],
        },
        ageRange: null,
        gender: null,
        buttonConfiguration: null,
        hearId: null,
        firmwareVersion: null,
        serialNumber: null,
        ambientSoundModeCycle: null,
        soundModesTypeThree: null,
        gamingMode: null,
        surroundSound: null,
        dualConnections: null,
        lowBatteryPrompt: null,
      }),
      connect: vi.fn<() => void>(),
      async setSoundModes(soundModes: SoundModes) {
        this.state.next({
          ...this.state.value,
          soundModes,
        });
      },
      async setEqualizerConfiguration(
        equalizerConfiguration: EqualizerConfiguration,
      ) {
        this.state.next({
          ...this.state.value,
          equalizerConfiguration,
        });
      },
    };
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    device = mockDevice as unknown as Device;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function renderSettings() {
    return render(
      <DeviceSettings
        device={device}
        // eslint-disable-next-line @typescript-eslint/no-empty-function
        disconnect={() => {}}
      />,
    );
  }

  it("should change ambient sound mode", async () => {
    const renderResult = renderSettings();
    expect(device.state.value.soundModes?.ambientSoundMode).toEqual(
      "noiseCanceling",
    );
    await user.click(renderResult.getByText("soundModes.soundModes"));
    await user.click(renderResult.getByText("ambientSoundMode.normal"));

    expect(device.state.value.soundModes?.ambientSoundMode).toEqual("normal");
  });

  it("should change noise canceling mode", async () => {
    const renderResult = renderSettings();
    expect(device.state.value.soundModes?.noiseCancelingMode).toEqual(
      "transport",
    );
    await user.click(renderResult.getByText("soundModes.soundModes"));
    await user.click(renderResult.getByText("noiseCancelingMode.indoor"));
    expect(device.state.value.soundModes?.noiseCancelingMode).toEqual(
      "indoor",
    );
  });

  it("should change equalizer configuration", async () => {
    const renderResult = renderSettings();
    expect([
      ...device.state.value.equalizerConfiguration.volumeAdjustments,
    ]).toEqual([0, 0, 0, 0, 0, 0, 0, 0]);
    await user.click(renderResult.getByText("equalizer.equalizer"));
    await user.click(
      renderResult.getByText("presetEqualizerProfile.soundcoreSignature"),
    );
    await user.click(
      renderResult.getByText("presetEqualizerProfile.classical"),
    );
    vi.advanceTimersByTime(5000);
    expect([
      ...device.state.value.equalizerConfiguration.volumeAdjustments,
    ]).not.toEqual([0, 0, 0, 0, 0, 0, 0, 0]);
  });

  it("should switch to custom profile when moving a slider", async () => {
    const renderResult = renderSettings();
    await user.click(renderResult.getByText("equalizer.equalizer"));

    const numberInputs = renderResult.baseElement.querySelectorAll(
      "input[type='number']",
    );
    await user.type(numberInputs[0], "1");
    vi.advanceTimersByTime(5000);
    expect(
      device.state.value.equalizerConfiguration.presetProfile,
    ).toBeNull();
  });

  it("should not show custom profile create/delete buttons when a preset is selected", () => {
    const renderResult = renderSettings();
    renderResult.getByText("equalizer.equalizer").click();

    expect(
      renderResult.queryByRole("button", { name: "application.create" }),
    ).toBeFalsy();
    expect(
      renderResult.queryByRole("button", { name: "application.delete" }),
    ).toBeFalsy();
  });

  it("should apply a stored custom profile and allow returning to a preset", async () => {
    (useCustomEqualizerProfiles as ReturnType<typeof vi.fn>).mockReturnValue([
      { name: "test", values: [1, 0, 0, 0, 0, 0, 0, 0], id: 1 },
    ]);
    const renderResult = renderSettings();
    await user.click(renderResult.getByText("equalizer.equalizer"));

    // select the stored custom profile from the select
    await user.click(renderResult.getByLabelText("equalizer.customProfile"));
    await user.click(await renderResult.findByRole("option", { name: /test/ }));
    vi.advanceTimersByTime(5000);
    expect(
      device.state.value.equalizerConfiguration.presetProfile,
    ).toBeNull();
    expect([
      ...device.state.value.equalizerConfiguration.volumeAdjustments,
    ]).toEqual([1, 0, 0, 0, 0, 0, 0, 0]);

    // back to a preset via the grid card
    await user.click(
      renderResult.getByText("presetEqualizerProfile.classical"),
    );
    vi.advanceTimersByTime(5000);
    expect(
      device.state.value.equalizerConfiguration.presetProfile,
    ).toEqual("Classical");
  });

  it("should synchronize sliders and number input values", async () => {
    const renderResult = renderSettings();
    await user.click(renderResult.getByText("equalizer.equalizer"));

    const numberInputs: NodeListOf<HTMLInputElement> =
      renderResult.baseElement.querySelectorAll("input[type='number']");
    await user.type(numberInputs[0], "12");
    const sliders: NodeListOf<HTMLInputElement> =
      renderResult.baseElement.querySelectorAll("input[type='range']");
    expect(Number(sliders[0].value)).toEqual(12);
  });

  it("should debounce equalizer updates", async () => {
    const renderResult = renderSettings();
    await user.click(renderResult.getByText("equalizer.equalizer"));

    const numberInputs: NodeListOf<HTMLInputElement> =
      renderResult.baseElement.querySelectorAll("input[type='number']");
    await user.type(numberInputs[0], "1");

    expect(
      device.state.value.equalizerConfiguration.presetProfile,
    ).toEqual("SoundcoreSignature");
    vi.advanceTimersByTime(500);
    expect(
      device.state.value.equalizerConfiguration.presetProfile,
    ).toBeNull();
  });

  it("should display a toast when creating a custom profile fails", async () => {
    (
      upsertCustomEqualizerProfile as ReturnType<typeof vi.fn>
    ).mockRejectedValue(new Error("It should error"));
    const renderResult = render(
      <ToastQueue>
        <DeviceSettings
          device={device}
          // eslint-disable-next-line @typescript-eslint/no-empty-function
          disconnect={() => {}}
        />
      </ToastQueue>,
    );

    await user.click(renderResult.getByText("equalizer.equalizer"));
    await user.click(renderResult.getByText("equalizer.custom"));
    await user.click(
      renderResult.getByRole("button", {
        name: "equalizer.createCustomProfile",
      }),
    );
    await user.type(
      renderResult.getByLabelText("equalizer.profileName"),
      "test",
    );

    const consoleErrorMock = vi
      .spyOn(console, "error")
      .mockImplementation(() => {
        // do nothing
      });
    await user.click(
      renderResult.getByRole("button", { name: "application.create" }),
    );
    expect(consoleErrorMock).toHaveBeenCalled();
    consoleErrorMock.mockRestore();

    expect(
      renderResult.queryByText("errors.failedToCreateCustomProfile"),
    ).toBeTruthy();
  });
});
