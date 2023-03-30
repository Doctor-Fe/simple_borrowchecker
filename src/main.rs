use std::{io::{BufWriter, Write, Read, BufReader}, fs::File};

use crate::parser::ExprParser;

pub mod parser;

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
                    error_writeln!(writer, a);
                } else {
                    match parser.parse(&s) {
                        Ok(a) => _ = writeln!(writer, "Succeed: {}", a),
                        Err(a) => error_writeln!(writer, a),
                    }
                    parser.clear_all();
                }
            },
            Err(e) => error_writeln!(writer, e),
        }
    }
}

/// ロガーのセットアップをする関数です。
fn init_logger() {
    let time = chrono::Local::now().format("%Y%m%d_%H%M%S");
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
        .chain(fern::log_file(format!("logs/debug_{time}.log")).unwrap());

    base_config
        .chain(debug)
        .apply()
        .unwrap();
}

#[macro_export]
macro_rules! error_writeln {
    ($stream: expr, $error: expr) => {
        _ = writeln!($stream, "Error: {}", $error)
    };
}