# MediaPipe Face and Pose Tracker

This directory contains the Python tracking implementation using MediaPipe Face Landmarker and Pose Landmarker.

## Setup

### 1. Create Virtual Environment (Recommended)

```bash
# From the project root
python3 -m venv .venv

# Activate virtual environment
# On Linux/macOS:
source .venv/bin/activate
# On Windows:
# .venv\Scripts\activate
```

### 2. Install Python Dependencies

```bash
# From the project root
pip install -r tools/requirements.txt
```

### 3. Download the MediaPipe Models

Download the face landmarker model with blendshapes support:

```bash
# From the project root
cd tools

# Face Landmarker model
wget -O face_landmarker.task https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task

# Pose Landmarker model
wget -O pose_landmarker_full.task https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/1/pose_landmarker_full.task

cd ..
```

Or download manually from:
- Face Landmarker: https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
- Pose Landmarker: https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/1/pose_landmarker_full.task

## Usage

The tracker is automatically started by the Rust application. It outputs face blendshapes and pose landmarks in JSON format to stdout.

To test the tracker manually:

```bash
# From the project root, make sure virtual environment is activated
source .venv/bin/activate  # or .venv\Scripts\activate on Windows
python tools/mediapipe_tracker.py
```

Make sure you have a webcam connected and both model files downloaded.

## Output Format

The tracker outputs one JSON object per line with the following structure:

```json
{
  "ts": 1234567890.123,
  "blendshapes": {
    "eyeBlinkLeft": 0.5,
    "eyeBlinkRight": 0.3,
    "jawOpen": 0.2,
    ...
  },
  "pose_landmarks": [
    {
      "x": 0.5,
      "y": 0.5,
      "z": -0.1,
      "visibility": 0.9,
      "presence": 0.95
    },
    ...
  ],
  "pose_world_landmarks": [
    {
      "x": 0.123,
      "y": 0.456,
      "z": -0.789,
      "visibility": 0.9,
      "presence": 0.95
    },
    ...
  ]
}
```

- `ts`: Timestamp in seconds (float)
- `blendshapes`: Dictionary of blendshape names and their values (0.0 to 1.0)
- `pose_landmarks`: Array of 33 pose landmarks in image coordinates (normalized 0.0 to 1.0)
- `pose_world_landmarks`: Array of 33 pose landmarks in real-world coordinates (meters, relative to hip center)

## Blendshapes

MediaPipe Face Landmarker provides up to 52 blendshapes that describe facial expressions, including:

- Eye movements (eyeBlinkLeft, eyeBlinkRight, eyeLookUpLeft, eyeLookDownRight, etc.)
- Mouth movements (jawOpen, mouthClose, mouthSmileLeft, mouthSmileRight, etc.)
- Eyebrow movements (browDownLeft, browDownRight, browInnerUp, etc.)
- Cheek movements (cheekPuff, cheekSquintLeft, cheekSquintRight, etc.)
- And many more...

For a complete list of blendshapes, see the MediaPipe documentation:
https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker

## Pose Landmarks

MediaPipe Pose Landmarker provides 33 3D landmarks representing the human body pose in two coordinate systems:

### Image Landmarks (`pose_landmarks`)
Normalized coordinates (0.0 to 1.0) relative to image dimensions:
- `x`, `y`: Normalized coordinates relative to image width/height
- `z`: Depth coordinate (roughly in the same scale as x)
- `visibility`: Likelihood that the landmark is visible in the image
- `presence`: Likelihood that the landmark is present in the scene

### World Landmarks (`pose_world_landmarks`)
Real-world 3D coordinates in meters, relative to the hip center:
- `x`, `y`, `z`: 3D coordinates in meters
- `visibility`: Likelihood that the landmark is visible in the image
- `presence`: Likelihood that the landmark is present in the scene

World landmarks are useful for estimating real-world distances and positions, making them ideal for:
- Measuring body dimensions
- Tracking physical movements in space
- Calculating joint angles
- Estimating real-world scale

### Landmark Indices

- **Upper Body (0-16)**: Nose, eyes, ears, mouth, shoulders, elbows, wrists, etc.
- **Torso (11-12, 23-24)**: Left/right shoulder, left/right hip
- **Lower Body (23-32)**: Hips, knees, ankles, feet, etc.

For a complete list and visualization of pose landmarks, see:
https://ai.google.dev/edge/mediapipe/solutions/vision/pose_landmarker
