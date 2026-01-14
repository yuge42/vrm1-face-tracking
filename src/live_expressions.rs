/// Live expression weights for VRM face tracking
///
/// This module provides a system to apply live expression weights directly to
/// mesh MorphWeights, bypassing VRMA entirely.
use bevy::prelude::*;
use expression_adapter::VrmExpression as AdapterExpression;
use std::collections::HashMap;

/// Component to hold live expression weights for a VRM model
///
/// This component stores expression weights that are updated in real-time from
/// face tracking input and applied directly to MorphWeights.
#[derive(Component, Default, Debug, Clone)]
pub struct LiveExpressionWeights {
    /// Map of VRM expression names to their current weights (0.0 to 1.0)
    pub weights: HashMap<String, f32>,
}

impl LiveExpressionWeights {
    /// Create a new empty LiveExpressionWeights
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the weight for an expression
    pub fn set_weight(&mut self, expression: impl Into<String>, weight: f32) {
        self.weights
            .insert(expression.into(), weight.clamp(0.0, 1.0));
    }

    /// Get the weight for an expression (returns 0.0 if not set)
    #[allow(dead_code)]
    pub fn get_weight(&self, expression: &str) -> f32 {
        self.weights.get(expression).copied().unwrap_or(0.0)
    }

    /// Clear all weights
    pub fn clear(&mut self) {
        self.weights.clear();
    }

    /// Update weights from a list of VRM expressions
    pub fn update_from_expressions(&mut self, expressions: &[AdapterExpression]) {
        // Clear old weights
        self.clear();

        // Set new weights
        // Note: expr.preset.as_str() returns canonical VRM expression names
        // as defined in the VRM 1.0 specification, guaranteed to be valid
        for expr in expressions {
            self.set_weight(expr.preset.as_str(), expr.weight);
        }
    }
}

/// Component that caches the mapping from expression names to morph target bindings
///
/// This is built once when a VRM is loaded by discovering the expression structure
/// created by bevy_vrm1, then used for efficient per-frame weight application.
#[derive(Component, Default, Debug, Clone)]
pub struct ExpressionMorphMap {
    /// Maps expression name to list of (mesh_entity, morph_index) pairs
    pub bindings: HashMap<String, Vec<(Entity, usize)>>,
}

/// Plugin that provides the live expression weights system
pub struct LiveExpressionsPlugin;

impl Plugin for LiveExpressionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (build_expression_morph_map, apply_live_expression_weights).chain(),
        );
    }
}

/// Build the expression → morph mapping using bevy_vrm1's structure
///
/// This system runs once when a VRM is initialized. It discovers the expression → morph
/// bindings by examining the entity hierarchy that bevy_vrm1 has already set up.
/// Each expression entity created by bevy_vrm1 can be queried via reflection to find
/// which morph targets it controls.
#[allow(clippy::type_complexity)]
fn build_expression_morph_map(
    mut commands: Commands,
    vrm_query: Query<
        (Entity, &Children),
        (
            With<bevy_vrm1::prelude::Vrm>,
            With<bevy_vrm1::prelude::Initialized>,
            Without<ExpressionMorphMap>,
        ),
    >,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    // Query all entities with MorphWeights to build the mapping
    morph_entities: Query<(Entity, &Name, &MorphWeights)>,
    // Use TypeRegistry to inspect components via reflection
    type_registry: Res<AppTypeRegistry>,
) {
    for (vrm_entity, vrm_children) in vrm_query.iter() {
        println!("\n=== VRM Load: Building Expression Morph Map ===");
        println!("VRM Entity: {vrm_entity:?}");

        let mut map = ExpressionMorphMap::default();

        // Find the expressions root
        let Some(expressions_root) = find_child_with_name(
            vrm_children,
            &name_query,
            bevy_vrm1::prelude::Vrm::EXPRESSIONS_ROOT,
        ) else {
            println!("  No expressions root found!");
            continue;
        };

        println!("  Expressions root: {expressions_root:?}");

        // Get expression children
        let Ok(expr_children) = children_query.get(expressions_root) else {
            println!("  No expression children found!");
            continue;
        };

        println!("  Found {} expression entities", expr_children.len());

        // Build a list of all mesh entities with their morph counts for reference
        println!("\n  Available mesh entities with MorphWeights:");
        for (mesh_entity, mesh_name, morph_weights) in morph_entities.iter() {
            println!(
                "    - '{}' (entity: {mesh_entity:?}): {} morph targets",
                mesh_name.as_str(),
                morph_weights.weights().len()
            );
        }

        // For each expression entity, try to discover its bindings via reflection
        for expr_entity in expr_children.iter() {
            if let Ok(expr_name) = name_query.get(expr_entity) {
                let expression_name = expr_name.as_str().to_string();
                println!("\n  Expression: '{expression_name}' (entity: {expr_entity:?})");

                // Try to extract binding information via reflection
                let bindings =
                    discover_bindings_via_reflection(expr_entity, &type_registry, &name_query);

                if !bindings.is_empty() {
                    println!("    -> {} morph target bindings discovered", bindings.len());

                    // Log detailed binding information
                    for (mesh_entity, morph_index) in bindings.iter() {
                        let mesh_name = name_query
                            .get(*mesh_entity)
                            .map(|n| n.as_str())
                            .unwrap_or("<unnamed>");
                        println!(
                            "      Binding: mesh '{mesh_name}' (entity: {mesh_entity:?}), morph index: {morph_index}"
                        );
                    }

                    map.bindings.insert(expression_name, bindings);
                } else {
                    println!(
                        "    -> No bindings discovered (expression may not affect any meshes)"
                    );
                }
            }
        }

        // Insert the map for efficient runtime use
        let bindings_count = map.bindings.len();
        commands.entity(vrm_entity).insert(map);
        println!("\n=== Expression Morph Map Complete: {bindings_count} expressions mapped ===\n");
    }
}

/// Discover morph bindings for an expression entity
///
/// **Current Implementation Status:**
///
/// bevy_vrm1 stores expression → morph bindings in the `RetargetExpressionNodes` component,
/// but this component is marked `pub(crate)` and not accessible from external crates.
///
/// **Options for a complete implementation:**
///
/// 1. **Upstream fix** (cleanest): Submit a PR to bevy_vrm1 to expose binding data publicly
/// 2. **Re-parse VRM data**: Parse the VRM GLTF extensions ourselves (duplicates bevy_vrm1's work)
/// 3. **Unsafe access**: Use unsafe code to access private component data (fragile, not recommended)
///
/// For now, this returns empty bindings with a clear diagnostic message.
/// This ensures the system compiles and runs, while making the limitation explicit.
fn discover_bindings_via_reflection(
    _expr_entity: Entity,
    _type_registry: &AppTypeRegistry,
    _name_query: &Query<&Name>,
) -> Vec<(Entity, usize)> {
    println!("    ⚠️  Cannot access bevy_vrm1's internal RetargetExpressionNodes component");
    println!("    The component is marked pub(crate) and not accessible from this crate.");
    println!("    ");
    println!("    To fix this, one of the following is needed:");
    println!("    1. bevy_vrm1 should expose expression binding data via a public API");
    println!("    2. Re-parse the VRM GLTF data independently (duplicates bevy_vrm1's work)");
    println!("    3. Use reflection/unsafe to access private component data (not recommended)");

    Vec::new()
}

/// Apply live expression weights directly to MorphWeights
///
/// This system reads from LiveExpressionWeights and applies weights directly to
/// the mesh MorphWeights, bypassing any VRMA or Transform-based mechanisms.
#[allow(clippy::type_complexity)]
fn apply_live_expression_weights(
    vrm_query: Query<
        (&LiveExpressionWeights, &ExpressionMorphMap),
        (
            Changed<LiveExpressionWeights>,
            With<bevy_vrm1::prelude::Vrm>,
        ),
    >,
    mut morph_query: Query<&mut MorphWeights>,
) {
    for (weights, map) in vrm_query.iter() {
        // Apply each expression weight to its bound morphs
        for (expression_name, weight) in weights.weights.iter() {
            if let Some(bindings) = map.bindings.get(expression_name) {
                for &(mesh_entity, morph_index) in bindings.iter() {
                    if let Ok(mut morph_weights) = morph_query.get_mut(mesh_entity) {
                        let morph_weights_mut = morph_weights.weights_mut();
                        if morph_index < morph_weights_mut.len() {
                            morph_weights_mut[morph_index] = *weight;
                        }
                    }
                }
            }
        }
    }
}

/// Helper function to find a child entity by name
fn find_child_with_name(
    children: &Children,
    name_query: &Query<&Name>,
    target_name: &str,
) -> Option<Entity> {
    for child in children.iter() {
        if let Ok(name) = name_query.get(child) {
            if name.as_str() == target_name {
                return Some(child);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_expression_weights() {
        let mut weights = LiveExpressionWeights::new();

        weights.set_weight("happy", 0.5);
        assert_eq!(weights.get_weight("happy"), 0.5);

        weights.set_weight("sad", 1.0);
        assert_eq!(weights.get_weight("sad"), 1.0);

        // Test clamping
        weights.set_weight("angry", 1.5);
        assert_eq!(weights.get_weight("angry"), 1.0);

        weights.clear();
        assert_eq!(weights.get_weight("happy"), 0.0);
    }

    #[test]
    fn test_update_from_expressions() {
        use expression_adapter::VrmExpressionPreset;

        let mut weights = LiveExpressionWeights::new();
        let expressions = vec![
            AdapterExpression::new(VrmExpressionPreset::Happy, 0.8),
            AdapterExpression::new(VrmExpressionPreset::Blink, 0.3),
        ];

        weights.update_from_expressions(&expressions);

        assert_eq!(weights.get_weight("happy"), 0.8);
        assert_eq!(weights.get_weight("blink"), 0.3);
        assert_eq!(weights.get_weight("sad"), 0.0);
    }
}
