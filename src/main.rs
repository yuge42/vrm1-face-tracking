use bevy::prelude::*;
use tracker_ipc::{spawn_tracker, TrackerFrame};

#[derive(Resource)]
struct TrackerReceiver {
    rx: crossbeam_channel::Receiver<TrackerFrame>,
}

#[derive(Resource)]
struct TrackerProcess {
    #[allow(dead_code)]
    child: std::process::Child,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_tracker)
        .add_systems(Update, dump_tracker_frames)
        .run();
}

fn setup_tracker(mut commands: Commands) {
    let (child, rx) = spawn_tracker(
        "python3",
        "tools/mediapipe_tracker.py", // Relative Path
    );

    commands.insert_resource(TrackerReceiver { rx });
    commands.insert_resource(TrackerProcess { child });

    println!("Tracker process started");
}

fn dump_tracker_frames(rx: Res<TrackerReceiver>) {
    while let Ok(frame) = rx.rx.try_recv() {
        let blink_l = frame
            .blendshapes
            .get("eyeBlinkLeft")
            .copied()
            .unwrap_or(0.0);

        let jaw = frame
            .blendshapes
            .get("jawOpen")
            .copied()
            .unwrap_or(0.0);

        println!(
            "ts={:.3} blinkL={:.2} jawOpen={:.2}",
            frame.ts, blink_l, jaw
        );
    }
}
