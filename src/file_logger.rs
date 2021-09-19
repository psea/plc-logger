// Log into timestamped file with daily rotation
// [BUG] No header in the csv file created based on daily rotation scheme.
// [TODO] Refactor or find some appropriate crate. I don't know why, but it looks ugly now

use std::io::prelude::*;
use std::fs;
use std::fs::{OpenOptions};
use std::path::Path;
use chrono::prelude::*;

pub struct FileLogger<'a> {
    file: std::fs::File,
    path: &'a Path,
    timestamp_format: &'a str,
    file_timestamp: String,
}

impl FileLogger<'_> {
    pub fn open(filename: &str) -> std::io::Result<FileLogger> {
        // Open log file
        let timestamp_format = "%Y-%m-%d";
        let timestamp = Local::now().format(timestamp_format).to_string();
        let path = Path::new(filename);
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let file_extension = path.extension().unwrap().to_str().unwrap();
        let filename_timestamped = format!("{}.{}.{}", file_stem, timestamp, file_extension);

        //let mut log_file = File::create(&log_filename)?;
        if Path::new(&format!(".\\{}", filename_timestamped)).exists() {
            let now = Local::now().format("%H-%M-%S").to_string();
            let backup_filename = format!("{}.{}_backup_{}.{}", file_stem, timestamp, now, file_extension);
            println!("{} exists. Moving to {}", filename_timestamped, backup_filename);
            fs::rename(&filename_timestamped, backup_filename)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(&filename_timestamped)?;
        println!("Logging into {}", filename_timestamped);
        let logger = FileLogger {
            file: file,
            path: path,
            timestamp_format: timestamp_format,
            file_timestamp: timestamp,
        };
        Result::Ok(logger)
    }

    pub fn write_line(&mut self, s: &str) -> std::io::Result<()> {
        let today = Local::now().format(self.timestamp_format).to_string();
        if today != self.file_timestamp {
            let file_stem = self.path.file_stem().unwrap().to_str().unwrap();
            let file_extension = self.path.extension().unwrap().to_str().unwrap();
            let log_filename = format!("{}.{}.{}", file_stem, today, file_extension);
            self.file = OpenOptions::new().create(true).append(true).open(&log_filename)?;
            self.file_timestamp = today;
            println!("Logging into {}", log_filename);
        }
        write!(self.file, "{}\n", s)?;
        Ok(())
    }
}