use clap::Parser;
use tipis::actions::local::Local;

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
    Init {
        #[clap(value_parser)]
        name: Option<String>,

        #[clap(short, long)]
        publish: bool,
    },
    #[clap(name = "repl", alias = "r", about = "Start the repl")]
    Repl,
    #[clap(name = "make", alias = "m", about = "Clone a template")]
    Make {
        #[clap(value_parser)]
        name: String,

        #[clap(short, long, num_args(0..))]
        args: Vec<String>,
    },
    #[clap(name = "pub", alias = "p", about = "Publish a template")]
    Pub {
        #[clap(value_parser)]
        path: String,

        #[clap(short, long)]
        name: Option<String>,

        #[clap(short, long)]
        local: bool,

        #[clap(short, long)]
        force: bool,
    },
    Check {
        #[clap(value_parser)]
        path: String,
    },
    Drop {
        #[clap(value_parser)]
        name: String,
    },
}

fn main() -> std::io::Result<()> {
    let app = App::parse();

    match app.cmd {
        Cmd::Init { name, publish } => {
            tipis::init(name, publish, true, false);
        }
        Cmd::Repl => {
            tipis::repl();
        }
        Cmd::Make { name, args } => {
            let program_args = tipis::args(args);
            if let None = program_args {
                println!("Error: Invalid arguments");
                return Ok(());
            }
            tipis::make(name, program_args.unwrap());
        }
        Cmd::Pub {
            name, path, force, ..
        } => {
            let local = true;
            if !tipis::return_check(path.clone()) {
                println!("Error: Invalid template");
                return Ok(());
            }
            tipis::publish(name, path, local, force);
        }
        Cmd::Check { path } => {
            tipis::check(path);
        }
        Cmd::Drop { name } => {
            // tipis::drop(name);
            let local = Local::new();
            if let Err(err) = local {
                println!("Sqlite Error: {}", err);
                return Ok(());
            }
            let local = local.unwrap();
            let res = local.del(&name);
            if let Err(err) = res {
                println!("Sqlite Error: {}", err);
                return Ok(());
            }
        }
    };

    Ok(())
}
