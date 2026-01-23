import json
import time
import sys
import os
import cv2
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision

# Path to the MediaPipe model files
# Download from: https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
FACE_MODEL_PATH = "tools/face_landmarker.task"
# Download from: https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/1/pose_landmarker_full.task
POSE_MODEL_PATH = "tools/pose_landmarker_full.task"

# Maximum consecutive frame read failures before exiting
MAX_FRAME_FAILURES = 30

def main():
    # Check if face model file exists
    if not os.path.exists(FACE_MODEL_PATH):
        print(json.dumps({
            "error": f"Face model file not found at {FACE_MODEL_PATH}. Please download it from: https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task"
        }), file=sys.stderr, flush=True)
        sys.exit(1)
    
    # Check if pose model file exists
    if not os.path.exists(POSE_MODEL_PATH):
        print(json.dumps({
            "error": f"Pose model file not found at {POSE_MODEL_PATH}. Please download it from: https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/1/pose_landmarker_full.task"
        }), file=sys.stderr, flush=True)
        sys.exit(1)
    
    # Initialize MediaPipe Face Landmarker
    face_base_options = python.BaseOptions(model_asset_path=FACE_MODEL_PATH)
    face_options = vision.FaceLandmarkerOptions(
        base_options=face_base_options,
        output_face_blendshapes=True,
        running_mode=vision.RunningMode.VIDEO,
        num_faces=1
    )
    
    face_landmarker = vision.FaceLandmarker.create_from_options(face_options)
    
    # Initialize MediaPipe Pose Landmarker
    pose_base_options = python.BaseOptions(model_asset_path=POSE_MODEL_PATH)
    pose_options = vision.PoseLandmarkerOptions(
        base_options=pose_base_options,
        running_mode=vision.RunningMode.VIDEO
    )
    
    pose_landmarker = vision.PoseLandmarker.create_from_options(pose_options)
    
    # Open webcam
    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        print(json.dumps({
            "error": "Failed to open webcam (device index 0). Please check that a webcam is connected and accessible."
        }), file=sys.stderr, flush=True)
        sys.exit(1)
    
    frame_count = 0
    consecutive_failures = 0
    
    try:
        while True:
            success, frame = cap.read()
            if not success:
                consecutive_failures += 1
                if consecutive_failures >= MAX_FRAME_FAILURES:
                    print(json.dumps({
                        "error": f"Failed to read {MAX_FRAME_FAILURES} consecutive frames. Possible causes: webcam disconnected, permission denied, or device error."
                    }), file=sys.stderr, flush=True)
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
            face_result = face_landmarker.detect_for_video(mp_image, timestamp_ms)
            
            # Extract blendshapes
            blendshapes = {}
            if face_result.face_blendshapes and len(face_result.face_blendshapes) > 0:
                for blendshape in face_result.face_blendshapes[0]:
                    blendshapes[blendshape.category_name] = blendshape.score
            
            # Detect pose landmarks
            pose_result = pose_landmarker.detect_for_video(mp_image, timestamp_ms)
            
            # Extract pose landmarks (33 3D landmarks in image coordinates)
            pose_landmarks = []
            if pose_result.pose_landmarks and len(pose_result.pose_landmarks) > 0:
                for landmark in pose_result.pose_landmarks[0]:
                    pose_landmarks.append({
                        "x": landmark.x,
                        "y": landmark.y,
                        "z": landmark.z,
                        "visibility": landmark.visibility,
                        "presence": landmark.presence
                    })
            
            # Extract pose world landmarks (33 3D landmarks in real-world coordinates)
            pose_world_landmarks = []
            if pose_result.pose_world_landmarks and len(pose_result.pose_world_landmarks) > 0:
                for landmark in pose_result.pose_world_landmarks[0]:
                    pose_world_landmarks.append({
                        "x": landmark.x,
                        "y": landmark.y,
                        "z": landmark.z,
                        "visibility": landmark.visibility,
                        "presence": landmark.presence
                    })
            
            # Output frame data in the expected format
            output = {
                "ts": time.time(),
                "blendshapes": blendshapes,
                "pose_landmarks": pose_landmarks,
                "pose_world_landmarks": pose_world_landmarks
            }
            
            print(json.dumps(output), flush=True)
            
            frame_count += 1
            
    except KeyboardInterrupt:
        pass
    finally:
        cap.release()
        face_landmarker.close()
        pose_landmarker.close()

if __name__ == "__main__":
    main()
