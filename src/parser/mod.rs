pub mod errors;
mod op;
mod splitting;
mod variables;

use std::collections::HashMap;
use std::rc::Rc;
use std::{collections::VecDeque, error::Error};

use log::{debug, info, trace};

use errors::{BracketError, InvalidExpressionError, OperationError, VariableNotFoundError};
use ElementType::Immediate;
use ElementType::Monomial;
use ElementType::Variable;

use variables::VarType;
use crate::ret_err;

/// 式を解釈するパーサです。現時点ではインタプリタとしてのみ動作します。
#[derive(Debug)]
pub struct ExprParser {
    cmds: Vec<String>,
    variables: HashMap<String, VarType>,
}

impl ExprParser {
    pub fn new() -> ExprParser {
        ExprParser {
            cmds: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cmds.clear();
    }

    pub fn clear_all(&mut self) {
        self.variables.clear();
        self.clear();
    }

    /// 文字列を式として解釈します。
    /// * `cmd` - 式として扱う文字列
    pub fn parse(&mut self, cmd: &String) -> Result<VarType, Box<dyn Error>> {
        info!("Start parsing...");
        self.split_elements(cmd); // 要素単位に分解
        debug!("Splitted elements: {:?}", self.cmds);
        let mut bracket1: i32 = 0;
        let mut bracket2: i32 = 0;
        for a in &self.cmds {
            match a.as_str() {
                "(" => bracket1 += 1,
                ")" => bracket1 -= 1,
                "{" => bracket2 += 1,
                "}" => bracket2 -= 1,
                _ => {}
            }
        }
        match bracket1.cmp(&0) {
            std::cmp::Ordering::Less => ret_err!(BracketError::new("(")),
            std::cmp::Ordering::Greater => ret_err!(BracketError::new(")")),
            std::cmp::Ordering::Equal => match bracket2.cmp(&0) {
                std::cmp::Ordering::Less => ret_err!(BracketError::new("{")),
                std::cmp::Ordering::Greater => ret_err!(BracketError::new("}")),
                std::cmp::Ordering::Equal => {
                    let mut p = 0;
                    return self.parse_sentence(&mut p);
                },
            },
        }; // かっこが一致することを確認
    }

    /// 分割された要素を解釈する関数です。
    /// * `pointer` - 次に解釈する単語を指すポインタ
    fn parse_sentence(&mut self, pointer: &mut usize) -> Result<VarType, Box<dyn Error>> {
        info!("Start parsing as sentence from {}.", pointer);
        let mut last = VarType::Void;
        while *pointer < self.cmds.len() {
            trace!("Pointer: {} ({})", pointer, self.cmds[*pointer]);
            match self.cmds.get(*pointer).map(|a| a.as_str()) {
                Some("}" | ";") => break,
                Some("let") => {
                        if let Some(a) = self.cmds.get(*pointer + 1) {
                            if a.parse::<i32>().is_err() {
                                self.create_variable(a.clone());
                            } else {
                                ret_err!(InvalidExpressionError::from("Next of \"let\" keyword must be variable name."))
                            }
                        } else {
                            ret_err!(InvalidExpressionError::from("Next of \"let\" keyword must be variable name."))
                        }
                    },
                Some("debug") => {
                    *pointer += 1;
                    last = self.parse_expression(pointer)?;
                    println!("{:?}", last);
                }
                Some(_) => {
                    last = self.parse_expression(pointer)?;
                },
                None => unreachable!(),
            }
            match self.cmds.get(*pointer).map(|a| a.as_str()) {
                Some("}") => break,
                _ => *pointer += 1,
            }
        }
        if *pointer >= self.cmds.len() {
            *pointer = self.cmds.len() - 1;
        }
        if self.cmds.get(*pointer) == Some(&String::from(";")) {
            last = VarType::Void;
        }
        return Ok(last);
    }

    /// 式を解釈する関数です。
    /// * `pointer` - 次に解釈する単語を指すポインタ
    fn parse_expression(&mut self, pointer: &mut usize) -> Result<VarType, Box<dyn Error>> {
        info!("Start parsing as expression from {}.", pointer);
        let mut list: Vec<(String, VecDeque<ElementType>)> = Vec::new();
        let mut monomial_flag: Vec<String> = vec![];
        while *pointer < self.cmds.len() {
            trace!("Pointer: {} ({})", pointer, self.cmds[*pointer]);
            let n: ElementType = match self.cmds.get(*pointer).map(|a| a.as_str()) {
                Some(";" | ")" | "}") => break,
                Some("(") => {
                    *pointer += 1;
                    Immediate(self.parse_expression(pointer)?)
                },
                Some("{") => {
                    *pointer += 1;
                    Immediate(self.parse_sentence(pointer)?)
                }
                Some(a) => match a.parse::<i32>() {
                    Ok(b) => {
                        Immediate(VarType::Integer(b))
                    },
                    Err(_) => if self.has_variable(a) {
                        Variable(String::from(a))
                    } else if Self::is_monomial(&a) {
                        monomial_flag.push(a.to_string());
                        *pointer += 1;
                        continue;
                    } else {
                        match Self::get_priority(&a) {
                            Some(_) => ret_err!(InvalidExpressionError::new(format!("Illegal operator \"{}\".", a))),
                            None => {
                                let mut t = a.chars();
                                if t.next() == Some('"') && t.last() == Some('"') {
                                    Immediate(VarType::new_string(a))
                                } else {
                                    ret_err!(VariableNotFoundError::new(String::from(a)))
                                }
                            },
                        }
                    },
                },
                None => unreachable!(),
            };
            let n = {
                let mut tmp = n;
                while let Some(a) = monomial_flag.pop() {
                    tmp =  Monomial(a, Rc::new(tmp));
                }
                tmp
            };
            *pointer += 1;
            if let Some(mut a) = list.pop() {
                match self.cmds.get(*pointer).map(|a| a.as_str()) {
                    Some(";" | ")" | "}") => {
                        a.1.push_back(n);
                        list.push(a);
                        break;
                    }
                    Some(upcoming) => {
                        if upcoming != &a.0 && Self::get_priority(upcoming) >= Self::get_priority(&a.0) {
                            a.1.push_back(n);
                            trace!("{:?}", a);
                            let tmp = self.try_calculate_all(a)?;
                            if tmp.is_empty() {
                                ret_err!(OperationError)
                            } else {
                                list.push((self.cmds[*pointer].clone(), VecDeque::from([Immediate(tmp)])));
                            }
                        } else {
                            list.push(a);
                            list.push((self.cmds[*pointer].clone(), VecDeque::from([n])));
                        }
                    }
                    None => {
                        a.1.push_back(n);
                        list.push(a);
                    }
                }
            } else {
                trace!("Upcoming: {:?}", self.cmds.get(*pointer));
                match self.cmds.get(*pointer).map(|a| a.as_str()) {
                    Some(";" | ")" | "}") => {
                        return n.to_vartype(self);
                    }
                    Some(upcoming) => {
                        list.push((String::from(upcoming), VecDeque::from([n])));
                    },
                    None => return n.to_vartype(self),
                }
            }
            *pointer += 1;
        }
        match list.pop() {
            Some(b) => {
                let mut num = self.try_calculate_all(b)?;
                while let Some(mut c) = list.pop() {
                    if num.is_empty() {
                        ret_err!(OperationError)
                    } else {
                        c.1.push_back(Immediate(num));
                        num = self.try_calculate_all(c)?;
                    }
                }
                return Ok(num);
            },
            None => {
                return Ok(VarType::Void);
            },
        }
    }
}

/// 式の要素の種類を定義します。
#[derive(Debug, Clone)]
pub enum ElementType {
    /// 変数であることを表します。
    Variable(String),
    /// 即値であることを表します。
    Immediate(VarType),
    /// 単項式であることを表します。
    Monomial(String, Rc<ElementType>)
}

impl ElementType {
    /// 数値へ変換します。
    /// * `expr` - 関数を呼び出した `ExprParser`
    fn to_vartype(&self, expr: &ExprParser) -> Result<VarType, Box<dyn Error>> {
        match self {
            ElementType::Variable(s) => Ok(expr.get_variable(s).map(|a| a.clone()).unwrap()),
            ElementType::Immediate(i) => Ok(i.clone()),
            ElementType::Monomial(s, e) => {
                match (e.to_vartype(expr)?, s.as_str()) {
                    (VarType::Uninitialized | VarType::Void, _) => ret_err!(OperationError),
                    (a, "&") => Ok(VarType::Pointer(Rc::new(a))),
                    (a, "&&") => Ok(VarType::Pointer(Rc::new(VarType::Pointer(Rc::new(a))))),
                    (VarType::Integer(i), "+") => Ok(VarType::Integer(i)),
                    (VarType::Integer(i), "-") => Ok(VarType::Integer(-i)),
                    (VarType::Integer(i), "~") => Ok(VarType::Integer(!i)),
                    (VarType::Integer(i), "!") => Ok(VarType::Integer(if i == 0 {1} else {0})),
                    (VarType::Integer(_), a) => ret_err!(InvalidExpressionError::new(format!("Monomial \"{}\" is not for integer.", a))),
                    (VarType::String(_), _) => ret_err!(InvalidExpressionError::from("There are no monomial for string.")),
                    (VarType::Pointer(p), "*") => Ok((*p).clone()),
                    (VarType::Pointer(_), _) => ret_err!(InvalidExpressionError::new(format!("Monomial \"{}\" is not for pointer.", s))),
                }
            }
        }
    }

    /// 二項演算子の演算を行います。
    /// - `expr` - 処理を呼び出すパーサのインスタンス
    /// - `right` - 右辺に来る `ElementType` 構造体
    /// - `op` - 具体的な処理内容を記述するクロージャ
    fn operation<F>(self, expr: &mut ExprParser, right: ElementType, op: F) -> Result<VarType, Box<dyn Error>>
    where
        F: Fn(VarType, VarType) -> Result<VarType, Box<dyn Error>>,
    {
        return op(self.to_vartype(expr)?, right.to_vartype(expr)?);
    }

    /// 二項演算子の演算を行います。
    /// - `expr` - 処理を呼び出すパーサのインスタンス
    /// - `right` - 右辺に来る `ElementType` 構造体
    /// - `op` - 具体的な処理内容を記述するクロージャ
    fn operation_mut<F>(self, expr: &mut ExprParser, right: ElementType, op: F) -> Result<VarType, Box<dyn Error>>
    where
        F: Fn(&mut VarType, VarType) -> Result<VarType, Box<dyn Error>>,
    {
        if let Variable(v) = self {
            let c = right.to_vartype(expr);
            match expr.get_variable_mut(&v) {
                Some(a) => return op(a, c?),
                None => ret_err!(VariableNotFoundError::new(v.clone())),
            }
        }
        ret_err!(InvalidExpressionError::from("The left-hand must be variable."));
    }

}
