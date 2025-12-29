import json
import time
import sys

i = 0

while True:
    frame = {
        "ts": time.time(),
        "blendshapes": {
            "eyeBlinkLeft": (i % 100) / 100.0,
            "eyeBlinkRight": (i % 80) / 80.0,
            "jawOpen": 0.3,
        }
    }

    print(json.dumps(frame), flush=True)
    i += 1
    time.sleep(0.1)  # 10 FPS
