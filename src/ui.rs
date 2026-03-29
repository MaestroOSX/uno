use indicatif::{ProgressBar, ProgressStyle};
use colored::*;

pub fn show_spinner(message: &str) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")
            .unwrap()
    );
    spinner.set_message(message.to_string());
    spinner.tick();
    std::thread::sleep(std::time::Duration::from_secs(2));
    spinner.finish_with_message(format!("{} {}", "[OK]".green(), "Done!".green()));
}

pub fn show_progress_bar(package: &str) {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .unwrap()
            .progress_chars("=>-")
    );

    for i in 0..=100 {
        pb.set_position(i);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    pb.finish_with_message(format!("Downloaded {}", package.cyan()));
}

pub fn success(msg: &str) {
    println!("{} {}", "[OK]".green(), msg.green());
}

pub fn error(msg: &str) {
    println!("{} {}", "[ERROR]".red(), msg.red());
}

pub fn info(msg: &str) {
    println!("{}", msg);
}
