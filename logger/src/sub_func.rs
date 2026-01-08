// ============================================================
// ВНУТРІШНЯ РЕАЛІЗАЦІЯ ЛОГЕРА
// ============================================================

use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::OnceLock;
use chrono::{DateTime, Datelike, Timelike, Utc};
use colored::*;

use crate::LogHandler;

// ============================================================
// СТРУКТУРА ЗАПИСУ ЛОГУ (публічна для хендлерів)
// ============================================================
pub struct LogRecord {
    pub color: &'static str,
    pub heading: &'static str,
    pub msg: String,
    pub timestamp: DateTime<Utc>,
    pub lvl: i32,
}

// ============================================================
// ГЛОБАЛЬНИЙ КАНАЛ (приватний)
// ============================================================
static TX: OnceLock<Sender<LogRecord>> = OnceLock::new();
static MIN_LEVEL_LOG: OnceLock<i32> = OnceLock::new();

// ============================================================
// КОНСОЛЬНИЙ ХЕНДЛЕР (за замовчуванням)
// ============================================================
struct ConsoleHandler;

impl LogHandler for ConsoleHandler {
    fn handle(&mut self, record: &LogRecord) {
        let ts = format_log_record_time(record, "YY-MM-DD HH:MM:SS");
        let colored_heading = ansi_to_colored(record.color, record.heading).bold();
        println!("[{}] : {} -> {}", colored_heading, ts, record.msg);
    }
}

// ============================================================
// ПОТІК ЛОГУВАННЯ З ХЕНДЛЕРАМИ
// ============================================================
fn logger_thread(rx: Receiver<LogRecord>, mut handlers: Vec<Box<dyn LogHandler>>) {
    while let Ok(record) = rx.recv() {
        // Передаємо запис кожному хендлеру
        for handler in &mut handlers {
            handler.handle(&record);
        }
    }
    
    // Flush всіх хендлерів при завершенні
    for handler in &mut handlers {
        handler.flush();
    }
}

// ============================================================
// КОНВЕРТАЦІЯ ANSI КОДУ В COLORED
// ============================================================
fn ansi_to_colored(ansi: &str, text: &str) -> ColoredString {
    match ansi {
        // Стандартні кольори
        "\033[30m" | "\x1b[30m" => text.black(),
        "\033[31m" | "\x1b[31m" => text.red(),
        "\033[32m" | "\x1b[32m" => text.green(),
        "\033[33m" | "\x1b[33m" => text.yellow(),
        "\033[34m" | "\x1b[34m" => text.blue(),
        "\033[35m" | "\x1b[35m" => text.magenta(),
        "\033[36m" | "\x1b[36m" => text.cyan(),
        "\033[37m" | "\x1b[37m" => text.white(),
        
        // Яскраві кольори (bright)
        "\033[90m" | "\x1b[90m" => text.bright_black(),
        "\033[91m" | "\x1b[91m" => text.bright_red(),
        "\033[92m" | "\x1b[92m" => text.bright_green(),
        "\033[93m" | "\x1b[93m" => text.bright_yellow(),
        "\033[94m" | "\x1b[94m" => text.bright_blue(),
        "\033[95m" | "\x1b[95m" => text.bright_magenta(),
        "\033[96m" | "\x1b[96m" => text.bright_cyan(),
        "\033[97m" | "\x1b[97m" => text.bright_white(),
        
        // Якщо невідомий код - білий
        _ => text.white(),
    }
}

// ============================================================
// ФОРМАТУВАННЯ ЧАСУ
// ============================================================
fn format_log_record_time(record: &LogRecord, pattern: &str) -> String {
    pattern
        .replace("YYYY", &record.timestamp.year().to_string())
        .replace("YY", &format!("{:02}", record.timestamp.year() % 100))
        .replace("DD", &format!("{:02}", record.timestamp.day()))
        .replace("HH", &format!("{:02}", record.timestamp.hour()))
        .replace("MM", &format!("{:02}", record.timestamp.minute()))
        .replace("SS", &format!("{:02}", record.timestamp.second()))
}

// ============================================================
// ПУБЛІЧНА ФУНКЦІЯ: Відправка логу
// Використовується макросами
// ============================================================
#[doc(hidden)]
pub fn internal_send_log(data: LogRecord) {
    // Якщо TX не ініціалізований - ініціалізуємо з консольним хендлером
    if TX.get().is_none() {
        init_logger(0);
    }
    
    let tx = TX.get().expect("Logger not initialized");
    
    if let Err(_) = tx.send(data) {
        eprintln!("Logger thread died!");
    }
}

// ============================================================
// ПУБЛІЧНА ФУНКЦІЯ: Ініціалізація логера (тільки консоль)
// ============================================================
pub fn init_logger(min_level: i32) {
    // Перевірка подвійної ініціалізації
    if TX.get().is_some() {
        panic!("Logger already initialized! Cannot initialize twice.");
    }
    
    // Увімкнути ANSI підтримку в Windows
    #[cfg(windows)]
    {
        let _ = enable_ansi_support::enable_ansi_support();
    }
    
    let (tx, rx) = channel();
    
    std::thread::spawn(move || {
        // Тільки консольний хендлер
        let handlers: Vec<Box<dyn LogHandler>> = vec![Box::new(ConsoleHandler)];
        logger_thread(rx, handlers);
    });
    
    TX.set(tx).expect("Failed to set logger transmitter");
    MIN_LEVEL_LOG.set(min_level).expect("Failed to set level log");
}

// ============================================================
// ПУБЛІЧНА ФУНКЦІЯ: Ініціалізація з кастомними хендлерами
// ============================================================
pub fn init_logger_with_handlers(mut custom_handlers: Vec<Box<dyn LogHandler>>, min_level: i32) {
    // Перевірка подвійної ініціалізації
    if TX.get().is_some() {
        panic!("Logger already initialized! Cannot initialize twice.");
    }
    
    // Увімкнути ANSI підтримку в Windows
    #[cfg(windows)]
    {
        let _ = enable_ansi_support::enable_ansi_support();
    }
    
    let (tx, rx) = channel();
    
    std::thread::spawn(move || {
        // Консольний хендлер завжди перший
        let mut handlers: Vec<Box<dyn LogHandler>> = vec![Box::new(ConsoleHandler)];
        // Додаємо кастомні хендлери
        handlers.append(&mut custom_handlers);
        
        logger_thread(rx, handlers);
    });
    
    TX.set(tx).expect("Failed to set logger transmitter");
    MIN_LEVEL_LOG.set(min_level).expect("Failed to set level log");
}

pub fn is_my_level(lvl: i32) -> bool {
    lvl >= *MIN_LEVEL_LOG.get().unwrap_or(&0)
}