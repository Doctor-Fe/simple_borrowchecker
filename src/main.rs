use std::io::BufWriter;
use std::io::Write;

use evaluater::Evaluater;

mod errors;
mod evaluater;
mod parser;

fn main() {
    let mut test = BufWriter::new(std::io::stdout().lock());
    let mut eval: Evaluater = Evaluater::new();

    loop {
        let mut s: String = String::new();
        _ = test.write(b"> ");
        _ = test.flush();
        _ = std::io::stdin().read_line(&mut s);
        eval.split_elements(s);
        match eval.pop_command() {
            Some(mut a) => {
                match parser::ExprParser::parse(&mut eval, &mut a) {
                    Ok(a) => println!("{:?}", a),
                    Err(a) => {
                        println!("error occured.");
                        println!("{}", a);
                    },
                }
            },
            None => {
            },
        }
    }
}
