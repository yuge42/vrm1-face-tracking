# VRM1 Face Tracking

Real-time face and upper body tracking for VRM models using MediaPipe Face Landmarker and Pose Landmarker.

## Features

- **VRM 1.0 Model Loading**: Load and display VRM 1.0 models from user data directory via custom asset source
- **File Dialog**: Select VRM models from the filesystem at runtime using a native file picker (press 'O' key)
- **User Data Directory**: VRM models are stored in platform-specific user data directories
- **Configuration**: Application configuration stored in platform-specific config directory
- **Real-time Face Tracking**: Capture face tracking data using MediaPipe Face Landmarker
  - Maps ARKit-style blendshapes to VRM expressions
  - Supports all VRM 1.0 expression presets (emotions, lip sync, blink, gaze)
- **Real-time Pose Tracking**: Capture upper body pose data using MediaPipe Pose Landmarker
  - Applies world coordinates to VRM bone rotations
  - Supports upper body bones: shoulders, elbows, wrists, and chest
  - Confidence-based filtering for stable tracking
- **Expression Adapter**: Flexible trait-based system for mapping tracker blendshapes to VRM expressions
- **Pose Adapter**: Converts MediaPipe 3D pose landmarks to VRM humanoid bone rotations
- **3D Rendering**: Display VRM models with proper lighting and camera setup

## Setup

### User Data Directory

The application stores VRM models and configuration in platform-specific user data directories (managed by the `directories` crate):

- **Linux**: `~/.local/share/vrm1-face-tracking/vrm_models/`
- **Windows**: `C:\Users\<USERNAME>\AppData\Roaming\vrm1-face-tracking\data\vrm_models\`
- **macOS**: `~/Library/Application Support/vrm1-face-tracking/vrm_models/`

The configuration file is stored in:

- **Linux**: `~/.config/vrm1-face-tracking/config.toml`
- **Windows**: `C:\Users\<USERNAME>\AppData\Roaming\vrm1-face-tracking\config\config.toml`
- **macOS**: `~/Library/Application Support/vrm1-face-tracking/config.toml`

These directories are created automatically when you first run the application.

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

# Download MediaPipe models
wget -O face_landmarker.task https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
wget -O pose_landmarker_full.task https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/1/pose_landmarker_full.task
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
2. Load the VRM model from the user data directory using a custom asset source
3. Start the Python face and pose tracker and print tracking data to the console

**Note**: A VRM model file is optional. If `model.vrm` is not found in the user data directory, the application will still run and track face data, but no model will be displayed in the 3D scene.

## Usage

### Loading VRM Models

You can load VRM models in two ways:

1. **Default model**: Place a VRM file named `model.vrm` in your user data directory (see paths in "User Data Directory" section above)
2. **File dialog**: Press the `O` key while the application is running to open a native file picker and select any VRM file from your filesystem

When you select a file via the file dialog, it will be copied to your user data directory and loaded using Bevy's custom asset source, replacing the current model. The file dialog runs in a separate thread to keep the application responsive.

### Configuration

The application configuration is stored in `config.toml` in your platform-specific config directory. The configuration includes:

- `user_vrm_dir`: Path to the directory where VRM models are stored
- `default_vrm_model`: Filename of the default VRM model to load on startup

The configuration file is created automatically with sensible defaults when you first run the application. You can edit it manually if needed.

## Architecture

### Custom VRM 1.0 Parser

The application uses a custom, minimal VRM 1.0 parser (`vrm_loader` crate) instead of external VRM libraries:

- **Lightweight**: Parses only what's needed for face tracking (no VRMA/animation assumptions)
- **Plain Data Structures**: VRM metadata exposed as simple Rust structs
- **Bevy Integration**: Works seamlessly with Bevy's asset system and built-in glTF loader
- **Console Logging**: Automatically prints VRM model metadata when loaded

The parser extracts:
- VRM metadata (name, authors, license, etc.)
- Expression definitions with morph target bindings
- Humanoid bone mapping
- Look-at configuration
- First-person view settings

For more details, see [crates/vrm_loader/README.md](crates/vrm_loader/README.md).

### Expression Adapter System

The application uses a flexible trait-based system to convert raw face tracking data into VRM expressions:

- **MediaPipe Face Landmarker** outputs 52 ARKit-style blendshapes (e.g., `eyeBlinkLeft`, `jawOpen`, `mouthSmileLeft`)
- **BlendshapeToExpression Trait** defines the interface for mapping tracker data to VRM expressions
- **ArkitToVrmAdapter** provides a default implementation mapping ARKit blendshapes to VRM 1.0 expression presets

#### VRM Expression Presets

The system supports all VRM 1.0 expression presets:

- **Emotions**: `happy`, `angry`, `sad`, `relaxed`, `surprised`
- **Lip Sync**: `aa`, `ih`, `ou`, `ee`, `oh`
- **Blink**: `blink`, `blinkLeft`, `blinkRight`
- **Gaze**: `lookUp`, `lookDown`, `lookLeft`, `lookRight`
- **Other**: `neutral`

For more details on the expression adapter system, see [crates/expression_adapter/README.md](crates/expression_adapter/README.md).

### Pose Adapter System

The application includes a pose adapter that converts MediaPipe Pose Landmarker data into VRM bone rotations:

- **MediaPipe Pose Landmarker** outputs 33 3D landmarks in world coordinates (meters)
- **PoseAdapter** converts landmarks to bone rotations for VRM humanoid bones
- **Bone Rotation System** applies calculated rotations to VRM bone Transform components

#### Supported Bones

The pose adapter currently supports upper body tracking:

- **Arms**: `leftUpperArm`, `leftLowerArm`, `rightUpperArm`, `rightLowerArm`
- **Torso**: `chest`

The system uses world coordinates for accurate 3D positioning and includes confidence-based filtering to ensure stable tracking. Only landmarks with visibility > 0.5 are used for bone rotation calculations.

For more details on the pose adapter, see [crates/pose_adapter/README.md](crates/pose_adapter/README.md).

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