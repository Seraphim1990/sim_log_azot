// ============================================================
// ТЕСТОВИЙ ДОДАТОК
// ============================================================

use sample_logger::{init_logger, LogLevel};
use rand::Rng;


// ============================================================
// КАСТОМНІ РІВНІ ЛОГУВАННЯ через derive
// ============================================================

#[derive(LogLevel)]
#[log_level(color = "\033[32m", heading = "EVENT", level = 4)]
struct Event;

#[derive(LogLevel)]
#[log_level(color = "\033[33m", heading = "WARN", level = 3)]
struct Warning;

#[derive(LogLevel)]
#[log_level(color = "\033[35m", heading = "CRITICAL", level = 2)]
struct Critical;

#[derive(LogLevel)]
#[log_level(color = "\033[31m", heading = "INFO", level = 1)]
struct Info;


fn main() {

    let rand_lvl = rand::thread_rng().gen_range(1..=4);
    println!("RNG Lvl: {}", rand_lvl);
    // Ініціалізуємо логер
    init_logger(rand_lvl);

    println!("=== Стрес-тест: 400 потоків ===\n");

    // ============================================================
    // 400 потоків які одночасно логують
    // ============================================================
    let handles: Vec<_> = (0..400)
        .map(|i| {
            std::thread::spawn(move || {
                // Кожен потік робить 10 логів
                for j in 0..10 {
                    // Чергуємо різні типи логів
                    // Тепер використовуємо .log() метод!
                    match j % 4 {
                        0 => Event.log(format!("Потік {} - подія {}", i, j)),
                        1 => Warning.log(format!("Потік {} - попередження {}", i, j)),
                        2 => Critical.log(format!("Потік {} - критично {}", i, j)),
                        3 => Info.log(format!("Потік {} - інфо {}", i, j)),
                        _ => unreachable!(),
                    }

                    let sleep_ms = rand::thread_rng().gen_range(2..=15);
                    std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
                    //std::thread::sleep(std::time::Duration::from_millis(5));
                }
            })
        })
        .collect();

    // Чекаємо всі потоки
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n=== Тест завершено: 4000 повідомлень з 400 потоків ===");

    // Даємо час логеру обробити всі повідомлення
    std::thread::sleep(std::time::Duration::from_millis(5000));
}