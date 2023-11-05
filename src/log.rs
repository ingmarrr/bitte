pub struct Logger {
    pub level: Level,
}

impl Logger {
    pub fn set_level(&mut self, level: Level) {
        self.level = level;
    }

    pub fn get_level(&self) -> &Level {
        &self.level
    }

    pub fn println(&self, level: Level, s: &str) {
        println!("[{}] :: {}", level, s);
    }

    pub fn print_args(&self, level: Level, args: std::fmt::Arguments) {
        println!("[{}] :: {}", level, args);
    }
}

#[derive(PartialEq, Eq)]
pub enum Level {
    INFO,
    WARN,
    ERROR,
    TEST,
    DEBUG,
    LEX,
    PARSE,
    None,
}

impl Level {
    pub fn from_str(s: &str) -> Level {
        match s.to_uppercase().as_str() {
            "INFO" => Level::INFO,
            "WARN" => Level::WARN,
            "ERROR" => Level::ERROR,
            "TEST" => Level::TEST,
            "DEBUG" => Level::DEBUG,
            "LEX" => Level::LEX,
            "PARSE" => Level::PARSE,
            _ => Level::None,
        }
    }
}

impl std::fmt::Display for crate::log::Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            crate::log::Level::INFO => "info",
            crate::log::Level::WARN => "warn",
            crate::log::Level::ERROR => "error",
            crate::log::Level::TEST => "test",
            crate::log::Level::DEBUG => "debug",
            crate::log::Level::LEX => "lex",
            crate::log::Level::PARSE => "parse",
            crate::log::Level::None => "none",
        };
        write!(f, "{}", s)
    }
}

// pub fn info(s: &str) {
//     #[cfg(any(log_info, everything))]
//     crate::LOGGER.println(Level::INFO, s);
// }

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)+) => {
        println!("[{}] :: {}", $level, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        #[cfg(any(log_info, everything))]
        crate::LOGGER.print_args(crate::log::Level::INFO, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        #[cfg(any(log_warn, everything))]
        crate::LOGGER.print_args(crate::log::Level::WARN, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        #[cfg(any(log_error, everything))]
        crate::LOGGER.print_args(crate::log::Level::ERROR, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        #[cfg(any(log_debug, everything))]
        crate::LOGGER.print_args(crate::log::Level::DEBUG, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! test {
    ($($arg:tt)+) => {
        #[cfg(any(log_test, everything))]
        crate::LOGGER.print_args(crate::log::Level::TEST, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! lex {
    ($($arg:tt)+) => {
        #[cfg(any(log_lex, everything))]
        crate::LOGGER.print_args(crate::log::Level::LEX, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! parse {
    ($($arg:tt)+) => {
        println!("[{}] :: {}", crate::log::Level::PARSE, format_args!($($arg)+))
    };
}
