# Sample Logger - Extensible Logging System

Простий і розширюваний логер для Rust з підтримкою кастомних рівнів логування через derive макроси, фільтрацією за рівнем та кастомними хендлерами.

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
└── test_app/               # Тестовий додаток (стрес-тест)
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## 🚀 Швидкий старт

### 1. Додай в Cargo.toml

```toml
[dependencies]
sample_logger = { path = "../logger" }
```

### 2. Базове використання

```rust
use sample_logger::{init_logger, LogLevel};

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 0)]
struct Event;

#[derive(LogLevel)]
#[log_level(color = "\033[31m", heading = "ERROR", level = 3)]
struct Error;

fn main() {
    // Ініціалізація: показувати логи з рівнем >= 0 (всі)
    init_logger(0);
    
    Event.log("Когенератор запустився");
    Error.log("Критична помилка!");
}
```

## 📝 Детальне використання

### Фільтрація за рівнем логування

```rust
use sample_logger::{init_logger, LogLevel};

#[derive(LogLevel)]
#[log_level(color = "\033[37m", heading = "DEBUG", level = 0)]
struct Debug;

#[derive(LogLevel)]
#[log_level(color = "\033[34m", heading = "INFO", level = 1)]
struct Info;

#[derive(LogLevel)]
#[log_level(color = "\033[33m", heading = "WARN", level = 2)]
struct Warning;

#[derive(LogLevel)]
#[log_level(color = "\033[31m", heading = "ERROR", level = 3)]
struct Error;

fn main() {
    // Показувати тільки WARN (2) і вище
    init_logger(2);
    
    Debug.log("Не покаже");    // level 0 < 2
    Info.log("Не покаже");     // level 1 < 2
    Warning.log("Покаже!");    // level 2 >= 2
    Error.log("Покаже!");      // level 3 >= 2
}
```

**Рівні можна задавати довільно:**
- Чим більше число - тим важливіший лог
- `init_logger(level)` - показує логи з `level` і вище
- Можна використовувати будь-які числа: 0, 1, 2, 10, 100, etc.

### Кастомні хендлери (консоль + файл)

```rust
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

// Рекомендується реалізувати Drop для коректного закриття файлу
impl Drop for FileHandler {
    fn drop(&mut self) {
        self.flush();
        // file автоматично закриється
    }
}

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 0)]
struct Event;

fn main() {
    // Консоль + файл одночасно!
    init_logger_with_handlers(
        vec![Box::new(FileHandler::new("app.log"))],
        0  // мінімальний рівень
    );
    
    Event.log("Запис йде в консоль І файл!");
}
```

### Thread-safe логування

```rust
use sample_logger::{init_logger, LogLevel};
use std::thread;

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 0)]
struct Event;

fn main() {
    init_logger(0);
    
    let handles: Vec<_> = (0..100)
        .map(|i| {
            thread::spawn(move || {
                Event.log(format!("Потік {}", i));
            })
        })
        .collect();
    
    for h in handles {
        h.join().unwrap();
    }
}
```

## 🎨 Кольори ANSI

Підтримуються всі стандартні ANSI кольори:

**Стандартні:**
- `\033[30m` - Чорний
- `\033[31m` - Червоний
- `\033[32m` - Зелений
- `\033[33m` - Жовтий
- `\033[34m` - Синій
- `\033[35m` - Фіолетовий (Magenta)
- `\033[36m` - Блакитний (Cyan)
- `\033[37m` - Білий

**Яскраві (Bright):**
- `\033[90m` - `\033[97m` - яскраві версії кольорів вище

Кольори **автоматично працюють у Windows** завдяки `enable-ansi-support` крейту!

## 🔧 Як це працює

### Derive макрос

```rust
// Ти пишеш:
#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 0)]
struct Event;

// Макрос генерує:
impl LogLevelTrait for Event {
    fn color(&self) -> &'static str { "\033[32m" }
    fn name(&self) -> &'static str { "EVENT" }
    fn level(&self) -> i32 { 0 }
}

impl Event {
    pub fn log(&self, msg: impl Into<String>) {
        if !is_my_level(0) {
            return; // Фільтрація за рівнем
        }
        let log = LogRecord {
            color: "\033[32m",
            heading: "EVENT",
            msg: msg.into(),
            timestamp: Utc::now(),
        };
        internal_send_log(log);
    }
}
```

### Архітектура

1. **MPSC Channel**: Всі логи йдуть через `std::sync::mpsc::channel`
2. **Окремий потік**: `logger_thread` обробляє логи асинхронно
3. **Хендлери**: Кожен лог передається всім зареєстрованим хендлерам
4. **Thread-safe**: `OnceLock` гарантує однократну ініціалізацію

```
User Code
   ↓
Event.log("msg")
   ↓
internal_send_log(LogRecord) → TX (Sender)
   ↓
[MPSC Channel]
   ↓
logger_thread ← RX (Receiver)
   ↓
for each handler:
   handler.handle(record)
```

## 🐛 Troubleshooting

### Помилка: "cannot find type `LogLevel`"

**Причина:** Не імпортовано derive макрос

**Рішення:**
```rust
use sample_logger::LogLevel;  // ← Додай це
```

### Помилка: "Logger already initialized"

**Причина:** Спроба ініціалізувати логер двічі

**Рішення:** Викликай `init_logger()` або `init_logger_with_handlers()` тільки раз на початку `main()`

### Кольори не працюють у Windows

**Рішення 1:** Використовуй Windows Terminal (підтримує ANSI з коробки)

**Рішення 2:** Вже включено автоматично через `enable-ansi-support` крейт!

## 📊 Стрес-тест

Тестовий додаток (`test_app/`) демонструє:
- **400 потоків** одночасно
- **4000 повідомлень** (10 логів × 400 потоків)
- Рандомні затримки для реалістичності
- Рандомний мінімальний рівень при кожному запуску

```bash
cd test_app
cargo run --release
```

## 🎯 Features

- ✅ Кастомні рівні логування через derive макрос
- ✅ Фільтрація за рівнем (показувати тільки WARN+)
- ✅ Thread-safe (MPSC channel + окремий потік)
- ✅ Кольори в консолі (Windows + Linux)
- ✅ Розширювані хендлери (файл, мережа, БД)
- ✅ Graceful shutdown (flush буферів)
- ✅ Zero-cost abstractions (compile-time генерація)

## 📚 API Reference

### Функції ініціалізації

```rust
pub fn init_logger(min_level: i32)
```
Ініціалізує логер тільки з консольним хендлером.

```rust
pub fn init_logger_with_handlers(
    custom_handlers: Vec<Box<dyn LogHandler>>, 
    min_level: i32
)
```
Ініціалізує логер з консольним + кастомними хендлерами.

### Трейти

```rust
pub trait LogLevelTrait {
    fn color(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> i32;
}
```

```rust
pub trait LogHandler: Send + 'static {
    fn handle(&mut self, record: &LogRecord);
    fn flush(&mut self) {}
}
```

**Рекомендація:** Реалізуй `Drop` для коректного закриття ресурсів:
```rust
impl Drop for MyHandler {
    fn drop(&mut self) {
        self.flush();
    }
}
```

### Структури

```rust
pub struct LogRecord {
    pub color: &'static str,
    pub heading: &'static str,
    pub msg: String,
    pub timestamp: DateTime<Utc>,
}
```


## 🍺 Автор

Створено через кров, сльози, макроси і пиво 🍻

## 🙏 Подяки

- **colored** крейт за підтримку кольорів
- **enable-ansi-support** за автоматичне увімкнення ANSI в Windows
