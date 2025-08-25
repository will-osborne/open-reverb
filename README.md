# Open Reverb

Open Reverb is an open-source voice, video, and text communication platform similar to TeamSpeak and Discord. It provides a self-hostable server and cross-platform clients for Windows and macOS.

## Features

- Voice over IP (VoIP) communication
- Video calling
- Screen sharing
- Channel-based communication
- Cross-platform support (Windows, macOS)
- Self-hostable server

## Architecture

Open Reverb is structured as a Rust workspace with three main components:

1. **open-reverb-common**: Shared code, models, and protocols used by both client and server
2. **open-reverb-server**: The server implementation that handles connections, channels, and message routing
3. **open-reverb-client**: The client application with a cross-platform GUI using egui

## Building from Source

### Prerequisites

- Rust toolchain (1.63+)
- Cargo package manager
- Platform-specific development dependencies

#### Windows Dependencies

- Visual Studio Build Tools with C++ support

#### macOS Dependencies

- Xcode Command Line Tools

### Optional Features

Open Reverb supports conditional compilation of certain features:

```bash
# Build with audio support (requires cpal)
cargo build --features audio

# Build with video support (requires gstreamer)
cargo build --features video

# Build with all features
cargo build --features "audio video"
```

Note: Full audio and video support requires additional platform-specific dependencies:

- Audio: CPAL dependencies (ALSA on Linux, CoreAudio on macOS)
- Video: GStreamer libraries with appropriate plugins

### Building

You can use the provided build scripts:

```bash
# On macOS
./scripts/build_macos.sh

# On Windows
scripts\build_windows.bat

# Cross-compile for Windows from macOS/Linux
./scripts/cross_compile_windows.sh
```

Or build manually:

```bash
# Build everything in debug mode
cargo build

# Build everything in release mode
cargo build --release

# Build specific component
cargo build -p open-reverb-server --release
```

## Running

### Server

```bash
./target/release/open-reverb-server
```

### Client

```bash
./target/release/open-reverb-client
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.