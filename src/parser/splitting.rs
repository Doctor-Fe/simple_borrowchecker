use std::{fs::File, io::{Result, Read}};

use super::ExprParser;

impl ExprParser {
    pub fn from_file(file: &str) -> Result<ExprParser> {
        match File::open(file) {
            Ok(mut e) => {
                let mut str = String::new();
                e.read_to_string(&mut str)?;
                return Ok(ExprParser::from_string(&str));
            }
            Err(_) => todo!(),
        }
    }

    pub fn from_string(cmd: &str) -> ExprParser {
        let mut parser = ExprParser::new();
        let mut tmp = cmd.chars().rev().collect::<Vec<char>>();
        let mut word: Vec<char> = Vec::new();
        let mut is_string = false;
        let mut comment_out: Option<CommentType> = None;
        while let Some(a) = tmp.pop() {
            if let Some(c) = comment_out {
                if c == CommentType::SingleLine && a == '\n' {
                    comment_out = None;
                } else if a == '*' && tmp.last() == Some(&'/') {
                    tmp.pop();
                    comment_out = None;
                }
                continue;
            }
            
            if a == '/' {
                match tmp.last() {
                    Some(&'/') => {
                        comment_out = Some(CommentType::SingleLine);
                        continue;
                    }
                    Some(&'*') => {
                        comment_out = Some(CommentType::MultiLine);
                        continue;
                    }
                    _ => {}
                }
            }
            
            if is_string {
                if a == '"' && word.last().map(|a| a != &'\\').unwrap_or(true) {
                    is_string = !is_string;
                }
                word.push(a);
            } else {
                match CharType::get_chartype(a) {
                    CharType::Normal => {
                        if word.last().map(|a| CharType::get_chartype(*a) != CharType::Normal).unwrap_or(false) {
                            parser.cmds.push(String::from_iter(&word));
                            word.clear();
                        }
                        word.push(a);
                    }
                    CharType::Punctuation => {
                        word.push(a);
                        if a == '"' {
                            is_string = !is_string;
                        } else if !word.is_empty() {
                            if Self::get_priority(&String::from_iter(&word)).is_none() {
                                word.pop();
                                if !word.is_empty() {
                                    parser.cmds.push(String::from_iter(&word));
                                }
                                word.clear();
                                word.push(a);
                            }
                        }
                    }
                    CharType::WhiteSpace => if !word.is_empty() {
                        parser.cmds.push(String::from_iter(&word));
                        word.clear();
                    }
                }
            }
        }
        if !word.is_empty() {
            parser.cmds.push(String::from_iter(word));
        }
        return parser;
    }

}

/// 文字の種類を定義します。要素に分割する時に使用します。
#[derive(Debug, PartialEq)]
enum CharType {
    /// 変数名、即値などを表します。
    Normal,
    /// 記号を表します。
    Punctuation,
    /// 空白を表します。
    WhiteSpace
}

impl CharType {
    fn get_chartype(c: char) -> CharType {
        if c.is_ascii_whitespace() {
            CharType::WhiteSpace
        } else if c != '_' && c.is_ascii_punctuation() {
            CharType::Punctuation
        } else {
            CharType::Normal
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum CommentType {
    SingleLine,
    MultiLine
}