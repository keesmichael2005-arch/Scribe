use std::env;
use std::path::Path;
use std::process::{exit, Command};

fn main() {
    let mut manifest_path = None;
    let mut passthrough = Vec::new();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--manifest-path" {
            manifest_path = args.next();
        } else {
            passthrough.push(arg);
        }
    }

    let mut command = Command::new("cargo-deny");
    let manifest_path = manifest_path.or_else(|| {
        let default_manifest = "src-tauri/Cargo.toml";
        Path::new(default_manifest)
            .exists()
            .then(|| default_manifest.to_string())
    });

    if let Some(path) = manifest_path {
        command.arg("--manifest-path").arg(path);
    }
    command.args(passthrough);

    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to run cargo-deny: {err}"));

    exit(status.code().unwrap_or(1));
}
