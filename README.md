# VRM1 Face Tracking

Real-time face tracking for VRM models using MediaPipe Face Landmarker.

## Setup

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

The application will automatically start the Python face tracker and display the blendshape data.

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