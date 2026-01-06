// ============================================================
// ПРИКЛАД: Кастомний файловий логер
// ============================================================

use sample_logger::{init_logger_with_handlers, LogHandler, LogRecord, LogLevel};
use std::fs::OpenOptions;
use std::io::Write;

// Файловий хендлер
struct FileHandler {
    file: std::fs::File,
}

impl FileHandler {
    fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("Failed to open log file");
        
        Self { file }
    }
}

impl LogHandler for FileHandler {
    fn handle(&mut self, record: &LogRecord) {
        // Пишемо в файл без кольорів
        writeln!(
            self.file,
            "[{}] {} - {}",
            record.heading,
            record.timestamp.format("%Y-%m-%d %H:%M:%S"),
            record.msg
        ).ok();
    }
    
    fn flush(&mut self) {
        self.file.flush().ok();
    }
}

// Кастомні рівні
#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 0)]
struct Event;

#[derive(LogLevel)]
#[log_level(color = "\033[33m", heading = "WARN", level = 0)]
struct Warning;

fn main() {
    // Ініціалізуємо з файловим логером
    // Консоль + файл одночасно!
    init_logger_with_handlers(vec![Box::new(FileHandler::new("app.log"))], 0);
    
    println!("=== Логування в консоль + файл ===\n");
    
    Event.log("Запущено програму");
    Warning.log("Це попередження");
    Event.log("Завершено роботу");
    
    println!("\nПеревір файл app.log!");
    
    std::thread::sleep(std::time::Duration::from_millis(100));
}
