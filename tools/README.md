# MediaPipe Face Tracker

This directory contains the Python face tracking implementation using MediaPipe Face Landmarker.

## Setup

### 1. Install Python Dependencies

```bash
cd tools
pip install -r requirements.txt
```

### 2. Download the MediaPipe Model

Download the face landmarker model with blendshapes support:

```bash
cd tools
wget -O face_landmarker.task https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
```

Or download manually from:
https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task

## Usage

The tracker is automatically started by the Rust application. It outputs face blendshapes in JSON format to stdout.

To test the tracker manually:

```bash
cd tools
python3 mediapipe_tracker.py
```

Make sure you have a webcam connected and the model file downloaded.

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
  }
}
```

- `ts`: Timestamp in seconds (float)
- `blendshapes`: Dictionary of blendshape names and their values (0.0 to 1.0)

## Blendshapes

MediaPipe Face Landmarker provides up to 52 blendshapes that describe facial expressions, including:

- Eye movements (eyeBlinkLeft, eyeBlinkRight, eyeLookUpLeft, eyeLookDownRight, etc.)
- Mouth movements (jawOpen, mouthClose, mouthSmileLeft, mouthSmileRight, etc.)
- Eyebrow movements (browDownLeft, browDownRight, browInnerUp, etc.)
- Cheek movements (cheekPuff, cheekSquintLeft, cheekSquintRight, etc.)
- And many more...

For a complete list of blendshapes, see the MediaPipe documentation:
https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker
