//! VRM asset loader implementation.
//!
//! This module provides the AssetLoader implementation that parses VRM 1.0
//! extensions from glTF files using Bevy's glTF loader.

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::{VrmAsset, VrmExpression, VrmMeta, VrmcVrmExtension};

/// Asset loader for VRM 1.0 files.
///
/// This loader:
/// 1. Uses Bevy's built-in glTF loader to load the base glTF data
/// 2. Parses the VRMC_vrm extension from the glTF JSON
/// 3. Creates a VrmAsset with the parsed data
#[derive(Default)]
pub struct VrmLoader;

impl AssetLoader for VrmLoader {
    type Asset = VrmAsset;
    type Settings = ();
    type Error = VrmLoadError;

    async fn load(
        &self,
        reader: &mut (dyn Reader + '_),
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // Read the entire VRM file into memory
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        // Parse as glTF/GLB
        let vrm_asset = parse_vrm_from_bytes(&bytes, load_context)?;

        Ok(vrm_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["vrm"]
    }
}

/// Error type for VRM loading.
#[derive(Debug, thiserror::Error)]
pub enum VrmLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("glTF parse error: {0}")]
    Gltf(String),

    #[error("Missing VRM extension")]
    MissingVrmExtension,

    #[error("Invalid VRM extension: {0}")]
    InvalidVrmExtension(String),
}

/// Parse VRM data from GLB or glTF bytes.
fn parse_vrm_from_bytes(
    bytes: &[u8],
    load_context: &mut LoadContext,
) -> Result<VrmAsset, VrmLoadError> {
    // Try to parse as GLB first (most VRM files are GLB format)
    let (json_data, _buffer_data) = if bytes.starts_with(b"glTF") {
        parse_glb(bytes)?
    } else {
        // If not GLB, treat as regular JSON glTF
        (bytes.to_vec(), Vec::new())
    };

    // Parse the JSON
    let json: Value = serde_json::from_slice(&json_data)?;

    // Extract the VRMC_vrm extension
    let vrm_extension = extract_vrm_extension(&json)?;

    // Load the glTF asset using Bevy's loader
    // Use the full asset path (including source) to preserve userdata:// scheme
    let asset_path = load_context.path().to_owned();
    let gltf_handle: Handle<Gltf> = load_context.load(asset_path);

    // Combine preset and custom expressions
    let mut all_expressions = HashMap::new();
    all_expressions.extend(vrm_extension.expressions.preset.clone());
    all_expressions.extend(vrm_extension.expressions.custom.clone());

    Ok(VrmAsset {
        gltf: gltf_handle,
        meta: vrm_extension.meta,
        humanoid: vrm_extension.humanoid,
        expressions: all_expressions,
        look_at: vrm_extension.look_at,
        first_person: vrm_extension.first_person,
    })
}

/// Parse GLB binary format.
///
/// GLB structure:
/// - 12-byte header (magic, version, length)
/// - JSON chunk (type 0x4E4F534A)
/// - Binary chunk (type 0x004E4942)
fn parse_glb(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), VrmLoadError> {
    if bytes.len() < 12 {
        return Err(VrmLoadError::Gltf("File too small to be GLB".to_string()));
    }

    // Check magic number "glTF"
    if &bytes[0..4] != b"glTF" {
        return Err(VrmLoadError::Gltf("Invalid GLB magic number".to_string()));
    }

    // Read version (should be 2)
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != 2 {
        return Err(VrmLoadError::Gltf(format!(
            "Unsupported GLB version: {version}"
        )));
    }

    // Read total length
    let _total_length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    let mut offset = 12;
    let mut json_data = Vec::new();
    let mut bin_data = Vec::new();

    // Read chunks
    while offset < bytes.len() {
        if offset + 8 > bytes.len() {
            break;
        }

        let chunk_length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        let chunk_type = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);

        offset += 8;

        if offset + chunk_length > bytes.len() {
            break;
        }

        match chunk_type {
            0x4E4F534A => {
                // JSON chunk
                json_data = bytes[offset..offset + chunk_length].to_vec();
            }
            0x004E4942 => {
                // BIN chunk
                bin_data = bytes[offset..offset + chunk_length].to_vec();
            }
            _ => {
                // Unknown chunk type, skip
            }
        }

        offset += chunk_length;
    }

    Ok((json_data, bin_data))
}

/// Extract VRMC_vrm extension from glTF JSON.
fn extract_vrm_extension(json: &Value) -> Result<VrmcVrmExtension, VrmLoadError> {
    // Navigate to extensions.VRMC_vrm
    let extensions = json
        .get("extensions")
        .ok_or(VrmLoadError::MissingVrmExtension)?;

    let vrmc_vrm = extensions
        .get("VRMC_vrm")
        .ok_or(VrmLoadError::MissingVrmExtension)?;

    // Deserialize the VRM extension
    let vrm_extension: VrmcVrmExtension = serde_json::from_value(vrmc_vrm.clone())
        .map_err(|e| VrmLoadError::InvalidVrmExtension(e.to_string()))?;

    Ok(vrm_extension)
}

/// Print VRM metadata to console.
pub fn print_vrm_metadata(meta: &VrmMeta) {
    println!("\n=== VRM Model Metadata ===");
    println!("Name: {}", meta.name);

    if !meta.version.is_empty() {
        println!("Version: {}", meta.version);
    }

    if !meta.authors.is_empty() {
        println!("Authors: {}", meta.authors.join(", "));
    }

    if !meta.copyright_information.is_empty() {
        println!("Copyright: {}", meta.copyright_information);
    }

    if !meta.license_url.is_empty() {
        println!("License: {}", meta.license_url);
    }

    if !meta.avatar_permission.is_empty() {
        println!("Avatar Permission: {}", meta.avatar_permission);
    }

    if !meta.commercial_usage.is_empty() {
        println!("Commercial Usage: {}", meta.commercial_usage);
    }

    println!("==========================\n");
}

/// Print VRM expressions to console.
pub fn print_vrm_expressions(expressions: &HashMap<String, VrmExpression>) {
    if expressions.is_empty() {
        return;
    }

    println!("\n=== VRM Expressions ===");
    println!("Total expressions: {}", expressions.len());

    for (name, expression) in expressions.iter() {
        println!(
            "  - {}: {} morph targets",
            name,
            expression.morph_target_binds.len()
        );
    }

    println!("=======================\n");
}
