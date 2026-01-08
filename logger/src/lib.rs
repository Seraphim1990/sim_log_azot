// ============================================================
// ОСНОВНИЙ КРЕЙТ - sample_logger
// Тут весь функціонал логера
// ============================================================

mod sub_func;

// Ре-експортуємо все публічне з sub_func
pub use sub_func::*;

// Ре-експортуємо залежності для макросів
pub use chrono;
pub use paste;

// Ре-експортуємо derive макрос
pub use logger_derive::LogLevel;

// ============================================================
// ТРЕЙТ для рівнів логування
// ============================================================
pub trait LogLevelTrait {
    fn color(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> i32;
}

// ============================================================
// ТРЕЙТ для хендлерів (консоль, файл, мережа, etc)
// ============================================================
pub trait LogHandler: Send + 'static {
    /// Обробити запис логу
    fn handle(&mut self, record: &LogRecord);
    
    /// Flush буфера (опціонально)
    fn flush(&mut self) {}
}
/*
// ============================================================
// СТАНДАРТНІ РІВНІ (приклад)
// ============================================================
pub struct Debug;
impl LogLevelTrait for Debug {
    fn color(&self) -> &'static str { "\033[33m" }
    fn name(&self) -> &'static str { "DEBUG" }
}

pub struct Info;
impl LogLevelTrait for Info {
    fn color(&self) -> &'static str { "\033[37m" }
    fn name(&self) -> &'static str { "INFO" }
}

pub struct Error;
impl LogLevelTrait for Error {
    fn color(&self) -> &'static str { "\033[31m" }
    fn name(&self) -> &'static str { "ERROR" }
}

// Глобальні функції для стандартних рівнів
pub fn DebugLog(msg: impl Into<String>) {
    Debug.log(msg);
}

pub fn InfoLog(msg: impl Into<String>) {
    Info.log(msg);
}

pub fn ErrorLog(msg: impl Into<String>) {
    Error.log(msg);
}


 */
