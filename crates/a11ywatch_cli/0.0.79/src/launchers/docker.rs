use std::process::Command;
use crate::fs::TempFs;

pub(crate) fn build_backend(file_manager: &TempFs) {
    Command::new("docker-compose")
    .args(["-f", &file_manager.backend_compose, "-f", &file_manager.frontend_compose, "build"])
    .status()
    .expect("Failed to execute command");
}

pub(crate) fn start_service(frontend: &bool, file_manager: &TempFs) {
    let mut cmd = Command::new("docker-compose");

    if *frontend {
        cmd
        .args(["-f", &file_manager.backend_compose, "-f", &file_manager.frontend_compose, "up", "-d"])
        .status()
        .expect("Failed to execute command");
    } else {
        cmd
        .args(["-f", &file_manager.backend_compose, "up", "-d"])
        .status()
        .expect("Failed to execute command");
    }
}

pub(crate) fn start_runner(url: &String,  file_manager: &TempFs) {
    let mut cmd = Command::new("docker-compose");

    cmd
        .args(["-f", &file_manager.backend_compose, "-f", &file_manager.runner_compose, "run", "--rm", "runner", url, "-v", &file_manager.app_dir])
        .status()
        .expect("Failed to execute command");
}

// /// run a command to the docker container
// pub(crate) fn run_backend(options_run: &str, file_manager: &TempFs) {
//     Command::new("docker-compose")
//         .args(["-f", &file_manager.backend_compose, "-f", &file_manager.frontend_compose, "up", "-d", &options_run])
//         .status()
//         .expect("Failed to execute command");
// }

/// shut down the local instance and remove containers
pub(crate) fn stop_service(frontend: &bool, file_manager: &TempFs) {
    let mut cmd = Command::new("docker-compose");

    if *frontend {
        cmd
        .args(["-f", &file_manager.backend_compose, "-f", &file_manager.frontend_compose, "down"])
        .status()
        .expect("Failed to execute command");
    } else {
        cmd
        .args(["-f", &file_manager.backend_compose, "down"])
        .status()
        .expect("Failed to execute command");
    }
}