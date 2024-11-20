use log::{self, Level, LevelFilter, Log, Metadata, Record};

use crate::println;

#[derive(Debug, Clone, Copy)]
enum LogColor {
    Red = 31,
    Yellow = 33,
    Green = 32,
    Blue = 34,
    White = 37,
}

impl LogColor {
    fn val(&self) -> u8 {
        *self as u8
    }

    fn from_level(level: Level) -> Self {
        match level {
            Level::Error => LogColor::Red,
            Level::Warn => LogColor::Yellow,
            Level::Info => LogColor::Green,
            Level::Debug => LogColor::Blue,
            Level::Trace => LogColor::White,
        }
    }
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let color = LogColor::from_level(record.level());
        println!(
            "\x1b[{}m{}\x1b[0m - {}",
            color.val(),
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

const LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}
