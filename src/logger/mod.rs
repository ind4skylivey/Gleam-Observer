use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_LOG_SIZE: u64 = 1024 * 1024;

pub struct FileLogger {
    path: PathBuf,
    file: Mutex<Option<File>>,
}

impl FileLogger {
    pub fn new(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
        Self {
            path,
            file: Mutex::new(file),
        }
    }

    pub fn default_path() -> PathBuf {
        let base = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".local/state")
            });
        base.join("gleam-observer").join("gleam.log")
    }

    pub fn init_default() -> Self {
        Self::new(Self::default_path())
    }

    fn rotate_if_needed(&self) {
        if let Ok(meta) = fs::metadata(&self.path) {
            if meta.len() >= MAX_LOG_SIZE {
                let rotated = self.path.with_extension("log.1");
                let _ = fs::rename(&self.path, rotated);
                if let Ok(mut guard) = self.file.lock() {
                    *guard = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&self.path)
                        .ok();
                }
            }
        }
    }

    pub fn write_line(&self, line: &str) {
        self.rotate_if_needed();
        if let Ok(mut guard) = self.file.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = writeln!(file, "{}", line);
                let _ = file.flush();
            }
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "[{}] [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        );
        self.write_line(&line);
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.file.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_writes_and_rotates() {
        let dir = std::env::temp_dir().join(format!("gleam-test-log-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.log");
        let _ = fs::remove_file(&path);
        let rotated = dir.join("test.log.1");

        let logger = FileLogger::new(path.clone());
        logger.write_line("hello world");
        let content = fs::read_to_string(&path).expect("log file readable");
        assert!(content.contains("hello world"));

        logger.write_line("second line");
        let content = fs::read_to_string(&path).expect("log file readable");
        assert!(content.contains("second line"));
        assert!(!rotated.exists());

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&rotated);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_default_path_under_state_dir() {
        std::env::set_var("XDG_STATE_HOME", "/tmp/gleam-state-test");
        let path = FileLogger::default_path();
        assert!(path.starts_with("/tmp/gleam-state-test"));
        assert!(path.ends_with("gleam.log"));
        std::env::remove_var("XDG_STATE_HOME");
    }

    #[test]
    fn test_log_trait_impl() {
        let dir = std::env::temp_dir().join(format!("gleam-test-trait-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("trait.log");
        let _ = fs::remove_file(&path);

        let logger = FileLogger::new(path.clone());
        log::info!(target: "test", "via log trait: {}", 42);
        logger.write_line("direct write");
        let mut content = String::new();
        File::open(&path)
            .and_then(|mut f| f.read_to_string(&mut content))
            .expect("readable");
        assert!(content.contains("direct write"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
