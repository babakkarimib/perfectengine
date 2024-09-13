use std::env;
use std::fs;
use std::path::Path;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable is not set");
        let src_path = "SDL2.dll";
        let dest_path = Path::new(&out_dir).join("SDL2.dll");
        fs::copy(src_path, dest_path).expect("Failed to copy file");
    }
}
