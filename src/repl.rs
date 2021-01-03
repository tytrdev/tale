use super::runtime;
use std::io;

fn read() -> String {
    let mut expr = String::new();

    io::stdin()
        .read_line(&mut expr)
        .expect("Failed to read line");

    expr
}

pub fn repl() {
    let env = &mut runtime::default_environment();

    println!("Welcome to the TALE repl. Trying some lisp expressions.");

    loop {
        println!("\nTALE >");
        let expr = read();
        match runtime::parse_eval(expr, env) {
            Ok(res) => println!("// 🔥 => {}", res),
            Err(error) => println!("// Error: {}", error),
        }
    }
}
