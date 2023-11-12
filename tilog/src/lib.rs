#[cfg(feature = "mt")]
use std::sync::Mutex;
use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(feature = "mt")]
static mut CONF: Mutex<Config> = Mutex::new(default_config());
#[cfg(not(feature = "mt"))]
static mut CONF: Config = default_config();

// #[cfg(feature = "mt")]
// static mut LEVEL: Mutex<Level> = Mutex::new(Level::Info);
// #[cfg(not(feature = "mt"))]
// static mut LEVEL: Level = Level::Info;

// #[cfg(feature = "mt")]
// static mut COLOR_CONFIG: Mutex<ColorConfig> = Mutex::new(default_color_config());
// #[cfg(not(feature = "mt"))]
// static mut COLOR_CONFIG: ColorConfig = default_color_config();

// #[cfg(feature = "mt")]
// static mut STYLE_CONFIG: Mutex<StyleConfig> = Mutex::new(default_style_config());
// #[cfg(not(feature = "mt"))]
// static mut STYLE_CONFIG: StyleConfig = default_style_config();

pub fn init_logger(conf: Config) {
    INIT.call_once(|| {
        dotenv::dotenv().ok();
        let level = match std::env::var("LOG_LEVEL")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("debug") => Level::Debug,
            Ok("info") => Level::Info,
            Ok("warn") => Level::Warn,
            Ok("error") => Level::Error,
            _ => Level::default(),
        };

        unsafe {
            #[cfg(feature = "mt")]
            {
                *CONF.lock().unwrap() = conf;
                CONF.lock().unwrap().level = level;
            }
            #[cfg(not(feature = "mt"))]
            {
                CONF = conf;
                CONF.level = level;
            }
        }
        info!("Logging initialized")
    });
}

pub fn set_level(level: Level) {
    unsafe {
        #[cfg(feature = "mt")]
        {
            CONF.lock().unwrap().level = level;
        }
        #[cfg(not(feature = "mt"))]
        {
            CONF.level = level;
        }
    }
}

pub fn get_level() -> Level {
    unsafe {
        #[cfg(feature = "mt")]
        {
            CONF.lock().unwrap().level
        }
        #[cfg(not(feature = "mt"))]
        {
            CONF.level
        }
    }
}

pub struct Config {
    pub color: ColorConfig,
    pub style: StyleConfig,
    pub level: Level,
    pub emoji: bool,
}

impl Config {
    pub fn with_color(mut self, color: ColorConfig) -> Self {
        self.color = color;
        self
    }

    pub fn with_style(mut self, style: StyleConfig) -> Self {
        self.style = style;
        self
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn with_emoji(mut self, emoji: bool) -> Self {
        self.emoji = emoji;
        self
    }
}

const fn default_config() -> Config {
    Config {
        color: default_color_config(),
        style: default_style_config(),
        level: Level::Info,
        emoji: false,
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color: ColorConfig::default(),
            style: StyleConfig::default(),
            level: Level::default(),
            emoji: false,
        }
    }
}

pub struct ColorConfig {
    debug: Color,
    info: Color,
    warn: Color,
    error: Color,
    success: Color,
}

impl Default for ColorConfig {
    fn default() -> Self {
        default_color_config()
    }
}

const fn default_color_config() -> ColorConfig {
    ColorConfig {
        debug: Color::Cyan,
        info: Color::White,
        warn: Color::Yellow,
        error: Color::Red,
        success: Color::Green,
    }
}

pub struct StyleConfig {
    debug: Style,
    info: Style,
    warn: Style,
    error: Style,
    success: Style,
}

impl Default for StyleConfig {
    fn default() -> Self {
        default_style_config()
    }
}

const fn default_style_config() -> StyleConfig {
    StyleConfig {
        info: Style::None,
        debug: Style::None,
        warn: Style::None,
        error: Style::None,
        success: Style::None,
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    #[default]
    Info,
    Debug,
    Warn,
    Error,
    Success,
}

impl Level {
    pub fn from_str(s: &str) -> Level {
        match s.to_uppercase().as_str() {
            "INFO" => Level::Info,
            "DEBUG" => Level::Debug,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            "SUCCESS" => Level::Success,
            _ => Level::Info,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Level::Info => "💡",
            Level::Debug => "❔",
            Level::Warn => "❗",
            Level::Error => "❌",
            Level::Success => "✅",
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Success => "good",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    Lexing,
    Parsing,
    Running,
    None,
}

impl Stage {
    pub fn emoji(&self) -> &'static str {
        match self {
            Stage::Lexing => "💭",
            Stage::Parsing => "🔎",
            Stage::Running => "🚀",
            Stage::None => "",
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Stage::Lexing => "lexing",
            Stage::Parsing => "parsing",
            Stage::Running => "running",
            Stage::None => "",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Orange,
    Blue,
    Magenta,
    Cyan,
    White,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Style {
    Bold,
    Italic,
    Underline,
    None,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Reset {
    All,
    Color,
    Style,
}

const fn color_code(color: Color) -> &'static str {
    match color {
        Color::Black => "\x1b[30m",
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Orange => "\x1b[38;5;208m",
        Color::Blue => "\x1b[34m",
        Color::Magenta => "\x1b[35m",
        Color::Cyan => "\x1b[36m",
        Color::White => "\x1b[37m",
    }
}

const fn style_code(style: Style) -> &'static str {
    match style {
        Style::Bold => "\x1b[1m",
        Style::Italic => "\x1b[3m",
        Style::Underline => "\x1b[4m",
        Style::None => "",
    }
}

const fn reset_code(reset: Reset) -> &'static str {
    match reset {
        Reset::All => "\x1b[0m",
        Reset::Color => "\x1b[39m",
        Reset::Style => "\x1b[22m", // Resets bold and italic
    }
}

pub fn dump<'a>(level: Level, stage: Stage, msg: std::fmt::Arguments) {
    if get_level() <= level {
        let conf = unsafe {
            #[cfg(feature = "mt")]
            {
                CONF.lock().unwrap()
            }
            #[cfg(not(feature = "mt"))]
            {
                &CONF
            }
        };

        let color = match level {
            Level::Debug => color_code(conf.color.debug),
            Level::Info => color_code(conf.color.info),
            Level::Warn => color_code(conf.color.warn),
            Level::Error => color_code(conf.color.error),
            Level::Success => color_code(conf.color.success),
        };

        let style = match level {
            Level::Debug => style_code(conf.style.debug),
            Level::Info => style_code(conf.style.info),
            Level::Warn => style_code(conf.style.warn),
            Level::Error => style_code(conf.style.error),
            Level::Success => style_code(conf.style.success),
        };

        if conf.emoji {
            println!(
                "{}{}{} {}> {}{}",
                style,
                color,
                format!("[{}]", level.emoji()),
                if stage != Stage::None {
                    format!("<{}", stage.emoji())
                } else {
                    "".to_string()
                },
                msg,
                reset_code(Reset::All)
            );
        } else {
            println!(
                "{}{}{: <5} {}> {}{}",
                style,
                color,
                format!("{}", level),
                if stage != Stage::None {
                    format!("<{}", stage.to_string())
                } else {
                    "".to_string()
                },
                msg,
                reset_code(Reset::All)
            );
        }
    }
}

#[macro_export]
macro_rules! debug {
    (lex, $($arg:tt)*) => {
        $crate::dump($crate::Level::Debug, $crate::Stage::Lexing, format_args!($($arg)*))
    };
    (parse, $($arg:tt)*) => {
        $crate::dump($crate::Level::Debug, $crate::Stage::Parsing, format_args!($($arg)*))
    };
    (run, $($arg:tt)*) => {
        $crate::dump($crate::Level::Debug, $crate::Stage::Running, format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Debug, $crate::Stage::None, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    (lex, $($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Lexing, format_args!($($arg)*))
    };
    (parse, $($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Parsing, format_args!($($arg)*))
    };
    (run, $($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Running, format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::None, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    (lex, $($arg:tt)*) => {
        $crate::dump($crate::Level::Warn, $crate::Stage::Lexing, format_args!($($arg)*))
    };
    (parse, $($arg:tt)*) => {
        $crate::dump($crate::Level::Warn, $crate::Stage::Parsing, format_args!($($arg)*))
    };
    (run, $($arg:tt)*) => {
        $crate::dump($crate::Level::Warn, $crate::Stage::Running, format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Warn, $crate::Stage::None, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    (lex; $($arg:tt)*) => {
        $crate::dump($crate::Level::Error, $crate::Stage::Lexing, format_args!($($arg)*))
    };
    (parse, $($arg:tt)*) => {
        $crate::dump($crate::Level::Error, $crate::Stage::Parsing, format_args!($($arg)*))
    };
    (run, $($arg:tt)*) => {
        $crate::dump($crate::Level::Error, $crate::Stage::Running, format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Error, $crate::Stage::None, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! success {
    (lex, $($arg:tt)*) => {
        $crate::dump($crate::Level::Success, $crate::Stage::Lexing, format_args!($($arg)*))
    };
    (parse, $($arg:tt)*) => {
        $crate::dump($crate::Level::Success, $crate::Stage::Parsing, format_args!($($arg)*))
    };
    (run, $($arg:tt)*) => {
        $crate::dump($crate::Level::Success, $crate::Stage::Running, format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Success, $crate::Stage::None, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! lex {
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Lexing, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! parse {
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Parsing, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! run {
    ($($arg:tt)*) => {
        $crate::dump($crate::Level::Info, $crate::Stage::Running, format_args!($($arg)*))
    };
}
