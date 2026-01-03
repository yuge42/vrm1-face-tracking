import json
import time
import sys
import cv2
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision

# Path to the MediaPipe model file
# Download from: https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
MODEL_PATH = "tools/face_landmarker.task"

# Maximum consecutive frame read failures before exiting
MAX_FRAME_FAILURES = 30

def main():
    # Initialize MediaPipe Face Landmarker
    base_options = python.BaseOptions(model_asset_path=MODEL_PATH)
    options = vision.FaceLandmarkerOptions(
        base_options=base_options,
        output_face_blendshapes=True,
        running_mode=vision.RunningMode.VIDEO,
        num_faces=1
    )
    
    landmarker = vision.FaceLandmarker.create_from_options(options)
    
    # Open webcam
    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        print(json.dumps({"error": "Failed to open webcam"}), file=sys.stderr, flush=True)
        sys.exit(1)
    
    frame_count = 0
    consecutive_failures = 0
    
    try:
        while True:
            success, frame = cap.read()
            if not success:
                consecutive_failures += 1
                if consecutive_failures >= MAX_FRAME_FAILURES:
                    print(json.dumps({"error": f"Failed to read {MAX_FRAME_FAILURES} consecutive frames, exiting"}), file=sys.stderr, flush=True)
                    sys.exit(1)
                continue
            
            # Reset failure counter on successful read
            consecutive_failures = 0
            
            # Convert BGR to RGB
            rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            
            # Create MediaPipe Image
            mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=rgb_frame)
            
            # Use frame count as timestamp to ensure monotonically increasing values
            # MediaPipe VIDEO mode requires timestamps in milliseconds
            timestamp_ms = frame_count
            
            # Detect face landmarks and blendshapes
            result = landmarker.detect_for_video(mp_image, timestamp_ms)
            
            # Extract blendshapes
            blendshapes = {}
            if result.face_blendshapes and len(result.face_blendshapes) > 0:
                for blendshape in result.face_blendshapes[0]:
                    blendshapes[blendshape.category_name] = blendshape.score
            
            # Output frame data in the expected format
            output = {
                "ts": time.time(),
                "blendshapes": blendshapes
            }
            
            print(json.dumps(output), flush=True)
            
            frame_count += 1
            
    except KeyboardInterrupt:
        pass
    finally:
        cap.release()
        landmarker.close()

if __name__ == "__main__":
    main()
