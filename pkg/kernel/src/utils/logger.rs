use log::{set_max_level, Metadata, Record, LevelFilter, Level};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    // FIXME: Configure the logger
    set_max_level(LevelFilter::Trace);

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
        match record.level() {
            Level::Info => println!("[ INFO]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
            Level::Warn => println!("[ WARN]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
            Level::Debug => println!("[DEBUG]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
            Level::Error => println!("[ERROR]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args()),
            Level::Trace => println!("[TRACE]: {}@{}: {}", record.file_static().unwrap(), record.line().unwrap(), record.args())
        }
    }

    fn flush(&self) {}
}
