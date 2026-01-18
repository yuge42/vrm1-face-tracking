//! Minimal VRM 1.0 glTF parser for live face-tracking applications.
//!
//! This crate provides a lightweight VRM 1.0 loader designed specifically for
//! real-time face tracking use cases. It parses VRM 1.0 extensions from glTF files
//! and exposes them as plain Rust data structures, without any animation or VRMA assumptions.
//!
//! # Architecture
//!
//! ```text
//! glTF (.vrm file)
//!    ↓
//! Bevy's built-in glTF loader (meshes, materials, scenes)
//!    ↓
//! VRM extension parser (this crate)
//!    ↓
//! Plain Rust structs (VRM metadata, expressions, morph targets)
//!    ↓
//! Application systems
//!    ↓
//! MorphWeights manipulation
//! ```

use bevy::prelude::*;
use std::collections::HashMap;

pub mod extensions;
pub mod loader;
pub mod plugin;

pub use extensions::*;
pub use loader::*;
pub use plugin::*;

/// VRM 1.0 asset containing parsed metadata and extension data.
///
/// This asset is created after loading a glTF file and extracting
/// the VRM 1.0 extensions from it.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct VrmAsset {
    /// Handle to the underlying glTF asset loaded by Bevy
    pub gltf: Handle<bevy::gltf::Gltf>,

    /// VRM 1.0 metadata
    pub meta: VrmMeta,

    /// VRM 1.0 humanoid bone mapping
    pub humanoid: Option<VrmHumanoid>,

    /// VRM 1.0 expressions (blend shapes)
    pub expressions: HashMap<String, VrmExpression>,

    /// VRM 1.0 look-at configuration
    pub look_at: Option<VrmLookAt>,

    /// First person view configuration
    pub first_person: Option<VrmFirstPerson>,
}

/// Component marking a spawned VRM entity in the scene.
#[derive(Component, Debug, Clone)]
pub struct VrmEntity {
    /// Handle to the VRM asset
    pub vrm: Handle<VrmAsset>,

    /// Name of the VRM model
    pub name: String,
}

/// Component storing morph target bindings for a specific mesh.
///
/// This component is attached to mesh entities that have morph targets,
/// mapping expression names to their morph target indices.
#[derive(Component, Debug, Clone)]
pub struct VrmMorphTargets {
    /// Map from expression name to (mesh primitives, morph target indices, weights)
    pub bindings: HashMap<String, Vec<MorphTargetBinding>>,
}

/// A single morph target binding for an expression.
#[derive(Debug, Clone)]
pub struct MorphTargetBinding {
    /// Index of the mesh primitive
    pub primitive_index: usize,

    /// Index of the morph target within that primitive
    pub morph_target_index: usize,

    /// Weight/multiplier for this morph target
    pub weight: f32,
}
