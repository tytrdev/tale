use std::env;

mod repl;
mod runtime;
pub mod tale;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run("repl"),
        2 => run(&args[1][..]),
        _ => println!("Confusing number of params..."),
    };
}

fn run(query: &str) {
    match query {
        "repl" => repl::repl(),
        filename => println!("Doing file: {}", filename),
    }
}
