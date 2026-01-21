//! VRM 1.0 extension data structures.
//!
//! These structures map directly to the VRM 1.0 specification:
//! <https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm-1.0>

use serde::Deserialize;
use std::collections::HashMap;

/// VRM 1.0 metadata (from VRMC_vrm extension).
///
/// See: <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/meta.md>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmMeta {
    /// Name of the VRM model
    pub name: String,

    /// Version of the VRM model
    #[serde(default)]
    pub version: String,

    /// Authors of the VRM model
    #[serde(default)]
    pub authors: Vec<String>,

    /// Copyright holder
    #[serde(default)]
    pub copyright_information: String,

    /// Contact information
    #[serde(default)]
    pub contact_information: String,

    /// References (URLs, etc.)
    #[serde(default)]
    pub references: Vec<String>,

    /// Third party licenses
    #[serde(default)]
    pub third_party_licenses: String,

    /// Thumbnail image index (glTF image index)
    pub thumbnail_image: Option<usize>,

    /// License URL
    #[serde(default)]
    pub license_url: String,

    /// Avatar permission
    #[serde(default)]
    pub avatar_permission: String,

    /// Commercial usage permission
    #[serde(default)]
    pub commercial_usage: String,

    /// Credit notation requirement
    #[serde(default)]
    pub credit_notation: String,

    /// Modification permission
    #[serde(default)]
    pub modification: String,

    /// Allow redistribution of modifications
    #[serde(default)]
    pub allow_redistribution: bool,
}

impl Default for VrmMeta {
    fn default() -> Self {
        Self {
            name: "Unnamed VRM".to_string(),
            version: String::new(),
            authors: Vec::new(),
            copyright_information: String::new(),
            contact_information: String::new(),
            references: Vec::new(),
            third_party_licenses: String::new(),
            thumbnail_image: None,
            license_url: String::new(),
            avatar_permission: String::new(),
            commercial_usage: String::new(),
            credit_notation: String::new(),
            modification: String::new(),
            allow_redistribution: false,
        }
    }
}

/// VRM 1.0 humanoid bone mapping.
///
/// See: <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/humanoid.md>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmHumanoid {
    /// Map of human bone names to node indices
    pub human_bones: HashMap<String, VrmHumanBone>,
}

/// A single human bone mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmHumanBone {
    /// glTF node index
    pub node: usize,
}

/// VRM 1.0 expression (blend shape preset).
///
/// See: <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmExpression {
    /// Morph target bindings for this expression
    #[serde(default)]
    pub morph_target_binds: Vec<VrmMorphTargetBind>,

    /// Material color bindings (not used for face tracking)
    #[serde(default)]
    pub material_color_binds: Vec<VrmMaterialColorBind>,

    /// Texture transform bindings (not used for face tracking)
    #[serde(default)]
    pub texture_transform_binds: Vec<VrmTextureTransformBind>,

    /// Whether this expression can be applied while looking at something
    #[serde(default)]
    pub is_binary: bool,

    /// Override mode for blending
    #[serde(default)]
    pub override_blink: String,

    #[serde(default)]
    pub override_look_at: String,

    #[serde(default)]
    pub override_mouth: String,
}

/// Morph target binding for an expression.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmMorphTargetBind {
    /// glTF node index
    pub node: usize,

    /// Morph target index within the node's mesh
    pub index: usize,

    /// Weight for this morph target (0.0 to 1.0)
    pub weight: f32,
}

/// Material color binding (for material property animations).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmMaterialColorBind {
    /// Material index
    pub material: usize,

    /// Property type
    #[serde(rename = "type")]
    pub property_type: String,

    /// Target color value
    pub target_value: [f32; 4],
}

/// Texture transform binding (for UV animations).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmTextureTransformBind {
    /// Material index
    pub material: usize,

    /// Scale value
    pub scale: [f32; 2],

    /// Offset value
    pub offset: [f32; 2],
}

/// VRM 1.0 look-at configuration.
///
/// See: <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/lookAt.md>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmLookAt {
    /// Offset from head bone
    #[serde(default)]
    pub offset_from_head_bone: [f32; 3],

    /// Look-at type
    #[serde(rename = "type", default)]
    pub look_at_type: String,

    /// Range map for horizontal inner movement
    pub range_map_horizontal_inner: Option<VrmLookAtRangeMap>,

    /// Range map for horizontal outer movement
    pub range_map_horizontal_outer: Option<VrmLookAtRangeMap>,

    /// Range map for vertical down movement
    pub range_map_vertical_down: Option<VrmLookAtRangeMap>,

    /// Range map for vertical up movement
    pub range_map_vertical_up: Option<VrmLookAtRangeMap>,
}

/// Range map for look-at angles.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmLookAtRangeMap {
    /// Input range maximum
    pub input_max_value: f32,

    /// Output scale
    pub output_scale: f32,
}

/// First person view configuration.
///
/// See: <https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/firstPerson.md>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmFirstPerson {
    /// Mesh annotations for first-person view
    #[serde(default)]
    pub mesh_annotations: Vec<VrmFirstPersonMeshAnnotation>,
}

/// Mesh annotation for first-person rendering.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmFirstPersonMeshAnnotation {
    /// Node index
    pub node: usize,

    /// First person flag
    #[serde(rename = "type")]
    pub annotation_type: String,
}

/// The root VRMC_vrm extension object.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmcVrmExtension {
    /// Spec version (should be "1.0")
    pub spec_version: String,

    /// VRM metadata
    pub meta: VrmMeta,

    /// Humanoid bone mapping
    pub humanoid: Option<VrmHumanoid>,

    /// Expressions (preset and custom)
    #[serde(default)]
    pub expressions: VrmExpressions,

    /// Look-at configuration
    pub look_at: Option<VrmLookAt>,

    /// First person configuration
    pub first_person: Option<VrmFirstPerson>,
}

/// Expressions container with preset and custom expressions.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmExpressions {
    /// Preset expressions (standard VRM expression names)
    #[serde(default)]
    pub preset: HashMap<String, VrmExpression>,

    /// Custom expressions (user-defined)
    #[serde(default)]
    pub custom: HashMap<String, VrmExpression>,
}
