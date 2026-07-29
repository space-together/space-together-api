use env_logger::Env;
use std::env;

/// Initialize logging for the application.
///
/// Usage:
/// ```rust
/// config::logger::init();
/// ```
pub fn init() {
    // Set default log level if RUST_LOG is not already set
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "debug,actix_web=debug",
        );
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let ts = buf.timestamp_seconds();

            // Add colors depending on level
            let level = match record.level() {
                log::Level::Error => "❌ ERROR",
                log::Level::Warn => "⚠️  WARN ",
                log::Level::Info => "✅ INFO ",
                log::Level::Debug => "🔍 DEBUG",
                log::Level::Trace => "📍 TRACE",
            };

            writeln!(buf, "[{}] [{}] {}", ts, level, record.args())
        })
        .init();

    log::info!("✅ Logger initialized. Logs are now active.");
}
