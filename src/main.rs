use std::{io::{BufWriter, Write, Read, BufReader}, fs::File};

use crate::parser::ExprParser;

mod errors;
mod parser;

fn main() {
    init_logger();
    
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
                    _ = writeln!(writer, "{}", s);
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

fn init_logger() {
    let base_config = fern::Dispatch::new();

    let debug = fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args! {
                "[{}] {}:{} {} {}",
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message
            })
        })
        .chain(fern::log_file("debug.log").unwrap());

    let info = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .format(|out, message, record| {
            out.finish(format_args! {
                "[{}] {}:{} {} {}",
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message
            })
        })
        .chain(fern::log_file("info.log").unwrap());

    base_config
        .chain(debug)
        .chain(info)
        .apply()
        .unwrap();
}

#[macro_export]
macro_rules! error_writeln {
    ($stream: expr, $error: expr) => {
        _ = writeln!($stream, "Error: {}", $error)
    };
}