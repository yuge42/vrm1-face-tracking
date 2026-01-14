/// Live expression weights for VRM face tracking
///
/// This module provides a system to apply live expression weights directly to
/// mesh MorphWeights by using the expression entities created by bevy_vrm1.

use bevy::prelude::*;
use std::collections::HashMap;
use expression_adapter::VrmExpression as AdapterExpression;

/// Component to hold live expression weights for a VRM model
///
/// This component stores expression weights that are updated in real-time from
/// face tracking input and applied to the VRM via expression Transform entities.
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
        self.weights.insert(expression.into(), weight.clamp(0.0, 1.0));
    }

    /// Get the weight for an expression (returns 0.0 if not set)
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
        for expr in expressions {
            self.set_weight(expr.preset.as_str(), expr.weight);
        }
    }
}

/// Plugin that provides the live expression weights system
pub struct LiveExpressionsPlugin;

impl Plugin for LiveExpressionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                PostUpdate,
                apply_live_expression_weights,
            );
    }
}

/// Apply live expression weights to VRM expression transforms
///
/// This system works by setting the Transform.translation.x value on expression
/// entities, which bevy_vrm1's VRMA system will then use to update MorphWeights.
/// This approach reuses the existing bevy_vrm1 infrastructure without
/// duplicating the expression->morph mapping logic.
fn apply_live_expression_weights(
    vrm_query: Query<(&LiveExpressionWeights, &Children), (Changed<LiveExpressionWeights>, With<bevy_vrm1::prelude::Vrm>)>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mut transform_query: Query<&mut Transform>,
) {
    for (weights, vrm_children) in vrm_query.iter() {
        // Find the expressions root
        let Some(expressions_root) = find_child_with_name(vrm_children, &name_query, bevy_vrm1::prelude::Vrm::EXPRESSIONS_ROOT) else {
            continue;
        };

        // Get expression children
        let Ok(expr_children) = children_query.get(expressions_root) else {
            continue;
        };

        // Apply weights to expression entities
        for expr_entity in expr_children.iter() {
            if let Ok(expr_name) = name_query.get(expr_entity) {
                let weight = weights.get_weight(expr_name.as_str());
                
                if let Ok(mut transform) = transform_query.get_mut(expr_entity) {
                    // bevy_vrm1 VRMA uses Transform.translation.x as the expression weight
                    transform.translation.x = weight;
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
        use expression_adapter::{VrmExpression as AdapterExpression, VrmExpressionPreset};
        
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
