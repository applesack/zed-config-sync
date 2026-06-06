use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Config;
use crate::gist::GistClient;

pub struct Workspace {
    pub zip_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl Workspace {
    pub fn prepare() -> Result<Self> {
        let base = Config::dir();
        let zip_dir = base.join("zip");
        let temp_dir = base.join("temp");
        cleanup_dir(&zip_dir)?;
        cleanup_dir(&temp_dir)?;
        fs::create_dir_all(&zip_dir)?;
        fs::create_dir_all(&temp_dir)?;
        Ok(Self { zip_dir, temp_dir })
    }

    pub fn cleanup(&self) -> Result<()> {
        cleanup_dir(&self.zip_dir)?;
        cleanup_dir(&self.temp_dir)
    }

    pub fn download_and_extract(&self, client: &GistClient, name: &str) -> Result<()> {
        let bytes = client.download_file(name)?;
        let zip_path = self.zip_dir.join(format!("{}.zip", name));
        fs::write(&zip_path, &bytes)?;
        extract_zip(&zip_path, &self.temp_dir)
    }
}

pub fn zed_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot locate user data directory"))
        .map(|d| d.join("Zed"))
}

pub fn machine_name() -> String {
    if let Ok(name) = std::env::var("COMPUTERNAME") && !name.is_empty() {
        return name;
    }
    if let Ok(output) = std::process::Command::new("hostname").output() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }
    String::new()
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn create_zip(src_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    write_dir_to_zip(&mut zip, src_dir, src_dir, options)?;
    zip.finish()?;
    Ok(())
}

pub fn extract_zip(zip_path: &Path, dst_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let out_path = dst_dir.join(entry.name());
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out_file = fs::File::create(&out_path)?;
        io::copy(&mut entry, &mut out_file)?;
    }
    Ok(())
}

pub fn cleanup_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|e| anyhow::anyhow!("Failed to remove '{}': {}", path.display(), e))?;
    }
    Ok(())
}

fn write_dir_to_zip(
    zip: &mut zip::ZipWriter<fs::File>,
    base: &Path,
    current: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            write_dir_to_zip(zip, base, &path, options)?;
        } else {
            let rel = path.strip_prefix(base)?;
            let zip_name = rel.to_string_lossy().replace('\\', "/");
            zip.start_file(&zip_name, options)?;
            let mut f = fs::File::open(&path)?;
            io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}
