//! Adapter for mapping MediaPipe pose landmarks to VRM bone rotations.
//!
//! This module provides functionality to convert MediaPipe's 33 3D pose world landmarks
//! into bone rotations suitable for applying to VRM 1.0 humanoid models.

use glam::{Quat, Vec3};

/// MediaPipe pose landmark indices (33 total)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum PoseLandmarkIndex {
    // Face (0-10)
    Nose = 0,
    LeftEyeInner = 1,
    LeftEye = 2,
    LeftEyeOuter = 3,
    RightEyeInner = 4,
    RightEye = 5,
    RightEyeOuter = 6,
    LeftEar = 7,
    RightEar = 8,
    MouthLeft = 9,
    MouthRight = 10,

    // Upper body (11-16)
    LeftShoulder = 11,
    RightShoulder = 12,
    LeftElbow = 13,
    RightElbow = 14,
    LeftWrist = 15,
    RightWrist = 16,

    // Hands (17-22)
    LeftPinky = 17,
    RightPinky = 18,
    LeftIndex = 19,
    RightIndex = 20,
    LeftThumb = 21,
    RightThumb = 22,

    // Lower body (23-28)
    LeftHip = 23,
    RightHip = 24,
    LeftKnee = 25,
    RightKnee = 26,
    LeftAnkle = 27,
    RightAnkle = 28,

    // Feet (29-32)
    LeftHeel = 29,
    RightHeel = 30,
    LeftFootIndex = 31,
    RightFootIndex = 32,
}

/// A 3D pose landmark in world coordinates (meters)
#[derive(Debug, Clone, Copy)]
pub struct PoseWorldLandmark {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visibility: f32,
}

impl PoseWorldLandmark {
    /// Convert to glam Vec3
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

/// Represents a bone rotation for a VRM humanoid bone
#[derive(Debug, Clone)]
pub struct VrmBoneRotation {
    /// VRM bone name (e.g., "leftUpperArm", "rightLowerLeg")
    pub bone_name: String,
    /// Rotation quaternion to apply to the bone
    pub rotation: Quat,
    /// Confidence/visibility score (0.0-1.0)
    pub confidence: f32,
}

impl VrmBoneRotation {
    pub fn new(bone_name: impl Into<String>, rotation: Quat, confidence: f32) -> Self {
        Self {
            bone_name: bone_name.into(),
            rotation,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Adapter for converting MediaPipe pose landmarks to VRM bone rotations
pub struct MediaPipePoseAdapter;

impl MediaPipePoseAdapter {
    /// Convert pose world landmarks to VRM bone rotations
    ///
    /// Takes 33 MediaPipe world landmarks and outputs bone rotations for upper body bones.
    /// Returns an empty vector if the landmarks are insufficient or invalid.
    pub fn landmarks_to_bone_rotations(landmarks: &[PoseWorldLandmark]) -> Vec<VrmBoneRotation> {
        if landmarks.len() < 33 {
            return Vec::new();
        }

        let mut rotations = Vec::new();

        // Process upper body bones only (shoulders, elbows, wrists)
        // We focus on the most reliable upper body tracking for now

        // Left arm chain: shoulder -> elbow -> wrist
        if let Some(rotation) = Self::compute_left_upper_arm_rotation(landmarks) {
            rotations.push(rotation);
        }
        if let Some(rotation) = Self::compute_left_lower_arm_rotation(landmarks) {
            rotations.push(rotation);
        }

        // Right arm chain: shoulder -> elbow -> wrist
        if let Some(rotation) = Self::compute_right_upper_arm_rotation(landmarks) {
            rotations.push(rotation);
        }
        if let Some(rotation) = Self::compute_right_lower_arm_rotation(landmarks) {
            rotations.push(rotation);
        }

        // Spine/chest rotation based on shoulders
        if let Some(rotation) = Self::compute_chest_rotation(landmarks) {
            rotations.push(rotation);
        }

        rotations
    }

    /// Compute rotation for left upper arm (shoulder to elbow)
    fn compute_left_upper_arm_rotation(landmarks: &[PoseWorldLandmark]) -> Option<VrmBoneRotation> {
        let shoulder = landmarks[PoseLandmarkIndex::LeftShoulder as usize];
        let elbow = landmarks[PoseLandmarkIndex::LeftElbow as usize];

        // Check visibility threshold
        if shoulder.visibility < 0.5 || elbow.visibility < 0.5 {
            return None;
        }

        let bone_dir = (elbow.to_vec3() - shoulder.to_vec3()).normalize();
        // Default T-pose direction for left upper arm is roughly -X (left)
        let default_dir = Vec3::new(-1.0, 0.0, 0.0);

        let rotation = rotation_between_vectors(default_dir, bone_dir);
        let confidence = (shoulder.visibility + elbow.visibility) / 2.0;

        Some(VrmBoneRotation::new("leftUpperArm", rotation, confidence))
    }

    /// Compute rotation for left lower arm (elbow to wrist)
    fn compute_left_lower_arm_rotation(landmarks: &[PoseWorldLandmark]) -> Option<VrmBoneRotation> {
        let elbow = landmarks[PoseLandmarkIndex::LeftElbow as usize];
        let wrist = landmarks[PoseLandmarkIndex::LeftWrist as usize];

        if elbow.visibility < 0.5 || wrist.visibility < 0.5 {
            return None;
        }

        let bone_dir = (wrist.to_vec3() - elbow.to_vec3()).normalize();
        let default_dir = Vec3::new(-1.0, 0.0, 0.0);

        let rotation = rotation_between_vectors(default_dir, bone_dir);
        let confidence = (elbow.visibility + wrist.visibility) / 2.0;

        Some(VrmBoneRotation::new("leftLowerArm", rotation, confidence))
    }

    /// Compute rotation for right upper arm (shoulder to elbow)
    fn compute_right_upper_arm_rotation(
        landmarks: &[PoseWorldLandmark],
    ) -> Option<VrmBoneRotation> {
        let shoulder = landmarks[PoseLandmarkIndex::RightShoulder as usize];
        let elbow = landmarks[PoseLandmarkIndex::RightElbow as usize];

        if shoulder.visibility < 0.5 || elbow.visibility < 0.5 {
            return None;
        }

        let bone_dir = (elbow.to_vec3() - shoulder.to_vec3()).normalize();
        // Default T-pose direction for right upper arm is roughly +X (right)
        let default_dir = Vec3::new(1.0, 0.0, 0.0);

        let rotation = rotation_between_vectors(default_dir, bone_dir);
        let confidence = (shoulder.visibility + elbow.visibility) / 2.0;

        Some(VrmBoneRotation::new("rightUpperArm", rotation, confidence))
    }

    /// Compute rotation for right lower arm (elbow to wrist)
    fn compute_right_lower_arm_rotation(
        landmarks: &[PoseWorldLandmark],
    ) -> Option<VrmBoneRotation> {
        let elbow = landmarks[PoseLandmarkIndex::RightElbow as usize];
        let wrist = landmarks[PoseLandmarkIndex::RightWrist as usize];

        if elbow.visibility < 0.5 || wrist.visibility < 0.5 {
            return None;
        }

        let bone_dir = (wrist.to_vec3() - elbow.to_vec3()).normalize();
        let default_dir = Vec3::new(1.0, 0.0, 0.0);

        let rotation = rotation_between_vectors(default_dir, bone_dir);
        let confidence = (elbow.visibility + wrist.visibility) / 2.0;

        Some(VrmBoneRotation::new("rightLowerArm", rotation, confidence))
    }

    /// Compute rotation for chest/upper body based on shoulder orientation
    fn compute_chest_rotation(landmarks: &[PoseWorldLandmark]) -> Option<VrmBoneRotation> {
        let left_shoulder = landmarks[PoseLandmarkIndex::LeftShoulder as usize];
        let right_shoulder = landmarks[PoseLandmarkIndex::RightShoulder as usize];

        if left_shoulder.visibility < 0.5 || right_shoulder.visibility < 0.5 {
            return None;
        }

        // Compute the shoulder line direction
        let shoulder_dir = (right_shoulder.to_vec3() - left_shoulder.to_vec3()).normalize();
        // Default shoulder line in T-pose is along +X
        let default_dir = Vec3::new(1.0, 0.0, 0.0);

        let rotation = rotation_between_vectors(default_dir, shoulder_dir);
        let confidence = (left_shoulder.visibility + right_shoulder.visibility) / 2.0;

        Some(VrmBoneRotation::new("chest", rotation, confidence))
    }
}

/// Compute the shortest rotation between two normalized vectors
///
/// Returns a quaternion that rotates from `from` to `to`.
/// Both vectors should be normalized.
fn rotation_between_vectors(from: Vec3, to: Vec3) -> Quat {
    // Handle the case where vectors are parallel or anti-parallel
    let dot = from.dot(to);

    if dot > 0.9999 {
        // Vectors are nearly parallel, no rotation needed
        return Quat::IDENTITY;
    }

    if dot < -0.9999 {
        // Vectors are nearly anti-parallel, rotate 180° around any perpendicular axis
        let axis = if from.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        return Quat::from_axis_angle(axis.normalize(), std::f32::consts::PI);
    }

    // Standard case: compute rotation axis and angle
    let axis = from.cross(to).normalize();
    let angle = dot.acos();

    Quat::from_axis_angle(axis, angle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_between_vectors_identity() {
        let from = Vec3::new(1.0, 0.0, 0.0);
        let to = Vec3::new(1.0, 0.0, 0.0);
        let rotation = rotation_between_vectors(from, to);
        assert!((rotation.w - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rotation_between_vectors_90deg() {
        let from = Vec3::new(1.0, 0.0, 0.0);
        let to = Vec3::new(0.0, 1.0, 0.0);
        let rotation = rotation_between_vectors(from, to);

        // Apply rotation to from vector
        let result = rotation * from;

        // Should be close to 'to' vector
        assert!((result - to).length() < 0.001);
    }

    #[test]
    fn test_adapter_insufficient_landmarks() {
        let landmarks = vec![];
        let rotations = MediaPipePoseAdapter::landmarks_to_bone_rotations(&landmarks);
        assert_eq!(rotations.len(), 0);
    }

    #[test]
    fn test_adapter_with_valid_landmarks() {
        // Create 33 landmarks with some visible upper body landmarks
        let mut landmarks = vec![
            PoseWorldLandmark {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                visibility: 0.0,
            };
            33
        ];

        // Set visible left arm landmarks
        landmarks[PoseLandmarkIndex::LeftShoulder as usize] = PoseWorldLandmark {
            x: -0.2,
            y: 0.5,
            z: 0.0,
            visibility: 0.9,
        };
        landmarks[PoseLandmarkIndex::LeftElbow as usize] = PoseWorldLandmark {
            x: -0.4,
            y: 0.3,
            z: 0.0,
            visibility: 0.9,
        };
        landmarks[PoseLandmarkIndex::LeftWrist as usize] = PoseWorldLandmark {
            x: -0.5,
            y: 0.1,
            z: 0.0,
            visibility: 0.9,
        };

        // Set visible right arm landmarks
        landmarks[PoseLandmarkIndex::RightShoulder as usize] = PoseWorldLandmark {
            x: 0.2,
            y: 0.5,
            z: 0.0,
            visibility: 0.9,
        };
        landmarks[PoseLandmarkIndex::RightElbow as usize] = PoseWorldLandmark {
            x: 0.4,
            y: 0.3,
            z: 0.0,
            visibility: 0.9,
        };
        landmarks[PoseLandmarkIndex::RightWrist as usize] = PoseWorldLandmark {
            x: 0.5,
            y: 0.1,
            z: 0.0,
            visibility: 0.9,
        };

        let rotations = MediaPipePoseAdapter::landmarks_to_bone_rotations(&landmarks);

        // Should get rotations for: leftUpperArm, leftLowerArm, rightUpperArm, rightLowerArm, chest
        assert_eq!(rotations.len(), 5);

        // Check that we have the expected bone names
        let bone_names: Vec<&str> = rotations.iter().map(|r| r.bone_name.as_str()).collect();
        assert!(bone_names.contains(&"leftUpperArm"));
        assert!(bone_names.contains(&"leftLowerArm"));
        assert!(bone_names.contains(&"rightUpperArm"));
        assert!(bone_names.contains(&"rightLowerArm"));
        assert!(bone_names.contains(&"chest"));
    }
}
