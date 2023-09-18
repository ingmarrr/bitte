pub enum Level {
    INFO,
    WARN,
    ERROR,
    TEST,
    DEBUG,
}

impl std::fmt::Display for crate::log::Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            crate::log::Level::INFO => "INFO",
            crate::log::Level::WARN => "WARN",
            crate::log::Level::ERROR => "ERROR",
            crate::log::Level::TEST => "TEST",
            crate::log::Level::DEBUG => "DEBUG",
        };
        write!(f, "{}", s)
    }
}

#[macro_export]
macro_rules! log {
    (LEX, $level:expr, $($arg:tt)+) => {
        println!("{} :: LEXING  :: {}", $level, format_args!($($arg)+));
    };
    (PAR, $level:expr, $($arg:tt)+) => {
        println!("{} :: PARSING :: {}", $level, format_args!($($arg)+));
    };
    (LA, $level:expr, $($arg:tt)+) => {
        println!("{} :: PEEKED :: {}", $level, format_args!($($arg)+));
    };
    ($level:expr, $($arg:tt)+) => {
        println!("{} :: {}", $level, format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! info {
    (LEX, $($arg:tt)+) => {
        log!(LEX, crate::log::Level::INFO, $($arg)+);
    };
    (PAR, $($arg:tt)+) => {
        log!(PAR, crate::log::Level::INFO, $($arg)+);
    };
    (LA, $($arg:tt)+) => {
        log!(LA, crate::log::Level::INFO, $($arg)+);
    };
    ($($arg:tt)+) => {
        log!(crate::log::Level::INFO, $($arg)+);
    };
}

#[macro_export]
macro_rules! warn {
    (LEX, $($arg:tt)+) => {
        log!(LEX, crate::log::Level::WARN, $($arg)+);
    };
    (PAR, $($arg:tt)+) => {
        log!(PAR, crate::log::Level::WARN, $($arg)+);
    };
    (LA, $($arg:tt)+) => {
        log!(LA, crate::log::Level::WARN, $($arg)+);
    };
    ($($arg:tt)+) => {
        log!(crate::log::Level::WARN, $($arg)+);
    };
}

#[macro_export]
macro_rules! error {
    (LEX, $($arg:tt)+) => {
        log!(LEX, crate::log::Level::ERROR, $($arg)+);
    };
    (PAR, $($arg:tt)+) => {
        log!(PAR, crate::log::Level::ERROR, $($arg)+);
    };
    (LA, $($arg:tt)+) => {
        log!(LA, crate::log::Level::ERROR, $($arg)+);
    };
    ($($arg:tt)+) => {
        log!(crate::log::Level::ERROR, $($arg)+);
    };
}

#[macro_export]
macro_rules! debug {
    (LEX, $($arg:tt)+) => {
        log!(LEX, crate::log::Level::DEBUG, $($arg)+);
    };
    (PAR, $($arg:tt)+) => {
        log!(PAR, crate::log::Level::DEBUG, $($arg)+);
    };
    (LA, $($arg:tt)+) => {
        log!(LA, crate::log::Level::DEBUG, $($arg)+);
    };
    ($($arg:tt)+) => {
        log!(crate::log::Level::DEBUG, $($arg)+);
    };
}

#[macro_export]
macro_rules! test {
    (LEX, $($arg:tt)+) => {
        log!(LEX, crate::log::Level::TEST, $($arg)+);
    };
    (PAR, $($arg:tt)+) => {
        log!(PAR, crate::log::Level::TEST, $($arg)+);
    };
    (LA, $($arg:tt)+) => {
        log!(LA, crate::log::Level::TEST, $($arg)+);
    };
    ($($arg:tt)+) => {
        log!(crate::log::Level::TEST, $($arg)+);
    };
}
