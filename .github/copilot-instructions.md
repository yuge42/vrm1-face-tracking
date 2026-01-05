# Copilot Instructions for VRM1 Face Tracking

This document provides comprehensive guidance for GitHub Copilot coding agents working on the VRM1 Face Tracking project.

## Project Overview

VRM1 Face Tracking is a real-time face tracking application for VRM models using MediaPipe Face Landmarker. The project consists of:

- **Rust Application** (main codebase): Bevy-based application that receives face tracking data
- **Python Tracker** (`tools/mediapipe_tracker.py`): MediaPipe-based face tracking script that outputs blendshape data via stdout
- **IPC Library** (`crates/tracker_ipc`): Handles communication between the Rust app and Python tracker

### Key Technologies

- **Rust**: Main application language (edition 2024, rust-version 1.88+)
- **Bevy**: Game engine framework (v0.17)
- **Python**: Face tracking implementation (requires Python 3)
- **MediaPipe**: Face landmark detection library
- **OpenCV**: Image processing for Python tracker

## Repository Structure

```
.
├── src/                        # Main Rust application source
│   └── main.rs                # Application entry point with tracker integration
├── crates/                     # Rust workspace crates
│   └── tracker_ipc/           # IPC library for Python-Rust communication
│       ├── src/lib.rs         # Spawns Python process, reads JSON from stdout
│       └── Cargo.toml
├── tools/                      # Python face tracking tools
│   ├── mediapipe_tracker.py   # MediaPipe face tracker script
│   ├── requirements.txt       # Python dependencies
│   ├── face_landmarker.task   # MediaPipe model (NOT in git, must download)
│   └── README.md
├── Cargo.toml                  # Workspace and package configuration
├── .github/
│   └── workflows/
│       └── ci.yml             # CI configuration with build/test/lint
└── README.md
```

## Development Setup

### System Dependencies (Linux/Ubuntu)

The following system packages are required for building Bevy applications:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwayland-dev \
  libxkbcommon-dev \
  libasound2-dev \
  libudev-dev \
  pkg-config
```

**CRITICAL**: These dependencies MUST be installed before running `cargo build` or `cargo clippy`. Without them, you will encounter a build error:
```
error: failed to run custom build command for `wayland-sys`
The system library `wayland-client` required by crate `wayland-sys` was not found.
```

### Rust Setup

- Toolchain: Stable 1.88+ (configured in `Cargo.toml`)
- Required components: `rustfmt`, `clippy`

```bash
# The CI uses:
rustup toolchain install 1.88 --component rustfmt clippy
```

### Python Setup

The project requires a Python virtual environment with MediaPipe and OpenCV:

```bash
# Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate  # On Linux/macOS
# .venv\Scripts\activate   # On Windows

# Install Python dependencies
cd tools
pip install -r requirements.txt

# Download MediaPipe model (REQUIRED - not in git due to size)
wget -O face_landmarker.task \
  https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
cd ..
```

**Note**: The `face_landmarker.task` file is gitignored (see `tools/*.task` in `.gitignore`) and must be downloaded separately.

## Build and Test Commands

### Linting

```bash
# Check code formatting (MUST pass in CI)
cargo fmt --all -- --check

# Run clippy linter with warnings as errors (MUST pass in CI)
cargo clippy --all-targets --all-features -- -D warnings
```

### Building

```bash
# Build the project
cargo build --verbose

# Build in release mode
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test --verbose
```

**Note**: Currently there are no tests defined (0 tests run). When adding tests, follow Rust best practices with `#[test]` and `#[cfg(test)]` modules.

### Running the Application

```bash
# Option 1: Use default python3
cargo run

# Option 2: Use virtual environment Python
export PYTHON_BIN=.venv/bin/python
cargo run

# Option 3: Custom Python path
export PYTHON_BIN=/usr/bin/python3.11
cargo run
```

The application requires:
1. Python environment with MediaPipe/OpenCV installed
2. `tools/face_landmarker.task` model file downloaded
3. A webcam connected and accessible

## Code Conventions

### Rust Code Style

- **Edition**: 2024 (specified in `Cargo.toml`)
- **Formatting**: Use `rustfmt` with default settings
- **Linting**: Clippy warnings treated as errors in CI
- **License**: Dual-licensed MIT OR Apache-2.0
- **Imports**: Follow standard Rust module organization

### Key Patterns

1. **Resource Management**: Bevy uses the Entity-Component-System (ECS) pattern
   - Resources are inserted with `commands.insert_resource()`
   - Systems access resources via `Res<T>` or `ResMut<T>`

2. **IPC Pattern**: Communication with Python uses JSON over stdout
   - Python tracker outputs one JSON object per line
   - Rust reads using `BufReader` and deserializes with `serde_json`
   - Channel-based communication using `crossbeam-channel`

3. **Error Handling**: 
   - Use `.expect()` with descriptive messages for unrecoverable errors
   - Use pattern matching for recoverable errors
   - See `tracker_ipc/src/lib.rs` for examples

### File Organization

- Main application code: `src/main.rs`
- Workspace crates: `crates/*/src/`
- Python tools: `tools/`
- Tests: Co-located with code using `#[cfg(test)]` modules

## Environment Variables

- `PYTHON_BIN`: Path to Python executable (default: `python3`)
  - Example: `.venv/bin/python`
  - Used by `src/main.rs` to spawn the tracker process

## Common Issues and Workarounds

### Issue 1: Wayland Build Failure

**Error**:
```
error: failed to run custom build command for `wayland-sys v0.31.7`
The system library `wayland-client` required by crate `wayland-sys` was not found.
```

**Workaround**: Install system dependencies as shown in the "System Dependencies" section above.

### Issue 2: MediaPipe Model Not Found

**Error** (from Python tracker):
```json
{"error": "Model file not found at tools/face_landmarker.task. Please download it from: ..."}
```

**Workaround**: Download the model file:
```bash
cd tools
wget -O face_landmarker.task https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
cd ..
```

### Issue 3: Webcam Access Failure

**Error** (from Python tracker):
```json
{"error": "Failed to open webcam (device index 0). Please check that a webcam is connected and accessible."}
```

**Workaround**: 
- Ensure a webcam is connected
- Check permissions (on Linux, user may need to be in `video` group)
- Try a different device index if multiple cameras are present

### Issue 4: Python Dependencies Missing

**Error**: Import errors when running the tracker

**Workaround**: Ensure virtual environment is activated and dependencies are installed:
```bash
source .venv/bin/activate
pip install -r tools/requirements.txt
```

## CI/CD Pipeline

The CI workflow (`.github/workflows/ci.yml`) runs on:
- Push to `main` branch
- Pull requests to `main` branch

CI Steps:
1. Checkout code
2. Install Rust toolchain (1.88 with rustfmt, clippy)
3. Install system dependencies
4. Cache cargo artifacts
5. Check formatting (`cargo fmt --all -- --check`)
6. Run clippy (`cargo clippy --all-targets --all-features -- -D warnings`)
7. Build (`cargo build --verbose`)
8. Test (`cargo test --verbose`)

**Important**: All CI checks must pass for PRs to be merged.

## Making Changes

### Adding Rust Dependencies

1. Add to appropriate `Cargo.toml`:
   - Workspace dependencies: root `Cargo.toml` under `[workspace.dependencies]`
   - Package-specific: individual `Cargo.toml` files
2. Run `cargo build` to update `Cargo.lock`
3. Ensure CI passes

### Adding Python Dependencies

1. Add to `tools/requirements.txt`
2. Update in virtual environment: `pip install -r tools/requirements.txt`
3. Document in `tools/README.md` if significant

### Modifying the Tracker IPC Protocol

The IPC protocol is defined by the `TrackerFrame` struct in `crates/tracker_ipc/src/lib.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct TrackerFrame {
    pub ts: f64,
    pub blendshapes: HashMap<String, f32>,
}
```

**When modifying**:
1. Update the struct in Rust
2. Update the JSON output format in `tools/mediapipe_tracker.py`
3. Ensure backward compatibility or version the protocol
4. Update documentation in both `tools/README.md` and code comments

### Adding New Blendshapes or Features

1. Modify `tools/mediapipe_tracker.py` to output new data in the JSON
2. Update `src/main.rs` to process the new data
3. Ensure the data flows through the `TrackerFrame` struct
4. Add tests if applicable

## Best Practices

1. **Always run linting before committing**: `cargo fmt --all && cargo clippy --all-targets --all-features`
2. **Test locally before pushing**: Ensure `cargo build` and `cargo test` pass
3. **Document significant changes**: Update relevant README files
4. **Keep system dependencies minimal**: Only add if absolutely necessary
5. **Maintain dual-licensing**: All contributions are dual-licensed MIT OR Apache-2.0
6. **Follow Bevy patterns**: Use ECS, Resources, and Systems appropriately
7. **Handle errors gracefully**: Use proper error types and messages
8. **Keep Python-Rust boundary clean**: All communication via JSON stdout

## Additional Resources

- [Bevy Documentation](https://bevyengine.org/)
- [MediaPipe Face Landmarker](https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker)
- [Rust Edition Guide (2024)](https://doc.rust-lang.org/edition-guide/)
- Project README: `README.md`
- Tools README: `tools/README.md`

## Questions or Issues

If you encounter issues not covered in this guide:
1. Check the CI logs for similar failures
2. Review recent commits for related changes
3. Check the issue tracker for known problems
4. Consult the project documentation in README files
