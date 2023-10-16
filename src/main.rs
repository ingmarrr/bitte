use std::io::Read;

use clap::Parser;

#[derive(clap::Parser)]
struct App {
    #[command(subcommand)]
    cmd: Cmd,

    #[clap(long, short = 'v', action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[clap(long, short, global = true)]
    quiet: bool,
}

pub struct Verbosity {
    pub level: usize,
}

#[derive(clap::Subcommand)]
enum Cmd {
    #[clap(name = "init", alias = "i", about = "Initialize a new project")]
    Init,
    #[clap(name = "repl", alias = "r", about = "Start the repl")]
    Repl,
    #[clap(name = "clone", alias = "c", about = "Clone a template")]
    Clone {
        #[clap(short, long)]
        url: String,

        #[clap(short, long)]
        name: String,

        #[clap(short, long, num_args(0..))]
        input: Vec<String>,
    },
}

fn main() -> std::io::Result<()> {
    let app = App::parse();

    match app.cmd {
        Cmd::Init => {}
        Cmd::Repl => {
            tipis::repl();
        }
        Cmd::Clone { name, input, .. } => {
            println!("{:?}", input);
            // let tp = include_str!("../templates/rs_bin.ti");
            println!("{:?}", std::env::current_dir()?);
            let p = format!("templates/{}.ti", name);
            let path = std::path::Path::new(&p);
            if !path.exists() {
                println!("{}", p);
                tipis::log!(tipis::log::Level::ERROR, "Template doesnt exists");
                return Ok(());
            }

            let tp = std::fs::read_to_string(p)?;

            let lx = tipis::lexer::Lexer::new(&tp);
            let mut par = tipis::parse::Parser::new(lx);
            let decls = match par.parse() {
                Ok(decls) => decls,
                Err(e) => {
                    tipis::log!(tipis::log::Level::ERROR, "{}", e);
                    return Ok(());
                }
            };
            let mut analyzer = tipis::sem::Analyzer::new();
            let _ = analyzer.analyze_all(decls);
            let exec = tipis::exec::Exec::new(name, input, analyzer);
            println!("{:#?}", exec.args);
            // println!("{:#?}", analyzer);
        }
    };

    Ok(())
}
