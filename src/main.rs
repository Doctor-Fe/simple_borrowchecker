use std::io::{BufWriter, Write};

use crate::parser::ExprParser;

mod errors;
mod parser;

fn main() {
    let mut writer = BufWriter::new(std::io::stdout().lock());
    let mut parser = ExprParser::new();
    loop {
        let mut s: String = String::new();
        _ = write!(writer, "> ");
        _ = writer.flush();
        _ = std::io::stdin().read_line(&mut s);
        match parser.parse(&s) {
            Ok(a) => _ = writeln!(writer, "Succeed: {:?}", a),
            Err(a) => _ = writeln!(writer, "Error: {}", a),
        }
    }
}
