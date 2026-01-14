# Expression Adapter

A Rust crate providing a flexible trait-based system for mapping raw face tracker blendshape data to VRM 1.0 expression presets.

## Overview

The `expression_adapter` crate defines the `BlendshapeToExpression` trait, which provides a standard interface for converting tracker-specific blendshape data (such as ARKit-style blendshapes from MediaPipe) into VRM 1.0 expressions.

## Features

- **VRM Expression Presets**: Complete enum of VRM 1.0 expression presets including emotions, lip sync, blink, and gaze
- **Trait-based Design**: Flexible adapter pattern allowing custom implementations
- **Default ARKit Adapter**: Ready-to-use adapter for MediaPipe Face Landmarker's 52 ARKit blendshapes
- **Type Safety**: Strongly-typed expression weights with automatic clamping (0.0-1.0)
- **Well-tested**: Comprehensive test suite ensuring correctness

## VRM Expression Presets

According to the [VRM 1.0 specification](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/expressions.md), expressions are categorized as:

### Emotions
- `happy` - joy/happiness (changed from VRM 0.x "joy")
- `angry` - anger
- `sad` - sadness (changed from VRM 0.x "sorrow")
- `relaxed` - comfortable/relaxed (changed from VRM 0.x "fun")
- `surprised` - surprise (new in VRM 1.0)

### Lip Sync (Procedural)
- `aa` - open mouth as in "father"
- `ih` - narrow mouth as in "eat"
- `ou` - rounded lips as in "boot"
- `ee` - wide mouth as in "see"
- `oh` - open rounded mouth as in "boat"

### Blink (Procedural)
- `blink` - close both eyelids
- `blinkLeft` - close left eyelid
- `blinkRight` - close right eyelid

### Gaze (Procedural)
- `lookUp` - look upward
- `lookDown` - look downward
- `lookLeft` - look left
- `lookRight` - look right

### Other
- `neutral` - neutral expression (for backward compatibility)

## Usage

### Using the Default ARKit Adapter

```rust
use expression_adapter::{ArkitToVrmAdapter, BlendshapeToExpression};
use std::collections::HashMap;

// Create the adapter
let adapter = ArkitToVrmAdapter;

// Prepare blendshape data (e.g., from MediaPipe Face Landmarker)
let mut blendshapes = HashMap::new();
blendshapes.insert("eyeBlinkLeft".to_string(), 0.8);
blendshapes.insert("eyeBlinkRight".to_string(), 0.9);
blendshapes.insert("mouthSmileLeft".to_string(), 0.7);
blendshapes.insert("mouthSmileRight".to_string(), 0.7);

// Convert to VRM expressions
let vrm_expressions = adapter.to_vrm_expressions(&blendshapes);

// Use the expressions
for expr in vrm_expressions {
    println!("{}: {:.2}", expr.preset.as_str(), expr.weight);
}
```

### Creating a Custom Adapter

```rust
use expression_adapter::{BlendshapeToExpression, VrmExpression, VrmExpressionPreset};
use std::collections::HashMap;

struct MyCustomAdapter;

impl BlendshapeToExpression for MyCustomAdapter {
    fn to_vrm_expressions(&self, raw_blendshapes: &HashMap<String, f32>) -> Vec<VrmExpression> {
        let mut expressions = Vec::new();
        
        // Custom mapping logic
        if let Some(&value) = raw_blendshapes.get("my_custom_blink") {
            expressions.push(VrmExpression::new(VrmExpressionPreset::Blink, value));
        }
        
        expressions
    }
}
```

## ARKit Blendshape Mapping

The `ArkitToVrmAdapter` implements sensible default mappings:

### Direct Mappings
- **Blink**: `eyeBlinkLeft` → `blinkLeft`, `eyeBlinkRight` → `blinkRight`, average → `blink`
- **Eye Gaze**: Eye look directions are averaged between left and right eyes

### Weighted Combinations
- **Happy**: Average of `mouthSmileLeft` and `mouthSmileRight` (threshold: 0.3)
- **Sad**: Average of `mouthFrownLeft` and `mouthFrownRight` (threshold: 0.3)

### Lip Sync Heuristics
- **Aa**: `jawOpen` (threshold: 0.5)
- **Ou**: `mouthPucker` (threshold: 0.5)
- **Oh**: `mouthFunnel` (threshold: 0.5)

Note: The default adapter uses simple heuristics for lip sync. For production use with actual speech, consider integrating with audio analysis or speech recognition.

## Testing

Run the test suite:

```bash
cargo test -p expression_adapter
```

The test suite includes:
- Enum string conversion tests
- Weight clamping validation
- ARKit adapter functionality tests
- Edge case handling

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
