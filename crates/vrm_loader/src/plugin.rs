//! Bevy plugin for VRM 1.0 loading.

use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetApp, AssetEvent, Assets, Handle};
use bevy::ecs::system::{Query, Res};
use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::{VrmAsset, VrmEntity, VrmLoader, print_vrm_expressions, print_vrm_metadata};

/// Plugin that adds VRM 1.0 loading support to a Bevy app.
///
/// This plugin:
/// - Registers the VRM asset loader
/// - Adds systems to process loaded VRM assets
/// - Prints VRM metadata to console when models are loaded
pub struct VrmLoaderPlugin;

impl Plugin for VrmLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VrmAsset>()
            .init_asset_loader::<VrmLoader>()
            .add_systems(Update, (process_loaded_vrm_assets, spawn_vrm_entities));
    }
}

/// System that processes newly loaded VRM assets.
///
/// This system:
/// - Detects when VRM assets finish loading
/// - Prints metadata to console
/// - Prints expression information
fn process_loaded_vrm_assets(
    mut events: MessageReader<AssetEvent<VrmAsset>>,
    vrm_assets: Res<Assets<VrmAsset>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            if let Some(vrm) = vrm_assets.get(*id) {
                // Print metadata to console
                print_vrm_metadata(&vrm.meta);
                print_vrm_expressions(&vrm.expressions);
            }
        }
    }
}

/// Component wrapper for VrmAsset handle to make it queryable
#[derive(Component)]
pub struct VrmHandle(pub Handle<VrmAsset>);

/// System that spawns VRM entities in the scene.
///
/// This system looks for unspawned VRM assets and creates entities for them.
/// It also ensures the underlying glTF scene is spawned.
fn spawn_vrm_entities(
    mut commands: Commands,
    vrm_assets: Res<Assets<VrmAsset>>,
    gltf_assets: Res<Assets<Gltf>>,
    query: Query<(Entity, &VrmHandle), Without<VrmEntity>>,
) {
    for (entity, vrm_handle) in query.iter() {
        if let Some(vrm) = vrm_assets.get(&vrm_handle.0) {
            // Check if the glTF is loaded
            if let Some(gltf) = gltf_assets.get(&vrm.gltf) {
                // Add VrmEntity component
                commands.entity(entity).insert(VrmEntity {
                    vrm: vrm_handle.0.clone(),
                    name: vrm.meta.name.clone(),
                });

                // Spawn the default glTF scene
                if let Some(scene) = gltf.scenes.first() {
                    commands.entity(entity).insert(SceneRoot(scene.clone()));
                }

                info!("Spawned VRM entity: {}", vrm.meta.name);
            }
        }
    }
}
