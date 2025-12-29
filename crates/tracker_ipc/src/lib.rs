use crossbeam_channel::{Receiver, Sender};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    thread,
};

/// A frame coming from python
#[derive(Debug, Deserialize)]
pub struct TrackerFrame {
    pub ts: f64,
    pub blendshapes: HashMap<String, f32>,
}

/// Run Python process and return a Receiver
pub fn spawn_tracker(python: &str, script_path: &str) -> (Child, Receiver<TrackerFrame>) {
    let mut child = Command::new(python)
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn tracker process");

    let stdout = child.stdout.take().expect("no stdout");

    let (tx, rx) = crossbeam_channel::unbounded();
    spawn_stdout_reader(stdout, tx);

    (child, rx)
}

fn spawn_stdout_reader(stdout: std::process::ChildStdout, tx: Sender<TrackerFrame>) {
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let Ok(frame) = serde_json::from_str::<TrackerFrame>(&line) else {
                eprintln!("invalid json: {line}");
                continue;
            };
            let _ = tx.send(frame);
        }
    });
}
