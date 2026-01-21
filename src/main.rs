use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::prelude::*;
use expression_adapter::{ArkitToVrmAdapter, BlendshapeToExpression, VrmExpression};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracker_ipc::{TrackerFrame, spawn_tracker};
use vrm_loader::{VrmAsset, VrmHandle, VrmLoaderPlugin};

mod config;
use config::AppConfig;

#[derive(Resource)]
struct TrackerReceiver {
    rx: crossbeam_channel::Receiver<TrackerFrame>,
}

#[derive(Resource)]
struct TrackerProcess {
    #[allow(dead_code)]
    child: std::process::Child,
}

#[derive(Resource, Default)]
struct VrmModelPath {
    path: Option<PathBuf>,
}

#[derive(Resource)]
struct FileDialogChannel {
    tx: Arc<Mutex<crossbeam_channel::Sender<Option<PathBuf>>>>,
    rx: crossbeam_channel::Receiver<Option<PathBuf>>,
}

#[derive(Resource)]
struct Config {
    inner: AppConfig,
}

#[derive(Component)]
struct CurrentVrmEntity;

/// Component that stores the VRM expression to morph target mapping for a mesh entity.
/// This is attached to mesh entities after a VRM is loaded to enable applying expressions.
#[derive(Component, Clone)]
struct VrmExpressionMap {
    /// Map from VRM expression name (e.g., "happy", "blink") to morph target indices and weights
    /// The Vec contains tuples of (morph_target_index, base_weight)
    expression_to_morphs: HashMap<String, Vec<(usize, f32)>>,
}

/// Resource that stores the current VRM expression weights from face tracking.
#[derive(Resource, Default)]
struct CurrentExpressions {
    expressions: Vec<VrmExpression>,
}

fn main() {
    // Load or create configuration
    let config = AppConfig::load_or_create().expect("Failed to load configuration");

    // Ensure user VRM directory exists
    if let Err(e) = config.ensure_user_vrm_dir() {
        eprintln!("Warning: Failed to create user VRM directory: {e}");
    }

    println!(
        "User VRM models directory: {}",
        config.user_vrm_dir.display()
    );
    println!("Configuration loaded successfully");

    let user_vrm_dir = config.user_vrm_dir.clone();

    App::new()
        // Register custom asset source BEFORE adding plugins
        .register_asset_source(
            AssetSourceId::Name("userdata".into()),
            AssetSource::build().with_reader(move || {
                Box::new(bevy::asset::io::file::FileAssetReader::new(
                    user_vrm_dir.clone(),
                ))
            }),
        )
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "assets".to_string(),
            ..default()
        }))
        .add_plugins(VrmLoaderPlugin)
        .insert_resource(Config { inner: config })
        .init_resource::<VrmModelPath>()
        .init_resource::<CurrentExpressions>()
        .add_systems(Startup, (setup_tracker, setup_scene, setup_file_dialog))
        .add_systems(
            Update,
            (
                dump_tracker_frames,
                check_vrm_load_status,
                handle_file_dialog_input,
                receive_file_dialog_result,
                load_vrm_from_path,
                build_expression_maps,
                apply_expressions,
            ),
        )
        .run();
}

fn setup_tracker(mut commands: Commands) {
    // Use PYTHON_BIN environment variable if set, otherwise default to "python3"
    let python_bin = std::env::var("PYTHON_BIN").unwrap_or_else(|_| "python3".to_string());

    let (child, rx) = spawn_tracker(
        &python_bin,
        "tools/mediapipe_tracker.py", // Relative Path
    );

    commands.insert_resource(TrackerReceiver { rx });
    commands.insert_resource(TrackerProcess { child });

    println!("Tracker process started with Python: {python_bin}");
}

fn dump_tracker_frames(
    rx: Res<TrackerReceiver>,
    mut current_expressions: ResMut<CurrentExpressions>,
) {
    let adapter = ArkitToVrmAdapter;

    while let Ok(frame) = rx.rx.try_recv() {
        // Use the expression adapter to convert ARKit blendshapes to VRM expressions
        let vrm_expressions = adapter.to_vrm_expressions(&frame.blendshapes);

        // Store expressions for the apply_expressions system to use
        current_expressions.expressions = vrm_expressions.clone();

        // Print the converted expressions
        if !vrm_expressions.is_empty() {
            let expr_summary: Vec<String> = vrm_expressions
                .iter()
                .map(|e| format!("{}={:.2}", e.preset.as_str(), e.weight))
                .collect();

            println!(
                "ts={:.3} expressions=[{}]",
                frame.ts,
                expr_summary.join(", ")
            );
        }
    }
}

fn setup_scene(mut commands: Commands, config: Res<Config>) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.3, 1.5).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // Spawn directional light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, 3.0, 0.3).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("Scene setup complete.");
    println!(
        "User VRM directory: {}",
        config.inner.user_vrm_dir.display()
    );
    println!("Press 'O' to open a file dialog and select a VRM model to load.");
}

fn check_vrm_load_status(
    mut events: MessageReader<AssetEvent<VrmAsset>>,
    mut reported: Local<bool>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                if !*reported {
                    println!("✓ VRM model loaded successfully");
                    *reported = true;
                }
            }
            AssetEvent::LoadedWithDependencies { .. } => {
                if !*reported {
                    println!("✓ VRM model and dependencies loaded successfully");
                    *reported = true;
                }
            }
            _ => {}
        }
    }
}

fn setup_file_dialog(mut commands: Commands) {
    let (tx, rx) = crossbeam_channel::unbounded();
    commands.insert_resource(FileDialogChannel {
        tx: Arc::new(Mutex::new(tx)),
        rx,
    });
}

fn handle_file_dialog_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    file_dialog_channel: Res<FileDialogChannel>,
    config: Res<Config>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyO) {
        println!("Opening file dialog...");

        let tx = file_dialog_channel.tx.clone();
        let user_vrm_dir = config.inner.user_vrm_dir.clone();

        // Spawn a thread to open the file dialog without blocking the main thread
        std::thread::spawn(move || {
            let file = rfd::FileDialog::new()
                .add_filter("VRM Model", &["vrm"])
                .set_title("Select VRM Model")
                .set_directory(&user_vrm_dir)
                .pick_file();

            if let Some(path) = &file {
                println!("Selected file: {}", path.display());
            } else {
                println!("File selection cancelled");
            }

            // Send the result through the channel
            if let Ok(sender) = tx.lock() {
                let _ = sender.send(file);
            }
        });
    }
}

fn receive_file_dialog_result(
    file_dialog_channel: Res<FileDialogChannel>,
    mut vrm_path: ResMut<VrmModelPath>,
) {
    while let Ok(result) = file_dialog_channel.rx.try_recv() {
        if let Some(path) = result {
            println!("Received selected file: {}", path.display());
            vrm_path.path = Some(path);
        }
    }
}

fn load_vrm_from_path(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vrm_path: ResMut<VrmModelPath>,
    current_vrm_query: Query<Entity, With<CurrentVrmEntity>>,
    config: Res<Config>,
) {
    if let Some(path) = vrm_path.path.take() {
        // Remove the current VRM entity if it exists
        for entity in current_vrm_query.iter() {
            commands.entity(entity).despawn();
        }

        // Copy the file to the user VRM directory so Bevy can load it
        let user_vrm_dir = &config.inner.user_vrm_dir;
        if let Err(e) = std::fs::create_dir_all(user_vrm_dir) {
            eprintln!("Failed to create user VRM directory: {e}");
            return;
        }

        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("model.vrm"));
        let dest_path = user_vrm_dir.join(file_name);

        // Only copy if source and destination are different
        if path != dest_path {
            if let Err(e) = std::fs::copy(&path, &dest_path) {
                eprintln!("Failed to copy VRM file to user directory: {e}");
                return;
            }
            println!("Copied VRM file to: {}", dest_path.display());
        }

        // Load the VRM model via the userdata asset source
        let asset_path = format!("userdata://{}", file_name.to_string_lossy());
        println!("Loading VRM model from user data: {asset_path}");
        let vrm_handle: Handle<VrmAsset> = asset_server.load(&asset_path);
        commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity));
    }
}

/// System that builds VRM expression maps for mesh entities.
///
/// This system runs after a VRM scene is spawned and builds the mapping from
/// expression names to morph target indices for each mesh entity.
///
/// Note: This is a simplified implementation that applies expression mappings to all
/// mesh entities. A more accurate implementation would map glTF node indices to
/// specific entities, but this works for most VRM models where expressions
/// are defined on facial meshes.
#[allow(clippy::type_complexity)]
fn build_expression_maps(
    mut commands: Commands,
    vrm_assets: Res<Assets<VrmAsset>>,
    gltf_assets: Res<Assets<bevy::gltf::Gltf>>,
    vrm_entities: Query<
        (Entity, &VrmHandle, &Children),
        (With<CurrentVrmEntity>, Without<VrmExpressionMap>),
    >,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Mesh3d>>,
) {
    for (vrm_entity, vrm_handle, children) in vrm_entities.iter() {
        let Some(vrm_asset) = vrm_assets.get(&vrm_handle.0) else {
            continue;
        };

        let Some(_gltf) = gltf_assets.get(&vrm_asset.gltf) else {
            continue;
        };

        // Collect all mesh entities in the scene
        let mut mesh_entities = Vec::new();
        collect_all_meshes(children, &children_query, &mesh_query, &mut mesh_entities);

        // Build expression maps
        // Since we don't have a reliable way to map glTF node indices to entities,
        // we'll create a combined expression map with all morph target bindings
        // and apply it to all mesh entities
        let mut combined_expr_map = VrmExpressionMap {
            expression_to_morphs: HashMap::new(),
        };

        for (expression_name, expression_data) in vrm_asset.expressions.iter() {
            for morph_bind in expression_data.morph_target_binds.iter() {
                combined_expr_map
                    .expression_to_morphs
                    .entry(expression_name.clone())
                    .or_default()
                    .push((morph_bind.index, morph_bind.weight));
            }
        }

        // Apply the expression map to all mesh entities
        for &mesh_entity in &mesh_entities {
            commands
                .entity(mesh_entity)
                .insert(combined_expr_map.clone());
        }

        // Mark the VRM entity as processed
        commands.entity(vrm_entity).insert(VrmExpressionMap {
            expression_to_morphs: HashMap::new(),
        });

        info!(
            "Built expression maps for VRM: {} ({} mesh entities)",
            vrm_asset.meta.name,
            mesh_entities.len()
        );
    }
}

/// Helper function to collect all mesh entities from the scene hierarchy
fn collect_all_meshes(
    children: &Children,
    children_query: &Query<&Children>,
    mesh_query: &Query<Entity, With<Mesh3d>>,
    mesh_entities: &mut Vec<Entity>,
) {
    for child in children.iter() {
        // Check if this child is a mesh entity
        if mesh_query.get(child).is_ok() {
            mesh_entities.push(child);
        }

        // Recursively check children
        if let Ok(grandchildren) = children_query.get(child) {
            collect_all_meshes(grandchildren, children_query, mesh_query, mesh_entities);
        }
    }
}

/// System that applies VRM expressions to mesh morph weights.
///
/// This system takes the current VRM expressions from face tracking and applies
/// them to the mesh entities' MorphWeights components.
fn apply_expressions(
    current_expressions: Res<CurrentExpressions>,
    mut mesh_query: Query<(&VrmExpressionMap, &mut MorphWeights)>,
) {
    if current_expressions.expressions.is_empty() {
        return;
    }

    for (expr_map, mut morph_weights) in mesh_query.iter_mut() {
        // Build a map from expression name to weight
        let mut expression_weights: HashMap<String, f32> = HashMap::new();
        for expr in current_expressions.expressions.iter() {
            expression_weights.insert(expr.preset.as_str().to_string(), expr.weight);
        }

        // Calculate the new morph weights
        // We need to know the total number of morph targets for this mesh
        let num_morph_targets = morph_weights.weights().len();
        let mut new_weights = vec![0.0; num_morph_targets];

        // Apply each expression
        for (expr_name, expr_weight) in expression_weights.iter() {
            if let Some(morph_bindings) = expr_map.expression_to_morphs.get(expr_name) {
                for &(morph_idx, base_weight) in morph_bindings {
                    if morph_idx < num_morph_targets {
                        new_weights[morph_idx] += expr_weight * base_weight;
                    }
                }
            }
        }

        // Clamp weights to [0, 1]
        for weight in new_weights.iter_mut() {
            *weight = weight.clamp(0.0, 1.0);
        }

        // Update the morph weights
        morph_weights.weights_mut().copy_from_slice(&new_weights);
    }
}
