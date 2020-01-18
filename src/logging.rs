use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use chrono::{Utc, Datelike, Timelike};

pub struct SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        // TODO : log to file
        if self.enabled(record.metadata()) {
            let datetime = Utc::now();
            let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", 
                datetime.year(), datetime.month(), datetime.day(), 
                datetime.hour(), datetime.minute(), datetime.second());
            println!("{} {} - {}", date, record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}