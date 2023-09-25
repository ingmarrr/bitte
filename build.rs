use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    if let Ok(logging_level) = env::var("LOG") {
        println!("cargo:rerun-if-env-changed=LOG");
        match logging_level.as_str() {
            "parser" => println!("cargo:rustc-cfg=log_parser"),
            "lexer" => println!("cargo:rustc-cfg=log_lexer"),
            "info" => println!("cargo:rustc-cfg=log_info"),
            "warn" => println!("cargo:rustc-cfg=log_warn"),
            "error" => println!("cargo:rustc-cfg=log_error"),
            "debug" => println!("cargo:rustc-cfg=log_debug"),
            "test" => println!("cargo:rustc-cfg=log_test"),
            _ => println!("cargo:rustc-cfg=everything"),
        }
    } else {
        println!("cargo:rustc-cfg=everything");
    }
}
