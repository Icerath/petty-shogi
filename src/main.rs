use petty_shogi::{Engine, command::Command, response::Response};

fn main() {
    let mut engine = Engine::init(|response| match response {
        Response::Error(error) => eprintln!("[ERROR] {error}"),
        _ => println!("{response}"),
    });
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let Some(command) = Command::from_usi(line.trim()) else {
            eprintln!("Invalid Command");
            continue;
        };
        if matches!(command, Command::Quit) {
            eprintln!("Goodbye!");
            break;
        }
        engine.process_command(command);
    }
}
