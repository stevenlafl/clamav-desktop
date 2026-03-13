use std::path::Path;
use std::time::Instant;

use super::*;

pub fn filter_log(log: String) -> bool {
    log.starts_with("Scanning ") || log.ends_with(": Empty file") || log.ends_with(": Access denied")
}

fn extract_path_from_log(log: &str) -> Option<String> {
    match true {
        true if log.starts_with("Scanning ") => Some(log.replace("Scanning ", "").trim().to_string()),
        true if log.ends_with(": Empty file") => Some(log.replace(": Empty file", "").trim().to_string()),
        true if log.ends_with(": Access denied") => Some(log.replace(": Access denied", "").trim().to_string()),
        _ => None,
    }
}

pub fn get_file_size_from_log(log: &str) -> u64 {
    extract_path_from_log(log)
        .and_then(|p| std::fs::metadata(Path::new(&p)).ok())
        .map(|m| m.len())
        .unwrap_or(0)
}

pub fn get_status_from_log(
    log: String,
    file_index: usize,
    total_files_length: usize,
    bytes_scanned: u64,
    total_bytes: u64,
    scan_start_time: Instant,
) -> state::ScannerPublicState {
    let current_path = extract_path_from_log(&log);
    let progress = (file_index as f64 + 1.0) / total_files_length as f64;

    if progress == 1.0 {
        return state::ScannerPublicState {
            current_path: None,
            estimated_seconds_remaining: Some(0.0),
            progress: Some(progress),
            step: state::ScannerStatusStep::Idle,
        };
    }

    let elapsed = scan_start_time.elapsed().as_secs_f64();
    let files_scanned = file_index + 1;
    let estimated_seconds_remaining = if elapsed < 2.0 || files_scanned < 10 {
        None
    } else if bytes_scanned > 0 && total_bytes > 0 {
        let bytes_remaining = total_bytes.saturating_sub(bytes_scanned);
        let bytes_per_second = bytes_scanned as f64 / elapsed;
        Some(bytes_remaining as f64 / bytes_per_second)
    } else {
        let remaining_files = total_files_length.saturating_sub(files_scanned);
        let avg_time_per_file = elapsed / files_scanned as f64;
        Some(avg_time_per_file * remaining_files as f64)
    };

    state::ScannerPublicState {
        current_path,
        estimated_seconds_remaining,
        progress: Some(progress),
        step: state::ScannerStatusStep::Running,
    }
}
