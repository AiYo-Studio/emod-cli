use std::{fs, io, path::PathBuf};

pub fn copy_folder(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
    if !src.exists() || !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory not found",
        ));
    }
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(src_path.file_name().unwrap());
        if src_path.is_file() {
            fs::copy(&src_path, &dest_path)?;
        } else if src_path.is_dir() {
            copy_folder(&src_path, &dest_path)?;
        }
    }
    Ok(())
}