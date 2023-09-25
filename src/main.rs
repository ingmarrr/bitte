use clap::Parser;

#[derive(clap::Parser)]
struct App {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand)]
enum Cmd {
    Init,
    Clone {
        #[clap(short, long)]
        url: String,

        #[clap(short, long)]
        name: String,

        #[clap(short, long, num_args(0..))]
        input: Vec<Option<String>>,
    },
}

fn main() {
    let app = App::parse();

    match app.cmd {
        Cmd::Init => {}
        Cmd::Clone { .. } => {
            let tp = include_str!("../templates/rs_bin.ti");
            let lx = tipis::lex::Lexer::new(tp);
            let mut par = tipis::parse::Parser::new(lx);
            let decls = par.parse().unwrap();
            let mut analyzer = tipis::sem::Analyzer::new();
            let decls = analyzer.analyze(decls);
            println!("{:#?}", decls);
            println!("{:#?}", analyzer);
        }
    }
}
