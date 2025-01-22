pub(crate) fn static_resources_dir() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
}

pub(crate) fn app_writeable_dir() -> std::path::PathBuf {
    let mut path = dirs::data_dir().unwrap();
    path.push(env!("CARGO_PKG_NAME"));
    std::fs::create_dir_all(&path).unwrap();
    path
}
