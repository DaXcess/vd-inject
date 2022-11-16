# Virtual Desktop Oculus Runtime Injector

This is a simple tool to inject Virtual Desktop's Oculus VR runtime into a VR game, allowing you to play games through Virtual Desktop without
SteamVR.

Note: This tool only works on games that have first class support for the Oculus SDK and are set up to use the Oculus runtime. It will not work on games that only support SteamVR, or that have their VR runtime set to OpenXR/SteamVR.

## Usage

1. Download the latest release from the [releases page](https://github.com/DaXcess/vd-inject/releases)
2. Run the executable

```bash
vd-inject.exe <game>
```

Where `<game>` is the process name of the game you want to inject into (e.g. "Beat Saber.exe").
