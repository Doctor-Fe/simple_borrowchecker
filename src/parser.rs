use std::collections::HashMap;
use std::{collections::VecDeque, error::Error};

use crate::errors::{BracketError, VariableNotFoundError, InvalidExpressionError};

use crate::ret_err;

/// 式を解釈するパーサです。現時点ではインタプリタとしてのみ動作します。
pub struct ExprParser {
    cmds: VecDeque<String>,
    variables: HashMap<String, i32>,
}

impl ExprParser {
    pub fn new() -> ExprParser {
        ExprParser {
            cmds: VecDeque::new(),
            variables: HashMap::new(),
        }
    }

    /// 演算子の優先順位を返します。
    /// * `op` - 優先順位を取得する演算子
    fn get_priority(op: &str) -> Option<usize> {
        let priorities = [
            vec!["(", ")", "{", "}"],
            vec!["++", "--"],
            vec!["!", "~", "&", "*"],
            vec!["*", "/", "%"],
            vec!["+", "-"],
            vec!["==", "!="],
            vec!["&", "|", "^"],
            vec!["&&, ||"],
            vec!["=", "+=", "-=", "*=", "/=", "%=", "|=", "&=", "^="]
        ];
        for a in 0..priorities.len() {
            if priorities[a].contains(&op) {
                return Some(a);
            }
        }
        return None;
    }

    /// 文字列を式として解釈します。
    /// * `cmd` - 式として扱う文字列
    pub fn parse(&mut self, cmd: String) -> Result<i32, Box<dyn Error>> {
        self.split_elements(cmd); // 要素単位に分解
        let a = self.cmds.iter().filter(|a| **a == "(").count().cmp(&self.cmds.iter().filter(|a| **a == ")").count());
        match a {
            std::cmp::Ordering::Less => ret_err!(BracketError::new("(")),
            std::cmp::Ordering::Equal => {}, // NOOP
            std::cmp::Ordering::Greater => ret_err!(BracketError::new(")")),
        } // かっこが一致することを確認
        print!("{}", self.cmds.iter().fold("".to_string(), |a, b| format!("{}{}\n", a, b))); // デバッグ用
        self.parse_middle_phase('\0') // 実行
    }

    fn split_elements(&mut self, cmd: String) {
        let mut word: Vec<char> = Vec::new();
        let mut is_string = false;
        self.cmds.clear();
        for a in cmd.chars() {
            if is_string {
                if a == '"' && if let Some(a) = word.last() {a != &'\\'} else {true} {
                    is_string = !is_string;
                }
                word.push(a);
            } else {
                match CharType::get_chartype(a) {
                    CharType::Normal => {
                        if !word.is_empty() && CharType::get_chartype(*word.last().unwrap()) != CharType::Normal {
                            self.cmds.push_back(word.iter().collect::<String>());
                            word.clear();
                        }
                        word.push(a);
                    },
                    CharType::Punctuation => {
                        if a == '"' {
                            is_string = !is_string;
                        } else if !word.is_empty() && (word.len() > 1 || !word.last().unwrap().is_ascii_punctuation()) {
                            self.cmds.push_back(word.iter().collect::<String>());
                            word.clear();
                        }
                        word.push(a);
                    },
                    CharType::WhiteSpace => {
                        if !word.is_empty() {
                            self.cmds.push_back(word.iter().collect::<String>());
                            word.clear();
                        }
                    },
                }
            }
        }
        if !word.is_empty() {
            self.cmds.push_back(word.iter().collect::<String>());
        }
    }

    fn parse_middle_phase(&mut self, bracket_flag: char) -> Result<i32, Box<dyn Error>> {
        let mut list: Vec<(String, Vec<i32>)> = Vec::new();
        while !self.cmds.is_empty() {
            let n = match self.cmds.pop_front() {
                Some(a) => match a.as_str() {
                    "let" => {
                        match self.cmds.front() {
                            Some(a) => {
                                self.create_variable(a.clone());
                                continue;
                            },
                            None => {
                                ret_err!(InvalidExpressionError::new("Next of \"let\" keyword must be variable name."))
                            },
                        }
                    }
                    "(" => match self.parse_middle_phase('(') {
                        Ok(a) => a,
                        Err(a) => return Err(a),
                    },
                    ")" => {
                        if bracket_flag == '(' {
                            break;
                        } else {
                            ret_err!(BracketError::new("("));
                        }
                    }
                    _ => match a.parse::<i32>() {
                        Ok(b) => b,
                        Err(_) => match self.get_variable(&a.to_string()) {
                            Some(a) => *a,
                            None => match Self::get_priority(&a) {
                                Some(_) => ret_err!(InvalidExpressionError::new("Illegal operator.")),
                                None => ret_err!(VariableNotFoundError::new(a)),
                            },
                        },
                    },
                },
                None => unreachable!(),
            };
            if let Some(a) = list.pop() {
                match self.cmds.front() {
                    Some(b) => {
                        if Self::get_priority(b) >= Self::get_priority(&a.0) {
                            let t = [a.1, vec![n]].concat().into_iter().reduce(|b, c| Self::calculate(&a.0, b, c).unwrap()).unwrap();
                            list.push((self.cmds.pop_front().unwrap(), vec![t]));
                        } else {
                            list.push(a);
                            list.push((self.cmds.pop_front().unwrap(), vec![n]));
                        }
                    }
                    None => {
                        list.push((a.0.clone(), [a.1.clone(), vec![n]].concat()));
                    }
                }
            } else {
                match self.cmds.pop_front() {
                    Some(a) => list.push((a, vec![n])),
                    None => return Ok(n),
                }
            }
        }
        let (mut a, mut b) = list.pop().unwrap();
        let mut num = b.into_iter().reduce(|c, d| Self::calculate(&a, c, d).unwrap()).unwrap();
        while !list.is_empty() {
            let c = list.pop().unwrap();
            a = c.0;
            b = [c.1, vec![num]].concat();
            num = b.into_iter().reduce(|c, d| Self::calculate(&a, c, d).unwrap()).unwrap();
        }
        return Ok(num);
    }

    fn calculate(op: &str, left: i32, right: i32) -> Option<i32> {
        match op {
            "+" => Some(left + right),
            "-" => Some(left - right),
            "*" => Some(left * right),
            "/" => Some(left / right),
            "%" => Some(left % right),
            "|" => Some(left | right),
            "&" => Some(left & right),
            "^" => Some(left ^ right),
            "=" => {
                //
                None
            }
            _ => unreachable!(),
        }
    }

    fn get_variable(&self, name: &String) -> Option<&i32> {
        self.variables.get(name)
    }

    fn create_variable(&mut self, name: String){//, num: i32) {
        self.variables.insert(name, 0);
    }
}

#[macro_export]
macro_rules! ret_err {
    ($x: expr) => {
        return Err(Box::new($x))
    };
}

#[derive(Debug, PartialEq)]
pub enum CharType {
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
