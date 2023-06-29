
use clap::{Parser, Subcommand, Args};


#[derive(Parser, Debug, Default)]
#[clap(author="Ingmar Falk", version, about="A cli for generating project templates.")]
pub struct App {
    template: Option<Template>,
}


#[derive(Args, Clone, Debug)]
pub struct Template {
    pub name: String,
}



let app = App::parse();
fn main() {

    match app.template {
        Some(template) => {
            
        }
        None => println!("No template provided.")
    }
}
