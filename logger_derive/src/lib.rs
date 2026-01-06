// ============================================================
// DERIVE МАКРОС ДЛЯ LogLevel
// Цей крейт тільки генерує код!
// ============================================================

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit, LitInt};

#[proc_macro_derive(LogLevel, attributes(log_level))]
pub fn derive_log_level(input: TokenStream) -> TokenStream {
    // ============================================================
    // 1️⃣ Парсинг вхідного коду
    // ============================================================
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // ============================================================
    // 2️⃣ Парсинг атрибутів #[log_level(...)]
    // ============================================================
    let mut color = None;
    let mut heading = None;
    let mut my_level: Option<LitInt> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("log_level") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("color") {
                    if let Lit::Str(s) = meta.value()?.parse()? {
                        color = Some(s.value());
                    }
                } else if meta.path.is_ident("heading") {
                    if let Lit::Str(s) = meta.value()?.parse()? {
                        heading = Some(s.value());
                    }
                } else if meta.path.is_ident("level") {
                    if let Lit::Int(s) = meta.value()?.parse()? {
                        my_level = Some(s);
                    }
                }
                Ok(())
            }).ok();
        }
    }

    // ============================================================
    // 3️⃣ Валідація
    // ============================================================
    let color = color.expect("Missing #[log_level(color = \"...\")]");
    let heading = heading.expect("Missing #[log_level(heading = \"...\")]");
    let my_level = my_level
        .expect("Missing #[log_level(level = ...)]")
        .base10_parse::<i32>()
        .expect("level must be an integer");

    // ============================================================
    // 4️⃣ Генерація коду
    // ============================================================
    let expanded = quote! {
        impl ::sample_logger::LogLevelTrait for #name {
            fn color(&self) -> &'static str {
                #color
            }

            fn name(&self) -> &'static str {
                #heading
            }

            // Тепер повертаємо i32 по значенню, без посилання
            fn level(&self) -> i32 {
                #my_level
            }
        }

        impl #name {
            /// Логує повідомлення з цим рівнем
            pub fn log(&self, msg: impl Into<String>) {
                if !::sample_logger::is_my_level(#my_level) {
                    return;
                }

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

    TokenStream::from(expanded)
}
