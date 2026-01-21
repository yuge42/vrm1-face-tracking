# VRM Loader

Minimal VRM 1.0 glTF parser for Bevy applications.

## Overview

This crate provides a lightweight VRM 1.0 loader designed specifically for real-time face tracking use cases. It parses VRM 1.0 extensions from glTF files and exposes them as plain Rust data structures, without any animation or VRMA assumptions.

## Architecture

```
glTF (.vrm file)
   ↓
Bevy's built-in glTF loader (meshes, materials, scenes)
   ↓
VRM extension parser (this crate)
   ↓
Plain Rust structs (VRM metadata, expressions, morph targets)
   ↓
Application systems
   ↓
MorphWeights manipulation
```

## Features

- **VRM 1.0 Extension Parsing**: Parses the `VRMC_vrm` extension from glTF files
- **Metadata Extraction**: Extracts VRM model metadata (name, authors, license, etc.)
- **Expression Mapping**: Parses VRM expressions (preset and custom) with morph target bindings
- **Humanoid Bone Mapping**: Reads humanoid bone structure
- **Look-at Configuration**: Parses look-at settings
- **First Person Settings**: Extracts first-person view configuration
- **Console Logging**: Automatically prints VRM metadata when models are loaded

## Usage

### Adding to Your Bevy App

```rust
use bevy::prelude::*;
use vrm_loader::{VrmLoaderPlugin, VrmAsset, VrmHandle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VrmLoaderPlugin)
        .run();
}
```

### Loading a VRM Model

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load a VRM file
    let vrm_handle: Handle<VrmAsset> = asset_server.load("models/my_model.vrm");
    
    // Spawn an entity with the VRM handle
    commands.spawn(VrmHandle(vrm_handle));
}
```

The `VrmLoaderPlugin` will automatically:
1. Load the VRM file using Bevy's glTF loader
2. Parse the VRM 1.0 extensions
3. Print metadata to the console
4. Spawn the glTF scene

### Accessing VRM Data

```rust
fn use_vrm_data(
    vrm_assets: Res<Assets<VrmAsset>>,
    query: Query<&VrmHandle>,
) {
    for vrm_handle in query.iter() {
        if let Some(vrm) = vrm_assets.get(&vrm_handle.0) {
            println!("Model name: {}", vrm.meta.name);
            println!("Expressions: {:?}", vrm.expressions.keys());
            
            // Access expression data
            if let Some(happy_expr) = vrm.expressions.get("happy") {
                println!("Happy expression has {} morph targets", 
                    happy_expr.morph_target_binds.len());
            }
        }
    }
}
```

## VRM 1.0 Specification

This crate implements parsing for the following VRM 1.0 extensions:

- **VRMC_vrm**: Core VRM extension
  - Meta: Model metadata and licensing
  - Humanoid: Bone mapping
  - Expressions: Preset and custom expressions with morph target bindings
  - LookAt: Eye gaze configuration
  - FirstPerson: First-person view settings

For more information on the VRM 1.0 specification, see:
- [VRM 1.0 Specification](https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm-1.0)
- [VRM Meta](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/meta.md)
- [VRM Expressions](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md)
- [VRM Humanoid](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/humanoid.md)

## File Format Support

- **GLB Format**: Binary glTF files (`.vrm` files are typically GLB format)
- **glTF Format**: JSON glTF files (with separate `.bin` files)

## Design Philosophy

This crate is designed with the following principles:

1. **Minimal and Focused**: Only implements VRM 1.0 parsing, no animation system
2. **Plain Data Structures**: VRM data exposed as simple Rust structs
3. **Face Tracking Ready**: Designed for live face-tracking applications
4. **No VRMA Assumptions**: Does not assume animation file (.vrma) usage
5. **Bevy Integration**: Works seamlessly with Bevy's asset system and glTF loader

## Example Output

When a VRM model is loaded, the plugin automatically prints metadata to the console:

```
=== VRM Model Metadata ===
Name: My VRM Avatar
Version: 1.0
Authors: John Doe
License: CC0
Avatar Permission: onlyAuthor
Commercial Usage: personalNonProfit
==========================

=== VRM Expressions ===
Total expressions: 15
  - happy: 2 morph targets
  - angry: 3 morph targets
  - sad: 2 morph targets
  - blink: 2 morph targets
  - blinkLeft: 1 morph targets
  - blinkRight: 1 morph targets
  ...
=======================
```

## License

Dual-licensed under MIT OR Apache-2.0, at your option.
