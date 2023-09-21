#[derive(clap::Parser)]
struct App {
    #[comman(subcommand)]
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

        #[clap(short, long, multiple = true)]
        input: Vec<Option<String>>,
    },
}

fn load_template(url: &str) -> &'static str {
    include_str!("templates/rs_bin.tp")
}

fn main() {
    let app = App::parse();

    match app.cmd {
        Cmd::Init => {}
        Cmd::Clone { url, name, input } => {
            let tp = load_template(&url);
            let mut analyzer = Analyzer::new(tp.stmts);
        }
    }
}
