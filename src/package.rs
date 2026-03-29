use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnoToml {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

pub fn create_package(uno_dir: &str, output: &str) -> Result<String, Box<dyn std::error::Error>> {
    let uno_path = Path::new(uno_dir);
    
    // uno.toml oku
    let uno_toml_path = uno_path.join("uno.toml");
    if !uno_toml_path.exists() {
        return Err("uno.toml not found".into());
    }

    let toml_content = fs::read_to_string(&uno_toml_path)?;
    let uno_toml: UnoToml = toml::from_str(&toml_content)?;

    // Metadata oluştur
    let metadata = PackageMetadata {
        name: uno_toml.package.name.clone(),
        version: uno_toml.package.version.clone(),
        author: uno_toml.package.author.clone(),
        description: uno_toml.package.description.clone(),
        dependencies: uno_toml.dependencies.keys().cloned().collect(),
    };

    // build/ klasörünü oluştur
    let build_dir = uno_path.join("build");
    fs::create_dir_all(&build_dir)?;

    // .uno dosyasının yolu
    let output_path = build_dir.join(output);

    // tar.gz oluştur
    let file = fs::File::create(&output_path)?;
    let gz = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(gz);

    // metadata.json ekle
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    let metadata_bytes = metadata_json.as_bytes();
    
    let mut header = tar::Header::new_gnu();
    header.set_size(metadata_bytes.len() as u64);
    header.set_cksum();
    tar.append_data(&mut header, "metadata.json", metadata_bytes)?;

    // bin/ klasörü varsa ekle
    let bin_dir = uno_path.join("bin");
    if bin_dir.exists() {
        for entry in fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let relative_path = format!("bin/{}", file_name.to_string_lossy());
                tar.append_file(&relative_path, &mut fs::File::open(&path)?)?;
            }
        }
    }

    // lib/ klasörü varsa ekle
    let lib_dir = uno_path.join("lib");
    if lib_dir.exists() {
        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let relative_path = format!("lib/{}", file_name.to_string_lossy());
                tar.append_file(&relative_path, &mut fs::File::open(&path)?)?;
            }
        }
    }

    // data/ klasörü varsa ekle
    let data_dir = uno_path.join("data");
    if data_dir.exists() {
        for entry in fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let relative_path = format!("data/{}", file_name.to_string_lossy());
                tar.append_file(&relative_path, &mut fs::File::open(&path)?)?;
            }
        }
    }

    tar.finish()?;

    Ok(output_path.to_string_lossy().to_string())
}
