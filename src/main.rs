use petty_shogi::{Engine, command::Command};

fn main() {
    let mut engine = Engine::init(|response| println!("{response}"));
    let mut line = String::new();
    loop {
        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        if line.ends_with('\n') {
            line.pop();
        }
        let Ok(command) = line.parse() else {
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
