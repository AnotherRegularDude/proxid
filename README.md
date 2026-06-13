# proxid

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.96%2B-blue.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/AnotherRegularDude/proxid)

> OpenRouter API proxy with audio transcoding.

`proxid` is a small OpenAI-compatible HTTP proxy that wraps OpenRouter's speech and transcription endpoints. It accepts common audio upload formats, transcodes them via FFmpeg, and forwards base64-encoded JSON to the provider. Text-to-speech output is always requested as PCM from OpenRouter and then locally transcoded to the requested format.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)

## Features

- **OpenAI-compatible endpoints** for `/api/v1/audio/speech` and `/api/v1/audio/transcriptions`.
- **Input format flexibility** — accepts `mp3`, `wav`, `flac`, `m4a`, `ogg`, `webm`, `mp4`, `aac`, `opus`, `pcm`, and more for transcription.
- **Output format flexibility** — speech responses can be returned as `pcm`, `mp3`, `wav`, `flac`, `aac`, or `opus`.
- **Local transcoding** via FFmpeg (pass-through when the requested PCM sample rate already matches).
- **Safe secret handling** — the provider API key is stored as a `secrecy::SecretString` and never logged.
- **Container-ready** — multi-stage Dockerfile with a non-root user, static FFmpeg binary, and health check.

## Installation

### Prerequisites

- Rust 1.96 or newer
- FFmpeg installed and available on `PATH`
- An [OpenRouter](https://openrouter.ai) API key

### From source

```bash
git clone https://github.com/AnotherRegularDude/proxid.git
cd proxid
# Optional: use rust-toolchain.toml
cargo build --release
```

The binary is produced at `target/release/proxid`.

### Docker

```bash
docker build -t proxid:latest .
docker run --env-file .env -p 8800:8800 proxid:latest
```

## Usage

1. Copy `.env.example` to `.env` and set your API key:

   ```bash
   cp .env.example .env
   # Edit PROXID__PROVIDER__API_KEY
   ```

2. Run the service:

   ```bash
   cargo run
   ```

3. Verify it is up:

   ```bash
   curl http://localhost:8800/health
   ```

### Text-to-speech example

```bash
curl -X POST http://localhost:8800/api/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "google/gemini-3.1-flash-tts-preview",
    "input": "Hello from proxid",
    "voice": "alloy",
    "response_format": "mp3"
  }' \
  --output speech.mp3
```

### Transcription example

```bash
curl -X POST http://localhost:8800/api/v1/audio/transcriptions \
  -H "Content-Type: multipart/form-data" \
  -F "file=@sample.wav" \
  -F "model=openai/whisper-large-v3"
```

Supported response formats for speech are `pcm`, `mp3`, `wav`, `flac`, `aac`, and `opus`.

## Project Structure

```
proxid/
├── config/                 # TOML defaults and test overrides
│   ├── default.toml
│   └── test.toml
├── src/
│   ├── main.rs             # Entry point: config, tracing, axum server
│   ├── lib.rs              # Public re-exports and tracing init
│   ├── app.rs              # AppState, router wiring, body limit
│   ├── config/             # Settings structs and Figment loader
│   ├── core/               # AppError, AudioFormat/OutputFormat, Usage
│   │   └── audio/
│   ├── features/           # HTTP handlers and DTOs
│   │   ├── meta/           # /, /health
│   │   ├── speech/         # TTS endpoint
│   │   └── transcription/  # STT endpoint
│   └── infrastructure/     # OpenRouter client + FFmpeg transcoder
│       ├── openrouter/
│       └── transcoder/
├── tests/                  # Integration tests with mockito fixtures
├── Cargo.toml
├── Dockerfile
├── justfile
└── .env.example
```

## Configuration

Configuration is layered: `config/default.toml` → optional custom config file → `PROXID__*` environment overrides.

Required environment variable:

| Variable | Description |
|----------|-------------|
| `PROXID__PROVIDER__API_KEY` | OpenRouter API key |

Common optional overrides:

| Variable | Default |
|----------|---------|
| `PROXID__SERVER__HOST` | `127.0.0.1` |
| `PROXID__SERVER__PORT` | `8800` |
| `PROXID__PROVIDER__BASE_URL` | `https://openrouter.ai/api/v1` |
| `PROXID__PROVIDER__DEFAULT_TRANSCRIPTION_MODEL` | `openai/whisper-large-v3-turbo` |
| `PROXID__PROVIDER__DEFAULT_SPEECH_MODEL` | `google/gemini-3.1-flash-tts-preview` |
| `PROXID__PROVIDER__REQUEST_TIMEOUT_SECS` | `60` |
| `PROXID__LOGGING__FILTER` | `proxid=debug,tower_http=info` |

Audio quality settings such as `PROXID__AUDIO__MP3_BITRATE_BPS` and `PROXID__AUDIO__PCM_SAMPLE_RATE` are also configurable. See `.env.example` for the full list.

## API Reference

The proxy exposes three routes:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Service identity string |
| `GET` | `/health` | Health check (`200 OK`) |
| `POST` | `/api/v1/audio/speech` | OpenAI-compatible TTS |
| `POST` | `/api/v1/audio/transcriptions` | OpenAI-compatible STT |

Request and response shapes follow the [OpenAI audio API](https://platform.openai.com/docs/guides/audio) conventions. Errors are returned as JSON in the shape `{ "error": { "message": "...", "type": "..." } }`.

## Contributing

1. Install the pinned toolchain (includes `rustfmt` and `clippy`):

   ```bash
   rustup show
   ```

2. Install `cargo-nextest` if you want to run tests via `just test`:

   ```bash
   cargo install cargo-nextest --locked
   ```

3. Run the full CI pipeline:

   ```bash
   just ci
   ```

This runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo nextest run`.

Other useful recipes:

- `just build` — debug build
- `just lint` — lint without tests
- `just lint-fix` — auto-fix formatting and clippy warnings
- `just docker-build` — build the Docker image locally
- `just docker-run` — run the container with `.env`

Please keep `unsafe_code` forbidden and ensure all lints pass before opening a pull request.

## License

MIT © [AnotherRegularDude](https://github.com/AnotherRegularDude)
