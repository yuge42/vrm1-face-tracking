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

/// Resource that stores the body position derived from shoulder world landmarks.
///
/// The midpoint of the two shoulder world landmarks is used to translate the
/// VRM root entity so that the model tracks the subject's real-world torso
/// movement.
#[derive(Resource, Default)]
struct CurrentShoulderPosition {
    /// World-space midpoint of the two shoulders in MediaPipe coordinates.
    /// Origin is at the hip centre; Y is up; X is to the person's right;
    /// Z is toward the camera.  Units are meters.
    midpoint: Option<Vec3>,
}

// Key upper body landmark indices and names for logging
const KEY_POSE_LANDMARKS: [usize; 7] = [0, 11, 12, 13, 14, 15, 16];
const KEY_POSE_LANDMARK_NAMES: [&str; 7] = [
    "nose",
    "left_shoulder",
    "right_shoulder",
    "left_elbow",
    "right_elbow",
    "left_wrist",
    "right_wrist",
];

/// Index of the left shoulder in MediaPipe's 33-landmark pose array
const LEFT_SHOULDER_IDX: usize = 11;
/// Index of the right shoulder in MediaPipe's 33-landmark pose array
const RIGHT_SHOULDER_IDX: usize = 12;
/// Minimum visibility score for a shoulder landmark to be considered reliable
const SHOULDER_VISIBILITY_THRESHOLD: f32 = 0.1;
/// Vertical offset (in meters) added to the shoulder world-space Y to obtain the
/// VRM root (feet) Y in Bevy world space.
///
/// MediaPipe world landmarks have their origin at the hip centre, so shoulders
/// sit at approximately +0.4 m.  A typical VRM model has shoulders at roughly
/// 1.4 m above its root (feet).  The hip centre is approximately 1.0 m above
/// the floor, so:
///
///   vrm_root_y = hip_floor_offset + shoulder_world_y - shoulder_height_from_feet
///             ≈ 1.0 + shoulder_world_y - 1.4
///             = shoulder_world_y - 0.4
const SHOULDER_Y_OFFSET: f32 = -0.4;

// ---------------------------------------------------------------------------
// Body movement direction / scale controls
//
// Each axis can be independently:
//   • flipped  – change 1.0 → -1.0 to invert that direction
//   • scaled   – multiply by a value other than 1.0 to amplify / dampen movement
//
// Defaults reproduce a 1-to-1 mapping from MediaPipe world coordinates to
// Bevy world coordinates with no inversion.
// ---------------------------------------------------------------------------

/// Sign multiplier for left-right (X) body translation.
/// 1.0  = natural (MediaPipe X is already "person's right", same as Bevy X)
/// -1.0 = mirror / flip left↔right
const BODY_X_SIGN: f32 = -1.0;

/// Scale factor for left-right (X) body translation.
/// Increase above 1.0 to amplify horizontal movement; decrease toward 0.0 to dampen it.
const BODY_X_SCALE: f32 = 1.0;

/// Sign multiplier for up-down (Y) body translation.
/// 1.0  = natural (up in MediaPipe = up in Bevy)
/// -1.0 = flip up↔down
const BODY_Y_SIGN: f32 = 1.0;

/// Scale factor for up-down (Y) body translation.
/// Increase above 1.0 to amplify vertical movement; decrease toward 0.0 to dampen it.
const BODY_Y_SCALE: f32 = 1.0;

/// Sign multiplier for forward-backward (Z) body translation.
/// 1.0  = natural (toward-camera in MediaPipe = toward-viewer in Bevy)
/// -1.0 = flip forward↔backward
const BODY_Z_SIGN: f32 = -1.0;

/// Scale factor for forward-backward (Z) body translation.
/// Increase above 1.0 to amplify depth movement; decrease toward 0.0 to dampen it.
const BODY_Z_SCALE: f32 = 1.0;

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
        .init_resource::<CurrentShoulderPosition>()
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
                apply_body_position,
            ),
        )
        .run();
}

fn setup_tracker(mut commands: Commands, config: Res<Config>) {
    // Use PYTHON_BIN environment variable if set, otherwise default to "python3"
    let python_bin = std::env::var("PYTHON_BIN").unwrap_or_else(|_| "python3".to_string());

    let camera_device_id = config.inner.camera_device_id.to_string();
    let (child, rx) = spawn_tracker(
        &python_bin,
        "tools/mediapipe_tracker.py", // Relative Path
        &["--camera", &camera_device_id],
    );

    commands.insert_resource(TrackerReceiver { rx });
    commands.insert_resource(TrackerProcess { child });

    println!("Tracker process started with Python: {python_bin}");
    println!("Using camera device ID: {}", config.inner.camera_device_id);
}

fn dump_tracker_frames(
    rx: Res<TrackerReceiver>,
    mut current_expressions: ResMut<CurrentExpressions>,
    mut shoulder_pos: ResMut<CurrentShoulderPosition>,
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

        // Print pose landmark information if available
        if !frame.pose_landmarks.is_empty() {
            let pose_summary: Vec<String> = KEY_POSE_LANDMARKS
                .iter()
                .zip(KEY_POSE_LANDMARK_NAMES.iter())
                .filter_map(|(&idx, &name)| {
                    frame.pose_landmarks.get(idx).map(|lm| {
                        format!(
                            "{}=({:.2},{:.2},{:.2},v={:.2})",
                            name, lm.x, lm.y, lm.z, lm.visibility
                        )
                    })
                })
                .collect();

            if !pose_summary.is_empty() {
                println!("ts={:.3} pose=[{}]", frame.ts, pose_summary.join(", "));
            }
        }

        // Print pose world landmarks if available
        if !frame.pose_world_landmarks.is_empty() {
            let world_summary: Vec<String> = KEY_POSE_LANDMARKS
                .iter()
                .zip(KEY_POSE_LANDMARK_NAMES.iter())
                .filter_map(|(&idx, &name)| {
                    frame.pose_world_landmarks.get(idx).map(|lm| {
                        format!(
                            "{}=({:.3},{:.3},{:.3}m,v={:.2})",
                            name, lm.x, lm.y, lm.z, lm.visibility
                        )
                    })
                })
                .collect();

            if !world_summary.is_empty() {
                println!(
                    "ts={:.3} pose_world=[{}]",
                    frame.ts,
                    world_summary.join(", ")
                );
            }
        }

        // Update body position from shoulder world landmarks.
        // Shoulder indices: 11 = left shoulder, 12 = right shoulder.
        if frame.pose_world_landmarks.len() > RIGHT_SHOULDER_IDX {
            let left = &frame.pose_world_landmarks[LEFT_SHOULDER_IDX];
            let right = &frame.pose_world_landmarks[RIGHT_SHOULDER_IDX];

            if left.visibility >= SHOULDER_VISIBILITY_THRESHOLD
                && right.visibility >= SHOULDER_VISIBILITY_THRESHOLD
            {
                shoulder_pos.midpoint = Some(Vec3::new(
                    (left.x + right.x) * 0.5,
                    (left.y + right.y) * 0.5,
                    (left.z + right.z) * 0.5,
                ));
            }
        }
    }
}

fn setup_scene(mut commands: Commands, config: Res<Config>) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.8, 1.5).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
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
        commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity, Transform::default()));
    }
}

/// System that builds VRM expression maps for entities with MorphWeights.
///
/// This system runs after a VRM scene is spawned and builds the mapping from
/// expression names to morph target indices for each entity with MorphWeights.
///
/// In Bevy's glTF loader, MorphWeights is attached to parent node entities,
/// while MeshMorphWeights is on the child mesh primitive entities. When we update
/// MorphWeights on the parent, it automatically syncs to the children.
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
    morph_weights_query: Query<Entity, With<MorphWeights>>,
) {
    for (vrm_entity, vrm_handle, children) in vrm_entities.iter() {
        let Some(vrm_asset) = vrm_assets.get(&vrm_handle.0) else {
            continue;
        };

        let Some(_gltf) = gltf_assets.get(&vrm_asset.gltf) else {
            continue;
        };

        // Collect all entities with MorphWeights in the scene
        let mut morph_entities = Vec::new();
        collect_morph_weight_entities(
            children,
            &children_query,
            &morph_weights_query,
            &mut morph_entities,
        );

        // Build expression maps
        // We create a combined expression map with all morph target bindings
        // and apply it to all entities with MorphWeights
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

        // Apply the expression map to all entities with MorphWeights
        for &morph_entity in &morph_entities {
            commands
                .entity(morph_entity)
                .insert(combined_expr_map.clone());
        }

        // Mark the VRM entity as processed
        commands.entity(vrm_entity).insert(VrmExpressionMap {
            expression_to_morphs: HashMap::new(),
        });

        info!(
            "Built expression maps for VRM: {} ({} morph entities)",
            vrm_asset.meta.name,
            morph_entities.len()
        );
    }
}

/// Helper function to collect all entities with MorphWeights from the scene hierarchy
fn collect_morph_weight_entities(
    children: &Children,
    children_query: &Query<&Children>,
    morph_weights_query: &Query<Entity, With<MorphWeights>>,
    morph_entities: &mut Vec<Entity>,
) {
    for child in children.iter() {
        // Check if this child has MorphWeights
        if morph_weights_query.get(child).is_ok() {
            morph_entities.push(child);
        }

        // Recursively check children
        if let Ok(grandchildren) = children_query.get(child) {
            collect_morph_weight_entities(
                grandchildren,
                children_query,
                morph_weights_query,
                morph_entities,
            );
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

/// System that translates the VRM root entity based on shoulder world landmarks.
///
/// The midpoint of the two shoulder world landmarks (MediaPipe indices 11 & 12)
/// is used to compute where the model's root (feet) should be placed in Bevy
/// world space, so that the model tracks the subject's real-world torso position.
///
/// Each axis is multiplied by its sign constant (`BODY_*_SIGN`) and scale constant
/// (`BODY_*_SCALE`) so movement can be flipped or amplified by editing those values.
///
/// Coordinate mapping (with default signs/scales of ±1.0 / 1.0):
/// - MediaPipe world X (person's right) → Bevy world X
/// - MediaPipe world Y (up, origin at hip centre) → Bevy world Y with `SHOULDER_Y_OFFSET`
/// - MediaPipe world Z (toward camera) → Bevy world Z
fn apply_body_position(
    shoulder_pos: Res<CurrentShoulderPosition>,
    mut vrm_query: Query<&mut Transform, With<CurrentVrmEntity>>,
) {
    let Some(midpoint) = shoulder_pos.midpoint else {
        return;
    };

    for mut transform in vrm_query.iter_mut() {
        transform.translation = Vec3::new(
            midpoint.x * BODY_X_SIGN * BODY_X_SCALE,
            (midpoint.y + SHOULDER_Y_OFFSET) * BODY_Y_SIGN * BODY_Y_SCALE,
            midpoint.z * BODY_Z_SIGN * BODY_Z_SCALE,
        );
    }
}
