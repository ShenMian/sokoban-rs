fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    copy_assets_to_work_dir();
}

fn copy_assets_to_work_dir() {
    use std::path::Path;

    let out_dir = get_cargo_target_dir().unwrap();
    let src = "assets";
    let dst = out_dir.join(src);

    if dst.exists() {
        std::fs::remove_dir_all(dst).unwrap();
    }

    let mut options = fs_extra::dir::CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(src, Path::new(&out_dir), &options).unwrap();
}

fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let profile = std::env::var("PROFILE")?;
    let mut target_dir = None;
    let mut current_path = out_dir.as_path();
    while let Some(parent) = current_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent);
            break;
        }
        current_path = parent;
    }
    Ok(target_dir.ok_or("not found")?.to_path_buf())
}
