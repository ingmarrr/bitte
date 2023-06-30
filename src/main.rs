use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(
    author = "Ingmar Falk",
    version,
    about = "A cli for generating project cmds."
)]
pub struct App {
    /// The action the execute. See `Action`.
    #[command(subcommand)]
    action: Action,
}

/// ```rs
/// pub enum Action {
///    New(Template),
///    Add,
/// }
/// ```
#[derive(Subcommand)]
pub enum Action {
    /// Creates a new project based on a template
    New(Template),
    /// Creates a partial template within a folder
    ///
    /// Creates the directories and files needed in
    /// case they do not exist.
    Add,
}

#[derive(Args, Debug)]
pub struct Template {
    /// Specifies the name of the template used.
    pub name: String,
    /// The author of the template. Can be
    /// specified with both the email and username.
    #[arg(short = 'a', long)]
    pub author: Option<String>,
}

fn main() {
    let app = App::parse();
    match app.action {
        Action::New(template) => {
            println!("Creating template: {:#?}", template);
        }
        Action::Add => {
            println!("Creating partial template.");
        }
    };
}
