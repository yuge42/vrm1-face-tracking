use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::prelude::*;
use bevy_vrm1::prelude::*;
use expression_adapter::{ArkitToVrmAdapter, BlendshapeToExpression};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracker_ipc::{TrackerFrame, spawn_tracker};

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
        .add_plugins(VrmPlugin)
        .insert_resource(Config { inner: config })
        .init_resource::<VrmModelPath>()
        .add_systems(Startup, (setup_tracker, setup_scene, setup_file_dialog))
        .add_systems(
            Update,
            (
                dump_tracker_frames,
                apply_tracker_to_vrm,
                check_vrm_load_status,
                handle_file_dialog_input,
                receive_file_dialog_result,
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
    let adapter = ArkitToVrmAdapter;

    while let Ok(frame) = rx.rx.try_recv() {
        // Use the expression adapter to convert ARKit blendshapes to VRM expressions
        let vrm_expressions = adapter.to_vrm_expressions(&frame.blendshapes);

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

fn apply_tracker_to_vrm(
    rx: Res<TrackerReceiver>,
    vrm_query: Query<Entity, With<Vrm>>,
    mut transform_query: Query<&mut Transform>,
    searcher: ChildSearcher,
) {
    let adapter = ArkitToVrmAdapter;

    while let Ok(frame) = rx.rx.try_recv() {
        // Convert ARKit blendshapes to VRM expressions once per frame
        let vrm_expressions = adapter.to_vrm_expressions(&frame.blendshapes);

        // Apply expressions to each loaded VRM model
        for vrm_entity in vrm_query.iter() {
            for expr in &vrm_expressions {
                // Find the expression entity by name
                let expression_name = expr.preset.as_str();
                if let Some(expression_entity) =
                    searcher.find_from_name(vrm_entity, expression_name)
                {
                    // Set the expression weight via Transform.translation.x
                    // This is how bevy_vrm1 handles expression weights (same as VRMA)
                    if let Ok(mut transform) = transform_query.get_mut(expression_entity) {
                        transform.translation.x = expr.weight;
                    }
                }
            }
        }
    }
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<Config>) {
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

    // Load VRM model from user data directory via custom asset source
    // First try to load from userdata source, fall back to default assets
    let default_model_path = format!("userdata://{}", config.inner.default_vrm_model);
    let vrm_handle = asset_server.load(&default_model_path);
    commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity));

    println!(
        "Scene setup complete. Attempting to load VRM model from user data: {default_model_path}"
    );
    println!(
        "User VRM directory: {}",
        config.inner.user_vrm_dir.display()
    );
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
        let vrm_handle = asset_server.load(asset_path);
        commands.spawn((VrmHandle(vrm_handle), CurrentVrmEntity));
    }
}
