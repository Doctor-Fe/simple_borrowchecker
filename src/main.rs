use std::{io::{BufWriter, Write, Read, BufReader}, fs::File};

use crate::parser::ExprParser;

mod errors;
mod parser;

fn main() {
    let mut writer = BufWriter::new(std::io::stdout().lock());
    let mut parser = ExprParser::new();
    loop {
        let mut s: String = String::new();
        _ = write!(writer, "Enter file name > ");
        _ = writer.flush();
        _ = std::io::stdin().read_line(&mut s);
        match File::open(s.trim()) {
            Ok(a) => {
                let mut stream = BufReader::new(a);
                s.clear();
                if let Err(a) = stream.read_to_string(&mut s) {
                    _ = writeln!(writer, "Error: {}", a);
                } else {
                    println!("{}", s);
                    match parser.parse(&s) {
                        Ok(a) => _ = writeln!(writer, "Succeed: {:?}", a),
                        Err(a) => _ = writeln!(writer, "Error: {}", a),
                    }
                    parser.clear_all();
                }
            },
            Err(e) => _ = writeln!(writer, "Error: {}", e),
        }
    }
}
