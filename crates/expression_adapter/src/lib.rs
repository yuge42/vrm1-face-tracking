use std::collections::HashMap;

/// Represents a VRM 1.0 expression preset name
///
/// Based on the VRM 1.0 specification:
/// https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VrmExpressionPreset {
    // Emotions
    Happy,
    Angry,
    Sad,
    Relaxed,
    Surprised,

    // Lip Sync (procedural)
    Aa,
    Ih,
    Ou,
    Ee,
    Oh,

    // Blink (procedural)
    Blink,
    BlinkLeft,
    BlinkRight,

    // Gaze (procedural)
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // Other
    Neutral,
}

impl VrmExpressionPreset {
    /// Get the canonical VRM expression name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            // Emotions
            VrmExpressionPreset::Happy => "happy",
            VrmExpressionPreset::Angry => "angry",
            VrmExpressionPreset::Sad => "sad",
            VrmExpressionPreset::Relaxed => "relaxed",
            VrmExpressionPreset::Surprised => "surprised",

            // Lip Sync
            VrmExpressionPreset::Aa => "aa",
            VrmExpressionPreset::Ih => "ih",
            VrmExpressionPreset::Ou => "ou",
            VrmExpressionPreset::Ee => "ee",
            VrmExpressionPreset::Oh => "oh",

            // Blink
            VrmExpressionPreset::Blink => "blink",
            VrmExpressionPreset::BlinkLeft => "blinkLeft",
            VrmExpressionPreset::BlinkRight => "blinkRight",

            // Gaze
            VrmExpressionPreset::LookUp => "lookUp",
            VrmExpressionPreset::LookDown => "lookDown",
            VrmExpressionPreset::LookLeft => "lookLeft",
            VrmExpressionPreset::LookRight => "lookRight",

            // Other
            VrmExpressionPreset::Neutral => "neutral",
        }
    }
}

/// A VRM expression with its weight value
#[derive(Debug, Clone)]
pub struct VrmExpression {
    pub preset: VrmExpressionPreset,
    pub weight: f32,
}

impl VrmExpression {
    pub fn new(preset: VrmExpressionPreset, weight: f32) -> Self {
        Self {
            preset,
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

/// Trait for converting raw tracker blendshape data to VRM expressions
///
/// This trait provides the interface for converting tracker-specific blendshape data
/// (e.g., ARKit-style blendshapes from MediaPipe) into VRM 1.0 expression presets.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use expression_adapter::{BlendshapeToExpression, VrmExpression, VrmExpressionPreset};
///
/// struct MyAdapter;
///
/// impl BlendshapeToExpression for MyAdapter {
///     fn to_vrm_expressions(&self, raw_blendshapes: &HashMap<String, f32>) -> Vec<VrmExpression> {
///         let mut expressions = Vec::new();
///         
///         // Simple direct mapping for blink
///         if let Some(&value) = raw_blendshapes.get("eyeBlinkLeft") {
///             expressions.push(VrmExpression::new(VrmExpressionPreset::BlinkLeft, value));
///         }
///         
///         expressions
///     }
/// }
/// ```
pub trait BlendshapeToExpression {
    /// Convert raw tracker blendshapes to VRM expressions
    ///
    /// # Arguments
    ///
    /// * `raw_blendshapes` - Map of blendshape names to their values (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A vector of VRM expressions with their weights
    fn to_vrm_expressions(&self, raw_blendshapes: &HashMap<String, f32>) -> Vec<VrmExpression>;
}

/// Default adapter for ARKit-style blendshapes (e.g., from MediaPipe)
///
/// This adapter provides sensible default mappings from the 52 ARKit blendshapes
/// to VRM 1.0 expression presets. The mapping includes:
///
/// - Direct mappings for blink and eye gaze
/// - Weighted combinations for emotions (e.g., smile → happy)
/// - Mouth shape to phoneme mappings for lip sync
pub struct ArkitToVrmAdapter;

impl BlendshapeToExpression for ArkitToVrmAdapter {
    fn to_vrm_expressions(&self, raw_blendshapes: &HashMap<String, f32>) -> Vec<VrmExpression> {
        let mut expressions = Vec::new();

        // Helper to get blendshape value
        let get = |name: &str| -> f32 { raw_blendshapes.get(name).copied().unwrap_or(0.0) };

        // Blink - direct mapping
        let blink_left = get("eyeBlinkLeft");
        let blink_right = get("eyeBlinkRight");

        if blink_left > 0.0 {
            expressions.push(VrmExpression::new(
                VrmExpressionPreset::BlinkLeft,
                blink_left,
            ));
        }
        if blink_right > 0.0 {
            expressions.push(VrmExpression::new(
                VrmExpressionPreset::BlinkRight,
                blink_right,
            ));
        }

        // Combined blink (average of both eyes)
        let blink = (blink_left + blink_right) * 0.5;
        if blink > 0.0 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Blink, blink));
        }

        // Eye gaze - direct mapping
        let look_up = (get("eyeLookUpLeft") + get("eyeLookUpRight")) * 0.5;
        let look_down = (get("eyeLookDownLeft") + get("eyeLookDownRight")) * 0.5;
        let look_left = (get("eyeLookInLeft") + get("eyeLookOutRight")) * 0.5;
        let look_right = (get("eyeLookOutLeft") + get("eyeLookInRight")) * 0.5;

        if look_up > 0.0 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::LookUp, look_up));
        }
        if look_down > 0.0 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::LookDown, look_down));
        }
        if look_left > 0.0 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::LookLeft, look_left));
        }
        if look_right > 0.0 {
            expressions.push(VrmExpression::new(
                VrmExpressionPreset::LookRight,
                look_right,
            ));
        }

        // Emotions - weighted combinations
        let smile = (get("mouthSmileLeft") + get("mouthSmileRight")) * 0.5;
        if smile > 0.3 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Happy, smile));
        }

        let frown = (get("mouthFrownLeft") + get("mouthFrownRight")) * 0.5;
        if frown > 0.3 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Sad, frown));
        }

        // Lip sync - map mouth shapes to phonemes
        // This is a simplified mapping; more sophisticated systems would use
        // actual speech recognition or audio analysis
        let jaw_open = get("jawOpen");
        let mouth_funnel = get("mouthFunnel");
        let mouth_pucker = get("mouthPucker");

        // "aa" - open mouth (as in "father")
        if jaw_open > 0.5 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Aa, jaw_open));
        }

        // "ou" - rounded lips (as in "boot")
        if mouth_pucker > 0.5 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Ou, mouth_pucker));
        }

        // "oh" - open rounded (as in "boat")
        if mouth_funnel > 0.5 {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Oh, mouth_funnel));
        }

        expressions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrm_expression_preset_as_str() {
        assert_eq!(VrmExpressionPreset::Happy.as_str(), "happy");
        assert_eq!(VrmExpressionPreset::Blink.as_str(), "blink");
        assert_eq!(VrmExpressionPreset::Aa.as_str(), "aa");
        assert_eq!(VrmExpressionPreset::LookUp.as_str(), "lookUp");
    }

    #[test]
    fn test_vrm_expression_clamping() {
        let expr = VrmExpression::new(VrmExpressionPreset::Happy, 1.5);
        assert_eq!(expr.weight, 1.0);

        let expr = VrmExpression::new(VrmExpressionPreset::Sad, -0.5);
        assert_eq!(expr.weight, 0.0);
    }

    #[test]
    fn test_arkit_adapter_blink() {
        let adapter = ArkitToVrmAdapter;
        let mut blendshapes = HashMap::new();
        blendshapes.insert("eyeBlinkLeft".to_string(), 0.8);
        blendshapes.insert("eyeBlinkRight".to_string(), 0.9);

        let expressions = adapter.to_vrm_expressions(&blendshapes);

        // Should have BlinkLeft, BlinkRight, and combined Blink
        assert!(
            expressions
                .iter()
                .any(|e| e.preset == VrmExpressionPreset::BlinkLeft)
        );
        assert!(
            expressions
                .iter()
                .any(|e| e.preset == VrmExpressionPreset::BlinkRight)
        );
        assert!(
            expressions
                .iter()
                .any(|e| e.preset == VrmExpressionPreset::Blink)
        );

        let blink = expressions
            .iter()
            .find(|e| e.preset == VrmExpressionPreset::Blink)
            .unwrap();
        assert!((blink.weight - 0.85).abs() < 0.01); // Average of 0.8 and 0.9
    }

    #[test]
    fn test_arkit_adapter_smile() {
        let adapter = ArkitToVrmAdapter;
        let mut blendshapes = HashMap::new();
        blendshapes.insert("mouthSmileLeft".to_string(), 0.7);
        blendshapes.insert("mouthSmileRight".to_string(), 0.7);

        let expressions = adapter.to_vrm_expressions(&blendshapes);

        assert!(
            expressions
                .iter()
                .any(|e| e.preset == VrmExpressionPreset::Happy)
        );
    }

    #[test]
    fn test_arkit_adapter_no_weak_emotions() {
        let adapter = ArkitToVrmAdapter;
        let mut blendshapes = HashMap::new();
        // Weak smile below threshold
        blendshapes.insert("mouthSmileLeft".to_string(), 0.2);
        blendshapes.insert("mouthSmileRight".to_string(), 0.2);

        let expressions = adapter.to_vrm_expressions(&blendshapes);

        // Should not include Happy due to threshold
        assert!(
            !expressions
                .iter()
                .any(|e| e.preset == VrmExpressionPreset::Happy)
        );
    }

    #[test]
    fn test_arkit_adapter_eye_gaze() {
        let adapter = ArkitToVrmAdapter;
        let mut blendshapes = HashMap::new();
        blendshapes.insert("eyeLookUpLeft".to_string(), 0.6);
        blendshapes.insert("eyeLookUpRight".to_string(), 0.4);

        let expressions = adapter.to_vrm_expressions(&blendshapes);

        let look_up = expressions
            .iter()
            .find(|e| e.preset == VrmExpressionPreset::LookUp)
            .unwrap();
        assert!((look_up.weight - 0.5).abs() < 0.01); // Average
    }
}
