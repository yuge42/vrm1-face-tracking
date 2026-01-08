# VRM1 Face Tracking

Real-time face tracking for VRM models using MediaPipe Face Landmarker.

## Features

- **VRM 1.0 Model Loading**: Load and display VRM 1.0 models via Bevy's asset server
- **File Dialog**: Select VRM models from the filesystem at runtime using a native file picker (press 'O' key)
- **Real-time Face Tracking**: Capture face tracking data using MediaPipe
- **3D Rendering**: Display VRM models with proper lighting and camera setup

## Setup

### VRM Model

Place your VRM 1.0 model file in the `assets/vrm/` directory:

1. Obtain a VRM 1.0 model from [VRoid Hub](https://hub.vroid.com/), [VRoid Studio](https://vroid.com/studio), or other VRM-compatible sources
2. Place the `.vrm` file in `assets/vrm/` and name it `model.vrm`
3. Alternatively, modify the model path in `src/main.rs`

See `assets/vrm/README.md` for more information about VRM models.

### Python Environment

This project requires Python with MediaPipe and OpenCV. We recommend using a virtual environment:

```bash
# Create virtual environment
python3 -m venv .venv

# Activate virtual environment
# On Linux/macOS:
source .venv/bin/activate
# On Windows:
# .venv\Scripts\activate

# Install dependencies
cd tools
pip install -r requirements.txt

# Download MediaPipe model
wget -O face_landmarker.task https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
cd ..
```

### Rust Application

Build and run the application:

```bash
# Set Python path (optional, defaults to python3)
export PYTHON_BIN=.venv/bin/python

# Build and run
cargo run
```

The application will:
1. Open a 3D rendering window with camera and lighting
2. Load the VRM model from the asset path `vrm/model.vrm` (which corresponds to `assets/vrm/model.vrm` in the project directory) using Bevy's asset server
3. Start the Python face tracker and print blendshape data to the console

**Note**: A VRM model file is optional. If the asset at `vrm/model.vrm` is not found, the application will still run and track face data, but no model will be displayed in the 3D scene. Models must be placed in the `assets/` directory to be loaded by Bevy's asset server.

## Usage

### Loading VRM Models

You can load VRM models in two ways:

1. **Default model**: Place a VRM file at `assets/vrm/model.vrm` (loaded automatically on startup)
2. **File dialog**: Press the `O` key while the application is running to open a native file picker and select any VRM file from your filesystem

When you select a file via the file dialog, it will be copied to the `assets/vrm/` directory and loaded, replacing the current model. The file dialog runs in a separate thread to keep the application responsive.

## Environment Variables

- `PYTHON_BIN`: Path to Python executable (default: `python3`)
  - Example: `.venv/bin/python` or `/usr/bin/python3`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Some external dependencies may carry additional copyright notices and license terms.
When building and distributing binaries, those external library licenses may be included.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.