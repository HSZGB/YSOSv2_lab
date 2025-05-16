use log::{set_max_level, Metadata, Record, LevelFilter, Level};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    // FIXME: Configure the logger
    // set_max_level(LevelFilter::Trace);
    // set_max_level(LevelFilter::Info);
    set_max_level(LevelFilter::Debug);

    info!("Logger Initialized.");
    // panic!("panic test");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        // println!("{}", record.args());
        // match record.level() {
        //     Level::Info => println!("[ INFO]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
        //     Level::Warn => println!("[ WARN]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
        //     Level::Debug => println!("[DEBUG]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
        //     Level::Error => println!("[ERROR]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
        //     Level::Trace => println!("[TRACE]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args())
        // }
        let (level_str, color_code) = match record.level() {
            log::Level::Info  => ("[ INFO]", "\x1b[32m"),    // Green
            log::Level::Warn  => ("[ WARN]", "\x1b[33m"),    // Yellow
            log::Level::Debug => ("[DEBUG]", "\x1b[34m"),    // Blue
            log::Level::Error => ("[ERROR]", "\x1b[31m"),    // Red
            log::Level::Trace => ("[TRACE]", "\x1b[35m"),    // Magenta
        };

        let reset = "\x1b[0m";
        let file = record.file_static().unwrap_or("unknown");
        let line = record.line().unwrap_or(0);

        println!(
            "{}{}: {}@{}: {}{}",
            color_code,
            level_str,
            file,
            line,
            record.args(),
            reset
        );
    }

    fn flush(&self) {}
}
