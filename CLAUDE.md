# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Open Reverb is a Rust-based open-source voice, video, and text communication platform similar to TeamSpeak and Discord. It provides a self-hostable server and cross-platform clients for Windows and macOS.

## Commands

### Build and Run

```bash
# Build all components in debug mode
cargo build

# Build all components in release mode
cargo build --release

# Build specific component
cargo build -p open-reverb-server
cargo build -p open-reverb-client
cargo build -p open-reverb-common

# Run the server
cargo run -p open-reverb-server

# Run the client
cargo run -p open-reverb-client

# Build with platform-specific scripts
# macOS
./scripts/build_macos.sh

# Windows
scripts\build_windows.bat

# Cross-compile for Windows from macOS/Linux
./scripts/cross_compile_windows.sh
```

### Testing

```bash
# Run all tests
cargo test

# Run specific component tests
cargo test -p open-reverb-server
cargo test -p open-reverb-client
cargo test -p open-reverb-common

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Check formatting without applying changes
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Fix linting issues automatically where possible
cargo clippy --fix
```

## Project Structure

The project is organized as a Rust workspace with three main components:

### Common (`open-reverb-common`)
- Shared models, protocols, and utilities
- Protocol definitions for client-server communication
- Data models for users, channels, servers
- Error definitions and handling

### Server (`open-reverb-server`)
- TCP-based server implementation
- Session management for client connections
- Channel and user management
- Authentication system
- VOIP, video, and screen sharing data routing

### Client (`open-reverb-client`)
- Cross-platform GUI using egui
- Audio capture and playback using cpal
- Video capture and display
- Screen sharing functionality
- Connection management with the server

## Architecture

- **Connection Protocol**: TCP-based with length-delimited frames containing serialized JSON messages
- **Audio/Video**: Raw audio/video data transmitted as binary payloads
- **State Management**: Server maintains the state of all channels and users
- **Authentication**: Simple username/password with server-side hashing using Argon2

## Cross-Platform Support

The client is designed to work on both Windows and macOS using:
- egui for cross-platform GUI
- cpal for cross-platform audio
- Platform-specific video capture APIs

## Important Files

- `open-reverb-common/src/protocol.rs`: Message definitions for client-server communication
- `open-reverb-common/src/models.rs`: Data models for the application
- `open-reverb-server/src/server.rs`: Server implementation and state management
- `open-reverb-server/src/session.rs`: Client session handling
- `open-reverb-client/src/app.rs`: Main client application state and logic
- `open-reverb-client/src/connection.rs`: Client-server communication
- `open-reverb-client/src/audio.rs`: Audio capture and playback
- `open-reverb-client/src/video.rs`: Video capture and display
- `open-reverb-client/src/ui.rs`: User interface components