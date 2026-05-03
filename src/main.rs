use clap::Parser;
use petty_shogi::{Engine, command::Command, response::Response};

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub run: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let mut engine = Engine::default();
    engine.set_recv(|response| match response {
        Response::Error(error) => eprintln!("[ERROR] {error}"),
        Response::Misc(message) => eprintln!("{message}"),
        _ => println!("{response}"),
    });
    if let Some(run) = &cli.run {
        for command in run.split("\\n") {
            process_command(&mut engine, command);
            engine.wait();
        }
    }
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        process_command(&mut engine, &line);
    }
}

fn process_command(engine: &mut Engine, command: &str) {
    if command.trim().is_empty() {
        return;
    }
    let Some(command) = Command::from_usi(command.trim()) else {
        eprintln!("Invalid Command {command:?}");
        return;
    };
    if matches!(command, Command::Quit) {
        eprintln!("Goodbye!");
        std::process::exit(0);
    }
    engine.process_command(command);
}
