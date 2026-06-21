pub mod docker;
pub mod sftp;
pub mod ssh;
pub mod stats;
pub mod stats_polling;
pub mod tracks;
pub mod cars;

pub fn parse_directory_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_start_matches("./");
            if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect()
}
