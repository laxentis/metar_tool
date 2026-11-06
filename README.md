# METAR Tool

A small desktop application for composing METAR observation reports. It provides a graphical editor for common report groups, formats the result as a METAR string, and can copy the report to the clipboard.

<img width="600" height="500" alt="image" src="https://github.com/user-attachments/assets/1fac0549-0f80-4d48-b837-788a30904c27" />


## Features

- Choose METAR, METAR COR, SPECI, or SPECI COR as the message type.
- Edit the station ICAO code, observation day and time, wind, visibility, cloud layers, present weather, temperature, dew point, and pressure.
- Configure variable wind direction, gusts, cloud layers, and weather phenomena.
- Preview the formatted report and copy it to the clipboard.
- Set an optional hourly sound reminder with configurable active hours and a test sound.
- Save and restore application settings through egui persistence.

## Requirements

- Rust toolchain with Cargo (the project uses Rust edition 2024).
- A working audio output device for reminder sounds.

## Run

From the project directory:

```sh
cargo run
```

Build a release version with:

```sh
cargo build --release
```

The application is a native eframe/egui desktop program. Its reminder sound is embedded from `assets/bell.wav`.

## Using the editor

Fill in the observation details in the main window. The generated METAR appears at the bottom; use the clipboard button beside it to copy the report. The **Overwrite** option enables editing the displayed text directly.

Configure one or more HTTP(S) endpoints from **Settings → Servers**. Each endpoint can have its own optional API key, which is sent in that server's `X-API-Key` request header. Leave a server's key empty to omit the header for that endpoint. Use **Send to servers** to POST the generated report (or the edited overwrite report) as `text/plain` to every configured URL. The app displays each endpoint's response status.

Open **Sound notifications** to enable or disable hourly reminders, set their active hours, and play a test sound. Reminder playback requires a usable default audio output device.

## Project layout

- `src/app.rs` — desktop interface and persisted application state.
- `src/metar.rs` — METAR report model and token formatting.
- `src/reminder.rs` — hourly reminder and sound playback.
- `assets/bell.wav` — sound embedded in the application binary.
