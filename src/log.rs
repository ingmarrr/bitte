#[derive(PartialEq, Eq)]
pub enum Level {
    INFO,
    WARN,
    ERROR,
    TEST,
    DEBUG,
    LEX,
    PARSE,
}

impl Level {
    pub fn from_str(s: &str) -> Option<Level> {
        match s {
            "INFO" => Some(Level::INFO),
            "WARN" => Some(Level::WARN),
            "ERROR" => Some(Level::ERROR),
            "TEST" => Some(Level::TEST),
            "DEBUG" => Some(Level::DEBUG),
            "LEX" => Some(Level::LEX),
            "PARSE" => Some(Level::PARSE),
            _ => None,
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
        };
        write!(f, "{}", s)
    }
}

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
        println!("[{}] :: {}", crate::log::Level::INFO, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        #[cfg(any(log_warn, everything))]
        println!("[{}] :: {}", crate::log::Level::WARN, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        #[cfg(any(log_error, everything))]
        println!("[{}] :: {}", crate::log::Level::ERROR, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        #[cfg(any(log_debug, everything))]
        println!("[{}] :: {}", crate::log::Level::DEBUG, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! test {
    ($($arg:tt)+) => {
        #[cfg(any(log_test, everything))]
        println!("[{}] :: {}", crate::log::Level::TEST, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! lex {
    ($($arg:tt)+) => {
        #[cfg(any(log_lex, everything))]
        println!("[{}] :: {}", crate::log::Level::LEX, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! parse {
    ($($arg:tt)+) => {
        println!("[{}] :: {}", crate::log::Level::PARSE, format_args!($($arg)+))
    };
}
