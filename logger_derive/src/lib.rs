// ============================================================
// DERIVE МАКРОС ДЛЯ LogLevel
// Цей крейт ТІЛЬКИ генерує код, не виконує його!
// ============================================================

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit};

/// Derive макрос для автоматичної генерації LogLevel імплементації
/// 
/// Використання:
/// ```rust
/// #[derive(LogLevel)]
/// #[log_level(color = "\033[32m", heading = "EVENT")]
/// struct Event;
/// 
/// // Тепер можна:
/// Event.log("Щось сталось");
/// ```
#[proc_macro_derive(LogLevel, attributes(log_level))]
//                  ^^^^^^^^  ^^^^^^^^^^^^^^^^^^^
//                  |         |
//                  |         └─ Дозволяємо атрибут #[log_level(...)]
//                  └─ Користувач пише #[derive(LogLevel)]
pub fn derive_log_level(input: TokenStream) -> TokenStream {
    // ============================================================
    // ШАГ 1: Парсинг вхідного коду
    // ============================================================
    
    // parse_macro_input! парсить TokenStream в структуровану AST
    let input = parse_macro_input!(input as DeriveInput);
    
    // Витягуємо ім'я структури (наприклад "Event")
    let name = &input.ident;
    
    // ============================================================
    // ШАГ 2: Парсинг атрибутів #[log_level(...)]
    // ============================================================
    
    let mut color = None;
    let mut heading = None;

    // Ітеруємось по всіх атрибутах структури
    for attr in &input.attrs {
        // Шукаємо #[log_level]
        if attr.path().is_ident("log_level") {
            // Парсимо вкладені параметри: color = "...", heading = "..."
            attr.parse_nested_meta(|meta| {
                // Перевіряємо color
                if meta.path.is_ident("color") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        color = Some(s.value());
                    }
                }
                // Перевіряємо heading
                else if meta.path.is_ident("heading") {
                    let value: Lit = meta.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        heading = Some(s.value());
                    }
                }
                Ok(())
            }).ok();
        }
    }
    
    // ============================================================
    // ШАГ 3: Валідація
    // ============================================================
    
    let color = color.expect("Missing #[log_level(color = \"...\")]");
    let heading = heading.expect("Missing #[log_level(heading = \"...\")]");

    // ============================================================
    // ШАГ 4: Генерація коду
    // ============================================================
    
    // quote! генерує Rust код
    // #name, #color, #heading - підстановка змінних
    let expanded = quote! {
        // Імплементуємо трейт LogLevelTrait для структури
        impl ::sample_logger::LogLevelTrait for #name {
            fn color(&self) -> &'static str {
                #color
            }
            
            fn name(&self) -> &'static str {
                #heading
            }
        }

        // Додаємо метод log() до структури
        impl #name {
            /// Логує повідомлення з цим рівнем
            pub fn log(&self, msg: impl Into<String>) {
                let log = ::sample_logger::LogRecord {
                    color: #color,
                    heading: #heading,
                    msg: msg.into(),
                    timestamp: ::sample_logger::chrono::Utc::now(),
                };
                
                ::sample_logger::internal_send_log(log);
            }
        }
    };

    // ============================================================
    // ШАГ 5: Повертаємо згенерований код
    // ============================================================
    
    TokenStream::from(expanded)
}
