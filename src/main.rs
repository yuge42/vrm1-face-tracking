use bevy::prelude::*;
use bevy_vrm1::prelude::*;
use std::path::PathBuf;
use tracker_ipc::{TrackerFrame, spawn_tracker};

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

#[derive(Component)]
struct CurrentVrmEntity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VrmPlugin)
        .init_resource::<VrmModelPath>()
        .add_systems(Startup, (setup_tracker, setup_scene))
        .add_systems(
            Update,
            (
                dump_tracker_frames,
                check_vrm_load_status,
                handle_file_dialog_input,
                load_vrm_from_path,
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

fn dump_tracker_frames(rx: Res<TrackerReceiver>) {
    while let Ok(frame) = rx.rx.try_recv() {
        let blink_l = frame
            .blendshapes
            .get("eyeBlinkLeft")
            .copied()
            .unwrap_or(0.0);

        let jaw = frame.blendshapes.get("jawOpen").copied().unwrap_or(0.0);

        println!(
            "ts={:.3} blinkL={:.2} jawOpen={:.2}",
            frame.ts, blink_l, jaw
        );
    }
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // Load VRM model via asset server
    // The asset path "vrm/model.vrm" corresponds to assets/vrm/model.vrm
    let vrm_handle = asset_server.load("vrm/model.vrm");
    commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity));

    println!("Scene setup complete. Loading VRM model via asset server: vrm/model.vrm");
    println!("If the model file is not found, the application will continue without it.");
    println!("Press 'O' to open a file dialog and select a different VRM model.");
}

fn check_vrm_load_status(
    mut events: MessageReader<bevy::asset::AssetEvent<bevy::gltf::Gltf>>,
    mut reported: Local<bool>,
) {
    for event in events.read() {
        match event {
            bevy::asset::AssetEvent::Added { .. } => {
                if !*reported {
                    println!("✓ VRM model loaded successfully");
                    *reported = true;
                }
            }
            bevy::asset::AssetEvent::LoadedWithDependencies { .. } => {
                if !*reported {
                    println!("✓ VRM model and dependencies loaded successfully");
                    *reported = true;
                }
            }
            _ => {}
        }
    }
}

fn handle_file_dialog_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vrm_path: ResMut<VrmModelPath>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyO) {
        println!("Opening file dialog...");

        // Open file dialog in a blocking manner
        // Note: This will block the main thread, but it's acceptable for a file dialog
        let file = rfd::FileDialog::new()
            .add_filter("VRM Model", &["vrm"])
            .set_title("Select VRM Model")
            .pick_file();

        if let Some(path) = file {
            println!("Selected file: {}", path.display());
            vrm_path.path = Some(path);
        } else {
            println!("File selection cancelled");
        }
    }
}

fn load_vrm_from_path(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut vrm_path: ResMut<VrmModelPath>,
    current_vrm_query: Query<Entity, With<CurrentVrmEntity>>,
) {
    if let Some(path) = vrm_path.path.take() {
        // Remove the current VRM entity if it exists
        for entity in current_vrm_query.iter() {
            commands.entity(entity).despawn();
        }

        // Copy the file to the assets/vrm directory so Bevy can load it
        let assets_dir = std::path::Path::new("assets/vrm");
        if let Err(e) = std::fs::create_dir_all(assets_dir) {
            eprintln!("Failed to create assets/vrm directory: {}", e);
            return;
        }

        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("model.vrm"));
        let dest_path = assets_dir.join(file_name);

        if let Err(e) = std::fs::copy(&path, &dest_path) {
            eprintln!("Failed to copy VRM file to assets directory: {}", e);
            return;
        }

        // Load the VRM model via asset server
        let asset_path = format!("vrm/{}", file_name.to_string_lossy());
        println!("Loading VRM model from: {}", asset_path);
        let vrm_handle = asset_server.load(asset_path);
        commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity));
    }
}
