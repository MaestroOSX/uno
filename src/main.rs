use clap::{Parser, Subcommand};
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
    }
}

fn pull_package(package: &str) {
    ui::info(&format!("Pulling {}...", package));
    ui::show_progress_bar(package);
    ui::success(&format!("{} pulled successfully!", package));
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
    ui::info(&format!("Searching for '{}'...", query));
    ui::show_spinner(&format!("Searching repositories..."));
    println!("\nFound packages matching '{}':", query);
    println!("  firefox-v1.0.0");
    println!("  firefox-v2.0.0");
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
