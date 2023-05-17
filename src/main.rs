use std::{env, io::{BufWriter, Write}};

use crate::parser::ExprParser;

pub mod parser;

fn main() {
    init_logger();

    let mut writer = BufWriter::new(std::io::stdout().lock());
    match env::args().skip(1).next() {
        Some(path) => match ExprParser::from_file(&path) {
            Ok(mut a) => match a.parse() {
                Ok(a) => _ = writeln!(writer, "Succed: {}", a),
                Err(p) => error_writeln!(writer, p),
            },
            Err(e) => error_writeln!(writer, e),
        },
        None => _ = writeln!(writer, "Enter the file name."),
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

    base_config.chain(debug).apply().unwrap();
}

#[macro_export]
macro_rules! error_writeln {
    ($stream: expr, $error: expr) => {
        _ = writeln!($stream, "Error: {}", $error)
    };
}