# pose_adapter

Maps MediaPipe pose landmarks to VRM 1.0 humanoid bone rotations.

## Overview

This crate provides the `MediaPipePoseAdapter` which converts MediaPipe's 33 3D pose world landmarks into bone rotations suitable for VRM humanoid models.

## MediaPipe Pose Landmarks

MediaPipe provides 33 pose landmarks:
- 0-10: Face landmarks (nose, eyes, ears, mouth)
- 11-16: Upper body (shoulders, elbows, wrists)
- 17-22: Hands (pinky, index, thumb)
- 23-28: Lower body (hips, knees, ankles)
- 29-32: Feet (heel, foot index)

## VRM Humanoid Bones

Standard VRM 1.0 humanoid bones include:
- Spine: hips, spine, chest, upperChest, neck, head
- Arms: leftShoulder, leftUpperArm, leftLowerArm, leftHand
- Arms: rightShoulder, rightUpperArm, rightLowerArm, rightHand
- Legs: leftUpperLeg, leftLowerLeg, leftFoot
- Legs: rightUpperLeg, rightLowerLeg, rightFoot

## Mapping Strategy

The adapter maps MediaPipe landmarks to VRM bones by:
1. Extracting relevant landmark positions
2. Computing bone directions from parent to child joints
3. Calculating rotation quaternions to align bones with tracked positions
4. Outputting bone rotations ready to apply to VRM Transform components
