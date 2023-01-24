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
                match stream.read_to_string(&mut s) {
                    Ok(_) => {
                        println!("{}", s);
                        match parser.parse(&s) {
                            Ok(a) => _ = writeln!(writer, "Succeed: {:?}", a),
                            Err(a) => _ = writeln!(writer, "Error: {}", a),
                        }
                        parser.clear_all();
                    },
                    Err(a) => _ = writeln!(writer, "Error: {}", a),
                }
            },
            Err(e) => _ = writeln!(writer, "Error: {}", e),
        }
    }
}
