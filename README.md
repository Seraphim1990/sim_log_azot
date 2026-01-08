# Sample Logger - Extensible Logger for Rust

---

# 💭 Philosophy
---
This logger is designed for multi-threaded programs with possible concurrent access to resources:  
files, databases, sockets, etc., through custom handlers.  
The numeric log level value is borrowed from slog (Go).  
I liked the ability to change the logger level at runtime  
through a config file or command-line arguments without recompiling the project.
---

##

## 📁 Project Structure

```
logger_workspace/
├── Cargo.toml              # Workspace
├── logger/                 # Main crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public API
│       └── sub_func.rs     # Internal implementation
├── logger_derive/          # Proc-macro crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # #[derive(LogLevel)]
└── test_app/               # Test application
    ├── Cargo.toml
    └── src/
        └── main.rs         # Stress test (400 threads)
```

## 🚀 Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
sample_logger = { path = "../logger" }
```

### 2. Basic Usage

```rust
use sample_logger::{init_logger, LogLevel};

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 1)]
struct Event;

#[derive(LogLevel)]
#[log_level(color = "\033[31m", heading = "ERROR", level = 3)]
struct Error;

fn main() {
    // Initialize: show logs with level >= 1
    init_logger(1);
    
    Event.log("Program started");
    Error.log("Critical error!");
}
```

**Output:**
```
[EVENT] : 26-01-08 14:30:25 -> Program started
[ERROR] : 26-01-08 14:30:25 -> Critical error!
```

## 📊 Log Levels and Filtering

### How It Works

Each level has a numeric value (`i32`). Higher number = more important log.

```rust
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
```

### Setting Minimum Level

```rust
// Show only WARN (2) and higher
init_logger(2);

Debug.log("Won't show");       // level 0 < 2
Info.log("Won't show");        // level 1 < 2
Warning.log("Will show!");     // level 2 >= 2
Error.log("Will show!");       // level 3 >= 2
```

### Configuration via Command Line

**You can change log level without recompiling!**

```rust
use std::env;

fn main() {
    // Read from command line arguments
    let min_level: i32 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1); // Default: INFO
    
    init_logger(min_level);
    
    Debug.log("Debug information");
    Info.log("Info message");
    Error.log("Error!");
}
```

**Usage:**
```bash
# Show all logs (including DEBUG)
cargo run -- 0

# Show INFO and higher (default)
cargo run -- 1

# Show only WARN and ERROR
cargo run -- 2

# Show only ERROR
cargo run -- 3
```

### Environment Variables

```rust
fn main() {
    let min_level: i32 = env::var("LOG_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    
    init_logger(min_level);
}
```

**Usage:**
```bash
LOG_LEVEL=0 cargo run  # Debug mode
LOG_LEVEL=3 cargo run  # Errors only
```

### Optimizing Expensive Operations with `is_active()`

Use `is_active()` to avoid executing expensive operations when the log level is disabled:

```rust
if Debug.is_active() {
    // This executes ONLY if DEBUG is active
    let expensive_data = parse_huge_file();
    Debug.log(format!("Parsed: {:?}", expensive_data));
}

// Without is_active(), parsing happens even if DEBUG is off!
```

**Real-world example:**

```rust
// Expensive JSON parsing
if Debug.is_active() {
    let json_str = format!("{:#?}", complex_struct);  // Expensive!
    Debug.log(format!("State: {}", json_str));
}

// Expensive SQL query for diagnostics
if Debug.is_active() {
    let stats = database.get_detailed_stats();  // Slow!
    Debug.log(format!("DB stats: {:?}", stats));
}
```

## 🎨 ANSI Colors

All standard ANSI colors are supported:

**Standard:**
- `\033[30m` - Black
- `\033[31m` - Red
- `\033[32m` - Green
- `\033[33m` - Yellow
- `\033[34m` - Blue
- `\033[35m` - Magenta
- `\033[36m` - Cyan
- `\033[37m` - White

**Bright:**
- `\033[90m` to `\033[97m` - bright versions of the colors above

**Colors work automatically on Windows** thanks to the `enable-ansi-support` crate!

## 🔧 Custom Handlers

### Basic Example: File Logger

```rust
use sample_logger::{init_logger_with_handlers, LogHandler, LogRecord, LogLevel};
use std::fs::OpenOptions;
use std::io::Write;

// File handler
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
        // Write to file without colors
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

// Recommended: implement Drop for proper cleanup
impl Drop for FileHandler {
    fn drop(&mut self) {
        self.flush();
        // file will close automatically
    }
}

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 1)]
struct Event;

fn main() {
    // Console + file simultaneously!
    init_logger_with_handlers(
        vec![Box::new(FileHandler::new("app.log"))],
        1  // minimum level
    );
    
    Event.log("Writing to console AND file!");
}
```

### Filtering in Handler by Level

```rust
struct FileHandler {
    file: File,
    min_level: i32, // Handler's own filter!
}

impl LogHandler for FileHandler {
    fn handle(&mut self, record: &LogRecord) {
        // File only for ERROR and above
        if record.lvl < 3 {
            return;
        }
        
        writeln!(
            self.file, 
            "[{}] {}", 
            record.heading, 
            record.msg
        ).ok();
    }
}
```

**Usage:**

```rust
fn main() {
    // init_logger(0) - all logs to console
    // FileHandler filters and writes only ERROR
    init_logger_with_handlers(
        vec![Box::new(FileHandler::new("errors.log"))],
        0  // Show everything in console
    );
    
    Debug.log("Console only");           // Console only
    Info.log("Console only");            // Console only
    Error.log("Console + file!"); // Console + file
}
```

### LogHandler Trait

```rust
pub trait LogHandler: Send + 'static {
    /// Handle log record
    fn handle(&mut self, record: &LogRecord);
    
    /// Flush buffers (optional)
    fn flush(&mut self) {}
}
```

### LogRecord Structure

```rust
pub struct LogRecord {
    pub color: &'static str,      // ANSI color code
    pub heading: &'static str,    // "EVENT", "ERROR", etc
    pub msg: String,              // Message
    pub timestamp: DateTime<Utc>, // Time
    pub lvl: i32,                 // Numeric level
}
```

**Recommendation:** Implement `Drop` for proper resource cleanup:

```rust
impl Drop for MyHandler {
    fn drop(&mut self) {
        self.flush();
    }
}
```

## 🧵 Thread-safe Logging

Logger is fully thread-safe thanks to `std::sync::mpsc::channel`:

```rust
use sample_logger::{init_logger, LogLevel};
use std::thread;

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 1)]
struct Event;

fn main() {
    init_logger(1);
    
    let handles: Vec<_> = (0..100)
        .map(|i| {
            thread::spawn(move || {
                Event.log(format!("Thread {}", i));
            })
        })
        .collect();
    
    for h in handles {
        h.join().unwrap();
    }
}
```

## 💡 Recommended Level Scheme

```rust
// Standard levels (recommendation)
const TRACE: i32 = -1;    // Very detailed debug
const DEBUG: i32 = 0;     // Debug information
const INFO: i32 = 1;      // General information
const WARN: i32 = 2;      // Warnings
const ERROR: i32 = 3;     // Errors
const CRITICAL: i32 = 4;  // Critical errors

// Custom levels (examples)
const AUDIT: i32 = 10;    // Security/audit logs
const METRICS: i32 = 20;  // Performance metrics
```

**You can use ANY `i32` values!** The library doesn't enforce a specific scheme.

## 🎯 Usage Examples

### Configuration from TOML

```rust
// config.toml
// [logging]
// min_level = 2

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    logging: LoggingConfig,
}

#[derive(Deserialize)]
struct LoggingConfig {
    min_level: i32,
}

fn main() {
    let config_str = std::fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();
    
    init_logger(config.logging.min_level);
}
```

### Different Levels for Different Modules

```rust
// Networking module - detailed DEBUG
#[derive(LogLevel)]
#[log_level(color = "\033[36m", heading = "NET_DBG", level = 0)]
struct NetworkDebug;

// Business logic - INFO only
#[derive(LogLevel)]
#[log_level(color = "\033[34m", heading = "BIZ_INFO", level = 1)]
struct BusinessInfo;

// init_logger(0) - shows both
// init_logger(1) - shows BusinessInfo only
```

## 📊 Stress Test

Test application (`test_app/`) demonstrates:
- **400 threads** simultaneously
- **4000 messages** (10 logs × 400 threads)
- Random delays for realism
- Random minimum level on each run

```bash
cd test_app
cargo run --release
```

## 🔍 Troubleshooting

### Error: "cannot find type `LogLevel`"

**Reason:** Derive macro not imported

**Solution:**
```rust
use sample_logger::LogLevel;  // ← Add this
```

### Error: "Logger already initialized"

**Reason:** Attempting to initialize logger twice

**Solution:** Call `init_logger()` or `init_logger_with_handlers()` only once at the start of `main()`

### Colors Don't Work on Windows

**Solution 1:** Use Windows Terminal (supports ANSI out of the box)

**Solution 2:** Already enabled automatically via `enable-ansi-support` crate!

## 🎯 Features

- ✅ Custom log levels via derive macro
- ✅ Level filtering (show only important logs)
- ✅ Configuration via CLI args / env vars / config files (no recompilation!)
- ✅ Expensive operation optimization (`is_active()`)
- ✅ Thread-safe (MPSC channel + separate thread)
- ✅ Console colors (Windows + Linux)
- ✅ Extensible handlers (file, network, DB)
- ✅ Graceful shutdown (buffer flushing)

## 📚 API Reference

### Initialization Functions

```rust
pub fn init_logger(min_level: i32)
```
Initializes logger with console handler only.

```rust
pub fn init_logger_with_handlers(
    custom_handlers: Vec<Box<dyn LogHandler>>, 
    min_level: i32
)
```
Initializes logger with console + custom handlers.

### Activity Check

```rust
pub fn is_my_level(lvl: i32) -> bool
```
Checks if the specified log level is active.

### Traits

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


## 📄 License

MIT

---
