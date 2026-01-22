#![feature(exit_status_error)]

use std::process::Command;

use walkdir::WalkDir;

fn main() {
    let carac_path = env!("CARGO_BIN_FILE_CARAC");
    for entry in WalkDir::new("tests").max_depth(1) {
        if let Ok(entry) = entry {
            if entry.file_type().is_dir() {
                continue;
            }
            println!("testing {}", entry.file_name().display());
            Command::new(carac_path)
                .arg("build")
                .arg(entry.path())
                .arg("-o")
                .arg("test.bin")
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
                .exit_ok()
                .unwrap();
            Command::new("./test.bin")
                .spawn()
                .unwrap()
                .wait()
                .unwrap()
                .exit_ok()
                .unwrap();
        }
    }
}
