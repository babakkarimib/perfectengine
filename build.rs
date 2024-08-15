use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the target directory (where the compiled binaries go)
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable is not set");

    // Define the source and destination paths
    let src_path = "SDL2.dll";
    let dest_path = Path::new(&out_dir).join("SDL2.dll");

    // Copy the file
    fs::copy(src_path, dest_path).expect("Failed to copy file");
}
