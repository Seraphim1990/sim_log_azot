# Sample Logger - Extensible Logging System

Простий і розширюваний логер для Rust з підтримкою кастомних рівнів логування через derive макроси.

## 📁 Структура проекту

```
logger_workspace/
├── Cargo.toml              # Workspace (об'єднує всі крейти)
├── logger/                 # Основний крейт з логікою
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Публічний API
│       └── sub_func.rs     # Внутрішня реалізація
├── logger_derive/          # Derive макрос (proc-macro)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # #[derive(LogLevel)]
└── test_app/               # Тестовий додаток
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## 🚀 Швидкий старт

### 1. Копіюємо структуру

```bash
# Скопіюй всю папку logger_workspace в свій проект
# Наприклад:
cp -r logger_workspace D:/RustPjt/
cd D:/RustPjt/logger_workspace
```

### 2. Компілюємо workspace

```bash
# В корені logger_workspace/
cargo build
```

### 3. Запускаємо тест

```bash
cd test_app
cargo run
```

## 📝 Використання

### Базове використання (тільки консоль)

```rust
use sample_logger::{init_logger, LogLevel};

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT")]
struct Event;

fn main() {
    init_logger();
    
    Event.log("Когенератор запустився");
}
```

### Кастомні хендлери (консоль + файл)

```rust
use sample_logger::{init_logger_with_handlers, LogHandler, LogRecord, LogLevel};
use std::fs::OpenOptions;
use std::io::Write;

// Файловий хендлер
struct FileHandler {
    file: std::fs::File,
}

impl LogHandler for FileHandler {
    fn handle(&mut self, record: &LogRecord) {
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

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT")]
struct Event;

fn main() {
    // Консоль + файл одночасно!
    init_logger_with_handlers(vec![
        Box::new(FileHandler::new("app.log"))
    ]);
    
    Event.log("Запис йде в консоль І файл!");
}
```

### Thread-safe

```rust
use sample_logger::{init_logger, InfoLog};
use std::thread;

fn main() {
    init_logger();
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                InfoLog(format!("Потік {}", i));
            })
        })
        .collect();
    
    for h in handles {
        h.join().unwrap();
    }
}
```

## 🎨 Кольори ANSI

- `\033[31m` - Червоний (ERROR)
- `\033[32m` - Зелений (EVENT, SUCCESS)
- `\033[33m` - Жовтий (WARNING, DEBUG)
- `\033[35m` - Фіолетовий (CRITICAL)
- `\033[37m` - Білий (INFO)

## 🔧 Як це працює

### 1. Основний крейт (`logger/`)

- Містить всю логіку логування
- Ре-експортує `chrono`, `paste` для макросів
- Ре-експортує derive макрос

### 2. Derive крейт (`logger_derive/`)

- `proc-macro = true` - ТІЛЬКИ для макросів
- Генерує код на основі `#[derive(LogLevel)]`
- Створює функції типу `EventLog()`

### 3. Як працює derive

```rust
// Користувач пише:
#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT")]
struct Event;

// Макрос генерує:
impl sample_logger::LogLevelTrait for Event {
    fn color(&self) -> &'static str { "\033[32m" }
    fn name(&self) -> &'static str { "EVENT" }
}

pub fn EventLog(msg: impl Into<String>) {
    let log = sample_logger::LogRecord {
        color: "\033[32m",
        heading: "EVENT",
        msg: msg.into(),
        timestamp: sample_logger::chrono::Utc::now(),
    };
    sample_logger::internal_send_log(log);
}
```

## 🐛 Troubleshooting

### Помилка: "cannot find type `LogLevel`"

**Причина:** Не імпортовано derive макрос

**Рішення:**
```rust
use sample_logger::LogLevel;  // ← Додай це
```

### Помилка: "unresolved import `sample_logger`"

**Причина:** Неправильний шлях в `Cargo.toml`

**Рішення:**
```toml
[dependencies]
sample_logger = { path = "../logger" }  # Перевір шлях!
```

### Помилка: "proc-macro derive panicked"

**Причина:** Відсутні атрибути `color` або `heading`

**Рішення:**
```rust
#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT")]  // ← Обидва обов'язкові!
struct Event;
```

## 📚 Додаткові можливості (TODO)

- [ ] File logger (запис в файл)
- [ ] Log rotation (ротація логів)
- [ ] Фільтрація за рівнем (показувати тільки ERROR+)
- [ ] Structured logging (JSON формат)
- [ ] Async logging (tokio)

## 🦀 Rust специфіка

### Чому 2 крейти?

**Proc-macro крейт (`proc-macro = true`) не може мати звичайний код!**

Це обмеження Rust. Тому:
- `logger/` - звичайний код
- `logger_derive/` - тільки макроси

### Чому `::sample_logger::` в макросі?

```rust
::sample_logger::LogRecord  // Абсолютний шлях
```

Це гарантує що макрос знайде типи навіть якщо користувач не зробив `use`.

## 📄 Ліцензія

MIT / Apache-2.0 (на твій вибір)

## 🍺 Автор

Створено через кров, сльози і пиво 🍻

**P.S.** Якщо CMake був складним, proc-macro - це його старший брат на стероїдах 😄
