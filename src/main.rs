use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::time::Instant;
use flate2::read::GzDecoder;
use tar::Archive;
mod ui;
mod package;

#[derive(Parser)]
#[command(name = "uno")]
#[command(version = "0.0.1")]
#[command(about = "Maestro package manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull (install) a package
    Pull {
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// Push (remove) a package
    Push {
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// Update packages
    Up {
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },
    /// Upgrade packages
    Upg {
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },
    /// Search for packages
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
    },
    /// List installed packages
    List,
    /// Make a package
    Make {
        #[arg(long, value_name = "DIR")]
        uno: String,
        #[arg(value_name = "OUTPUT")]
        output: String,
    },
    /// Configure server
    Config {
        #[arg(long)]
        server: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Pull { package } => pull_package(&package),
        Commands::Push { package } => push_package(&package),
        Commands::Up { package } => up_package(package),
        Commands::Upg { package } => upg_package(package),
        Commands::Search { query } => search_packages(&query),
        Commands::List => list_packages(),
        Commands::Make { uno, output } => make_package(&uno, &output),
        Commands::Config { server } => set_config(&server),
    }
}

fn get_home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn get_server_config() -> Result<String, String> {
    let home_dir = get_home_dir();
    let config_file = home_dir.join(".uno_server");
    
    match fs::read_to_string(&config_file) {
        Ok(content) => Ok(content.trim().to_string()),
        Err(_) => Err("Sunucu ayarlanmamış, uno config --server <IP> komutunu kullanın".to_string()),
    }
}

fn set_config(server: &str) {
    let home_dir = get_home_dir();
    let config_file = home_dir.join(".uno_server");
    
    match fs::File::create(&config_file) {
        Ok(mut file) => {
            match file.write_all(server.as_bytes()) {
                Ok(_) => {
                    ui::success(&format!("Sunucu ayarlandı: {}", server));
                }
                Err(e) => {
                    ui::error(&format!("Dosya yazma hatası: {}", e));
                }
            }
        }
        Err(e) => {
            ui::error(&format!("Yapılandırma dosyası oluşturulamadı: {}", e));
        }
    }
}

fn pull_package(package: &str) {
    match get_server_config() {
        Err(e) => {
            ui::error(&e);
            return;
        }
        Ok(server_ip) => {
            ui::info(&format!("{} sunucusundan {} çekiliyor...", server_ip, package));
            
            // Önce tar.gz dosyasını dene (klasör)
            if try_pull_tarball(&server_ip, package) {
                return;
            }
            
            // tar.gz başarısız olursa, düz dosya dene
            try_pull_file(&server_ip, package);
        }
    }
}

fn try_pull_tarball(server_ip: &str, package_name: &str) -> bool {
    let tarball_name = format!("{}.tar.gz", package_name);
    let url = format!("http://{}/{}", server_ip, tarball_name);
    
    ui::info(&format!("📂 Klasör olarak çekiliyor..."));
    
    match reqwest::blocking::get(&url) {
        Ok(response) => {
            if !response.status().is_success() {
                return false; // tar.gz bulunamadı, dosya çekmeyi dene
            }
            
            let total_size = response.content_length().unwrap_or(0);
            if total_size > 0 {
                let size_mb = total_size as f64 / 1_000_000.0;
                ui::info(&format!("📦 Dosya boyutu: {:.2} MB", size_mb));
            }
            
            let start = Instant::now();
            
            match response.bytes() {
                Ok(content) => {
                    let elapsed = start.elapsed();
                    let file_size_mb = content.len() as f64 / 1_000_000.0;
                    let speed_mbps = if elapsed.as_secs_f64() > 0.0 {
                        file_size_mb / elapsed.as_secs_f64()
                    } else {
                        0.0
                    };
                    
                    ui::info(&format!("⚡ İndirme hızı: {:.2} MB/s", speed_mbps));
                    ui::info(&format!("⏱️  İndirme süresi: {:.2}s", elapsed.as_secs_f64()));
                    
                    let home_dir = get_home_dir();
                    let download_dir = home_dir.join("uno-server-test");
                    
                    if !download_dir.exists() {
                        if let Err(e) = fs::create_dir_all(&download_dir) {
                            ui::error(&format!("Klasör oluşturulamadı: {}", e));
                            return false;
                        }
                    }
                    
                    // tar.gz dosyasını geçici olarak kaydet
                    let tar_path = download_dir.join(&format!("{}.tar.gz", package_name));
                    
                    if let Err(e) = fs::write(&tar_path, &content) {
                        ui::error(&format!("Tar.gz yazılamadı: {}", e));
                        return false;
                    }
                    
                    // tar.gz'yi çıkart
                    match fs::File::open(&tar_path) {
                        Ok(file) => {
                            let gz = GzDecoder::new(file);
                            let mut archive = Archive::new(gz);
                            
                            match archive.unpack(&download_dir) {
                                Ok(_) => {
                                    // Geçici tar.gz dosyasını sil
                                    let _ = fs::remove_file(&tar_path);
                                    ui::success(&format!("✅ Klasör çıkartıldı: {}/{}", download_dir.display(), package_name));
                                    true
                                }
                                Err(e) => {
                                    ui::error(&format!("Klasör çıkartılamadı: {}", e));
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            ui::error(&format!("Tar.gz dosyası açılamadı: {}", e));
                            false
                        }
                    }
                }
                Err(_) => false,
            }
        }
        Err(_) => false, // Bağlantı hatası, dosya çekmeyi dene
    }
}

fn try_pull_file(server_ip: &str, package: &str) {
    let url = format!("http://{}/{}", server_ip, package);
    
    match reqwest::blocking::get(&url) {
        Ok(response) => {
            if response.status().is_success() {
                let total_size = response.content_length().unwrap_or(0);
                
                if total_size > 0 {
                    let size_mb = total_size as f64 / 1_000_000.0;
                    ui::info(&format!("📦 Dosya boyutu: {:.2} MB", size_mb));
                }
                
                let start = Instant::now();
                
                match response.bytes() {
                    Ok(content) => {
                        let elapsed = start.elapsed();
                        let file_size_mb = content.len() as f64 / 1_000_000.0;
                        let speed_mbps = if elapsed.as_secs_f64() > 0.0 {
                            file_size_mb / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };
                        
                        ui::info(&format!("⚡ İndirme hızı: {:.2} MB/s", speed_mbps));
                        ui::info(&format!("⏱️  İndirme süresi: {:.2}s", elapsed.as_secs_f64()));
                        
                        let home_dir = get_home_dir();
                        let download_dir = home_dir.join("uno-server-test");
                        
                        if !download_dir.exists() {
                            if let Err(e) = fs::create_dir_all(&download_dir) {
                                ui::error(&format!("Klasör oluşturulamadı: {}", e));
                                return;
                            }
                        }
                        
                        let file_path = download_dir.join(package);
                        
                        match fs::write(&file_path, content) {
                            Ok(_) => {
                                ui::success(&format!("✅ Dosya kaydedildi: {}", file_path.display()));
                            }
                            Err(e) => {
                                ui::error(&format!("Dosya yazılamadı: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        ui::error(&format!("İçerik alınamadı: {}", e));
                    }
                }
            } else {
                ui::error(&format!("Sunucu hatası: {} - {} paketi sunucuda bulunamadı", response.status(), package));
            }
        }
        Err(e) => {
            ui::error(&format!("Bağlantı hatası: {}", e));
        }
    }
}

fn push_package(package: &str) {
    ui::info(&format!("Pushing {}...", package));
    ui::show_spinner(&format!("Removing {}...", package));
    ui::success(&format!("{} pushed successfully!", package));
}

fn up_package(package: Option<String>) {
    match package {
        Some(p) => {
            ui::info(&format!("Updating {}...", p));
            ui::show_progress_bar(&p);
            ui::success(&format!("{} updated!", p));
        }
        None => {
            ui::info("Updating all packages...");
            ui::show_progress_bar("packages");
            ui::success("All packages updated!");
        }
    }
}

fn upg_package(package: Option<String>) {
    match package {
        Some(p) => {
            ui::info(&format!("Upgrading {}...", p));
            ui::show_progress_bar(&p);
            ui::success(&format!("{} upgraded!", p));
        }
        None => {
            ui::info("Upgrading all packages...");
            ui::show_progress_bar("packages");
            ui::success("All packages upgraded!");
        }
    }
}

fn search_packages(query: &str) {
    match get_server_config() {
        Err(e) => {
            ui::error(&e);
            return;
        }
        Ok(server_ip) => {
            if query == "all" {
                // Tüm dosyaları listele - files.txt'den oku
                ui::info("Tüm dosyalar sunucudan sorgulanıyor...");
                search_all(&server_ip);
            } else {
                // Spesifik dosyayı ara
                ui::info(&format!("'{}' dosyası sunucuda aranıyor...", query));
                search_file(&server_ip, query);
            }
        }
    }
}

fn search_all(server_ip: &str) {
    let url = format!("http://{}/files.txt", server_ip);
    
    match reqwest::blocking::get(&url) {
        Ok(response) => {
            if response.status().is_success() {
                match response.text() {
                    Ok(content) => {
                        ui::success("📋 Sunucudaki dosyalar:");
                        println!();
                        for line in content.lines() {
                            let parts: Vec<&str> = line.trim().split_whitespace().collect();
                            if parts.len() >= 2 {
                                let filename = parts[0];
                                let size_str = parts[1];
                                if let Ok(size) = size_str.parse::<u64>() {
                                    let size_mb = size as f64 / 1_000_000.0;
                                    if size < 1_000_000 {
                                        println!("  📄 {} ({} bytes)", filename, size);
                                    } else {
                                        println!("  📦 {} ({:.2} MB)", filename, size_mb);
                                    }
                                } else {
                                    println!("  📄 {}", filename);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        ui::error(&format!("Dosya listesi okunamadı: {}", e));
                    }
                }
            } else {
                ui::error("Sunucuda dosya listesi (files.txt) bulunamadı. Termux'ta şu komutu çalıştır:\nls -1 > files.txt");
            }
        }
        Err(e) => {
            ui::error(&format!("Bağlantı hatası: {}", e));
        }
    }
}

fn search_file(server_ip: &str, filename: &str) {
    let url = format!("http://{}/{}", server_ip, filename);
    
    // HEAD isteği at - sadece header'ları al
    match reqwest::blocking::Client::new().head(&url).send() {
        Ok(response) => {
            if response.status().is_success() {
                let content_length = response.content_length().unwrap_or(0);
                let size_mb = content_length as f64 / 1_000_000.0;
                
                ui::success("✅ Dosya bulundu!");
                println!();
                if content_length < 1_000_000 {
                    println!("  📄 Dosya adı: {}", filename);
                    println!("  📊 Boyut: {} bytes", content_length);
                } else {
                    println!("  📦 Dosya adı: {}", filename);
                    println!("  📊 Boyut: {:.2} MB", size_mb);
                }
                
                // .tar.gz dosyası olup olmadığını kontrol et
                if filename.ends_with(".tar.gz") || filename.ends_with("tar") {
                    println!("  📂 Tür: Sıkıştırılmış Klasör");
                } else {
                    println!("  📃 Tür: Dosya");
                }
            } else if response.status() == 404 {
                ui::error(&format!("'{}' dosyası sunucuda bulunamadı", filename));
            } else {
                ui::error(&format!("Sunucu hatası: {}", response.status()));
            }
        }
        Err(e) => {
            ui::error(&format!("Bağlantı hatası: {}", e));
        }
    }
}

fn list_packages() {
    ui::info("Fetching installed packages...");
    ui::show_spinner("Reading package database...");
    println!("\nInstalled packages:");
    println!("  firefox v1.0.0");
    println!("  chromium v2.5.1");
}

fn make_package(uno_dir: &str, output: &str) {
    ui::info(&format!("Creating package from {}...", uno_dir));
    
    match package::create_package(uno_dir, output) {
        Ok(path) => {
            ui::success(&format!("Package created: {}", path));
        }
        Err(e) => {
            ui::error(&format!("Failed to create package: {}", e));
        }
    }
}
