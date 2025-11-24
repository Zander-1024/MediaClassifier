use log::info;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn remove_empty_dirs<P: AsRef<Path>>(root: P) -> std::io::Result<()> {
    let root = root.as_ref();

    for entry in WalkDir::new(root)
        .contents_first(true)
        .min_depth(1)
        .max_depth(9)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() && path != root && is_empty_dir(path) {
            fs::remove_dir(path)?;
            info!("Removed empty directory: {:?}", path);
            println!("🗑️  Removed empty directory: {}", path.display());
        }
    }

    Ok(())
}

fn is_empty_dir<P: AsRef<Path>>(path: P) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

