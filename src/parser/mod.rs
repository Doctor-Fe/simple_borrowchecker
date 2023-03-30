pub mod errors;
mod op;
mod splitting;
mod variables;

use std::collections::HashMap;
use std::rc::Rc;
use std::{collections::VecDeque, error::Error};

use log::{debug, info, trace};

use errors::{BracketError, InvalidExpressionError, NoInputError, OperationError, VariableNotFoundError, ReferenceError};
use ElementType::Immediate;
use ElementType::Monomial;
use ElementType::Variable;

use variables::VarType;
use crate::ret_err;
use crate::vec_deque;

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
        match self.cmds.iter().filter(|a| **a == "(").count().cmp(&self.cmds.iter().filter(|a| **a == ")").count()) {
            std::cmp::Ordering::Less => ret_err!(BracketError::new("(")),
            std::cmp::Ordering::Equal => {
                let mut p = 0;
                loop {
                    match self.parse_sentence(&mut p) {
                        Ok(a) => {
                            if p >= self.cmds.len() {
                                info!("Successfully parsed (Result: {})", a);
                                return Ok(a);
                            }
                        },
                        Err(b) => return Err(b),
                    }
                    p += 1;
                }
            },
            std::cmp::Ordering::Greater => ret_err!(BracketError::new(")")),
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
                Some("}") | Some(";") => break,
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
                Some("{") => {
                    *pointer += 1;
                    last = self.parse_sentence(pointer)?;
                },
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
        let mut monomial_flag: Option<String> = None;
        while *pointer < self.cmds.len() {
            trace!("Pointer: {} ({})", pointer, self.cmds[*pointer]);
            let n: ElementType = match self.cmds.get(*pointer) {
                Some(a) => match a.as_str() {
                    ";" | ")" | "}" => {
                        break;
                    },
                    "(" => {
                        *pointer += 1;
                        Immediate(self.parse_expression(pointer)?)
                    },
                    "{" => {
                        *pointer += 1;
                        let tmp = Immediate(self.parse_sentence(pointer)?);
                        tmp
                    }
                    _ => match a.parse::<i32>() {
                        Ok(b) => {
                            if monomial_flag.is_some() {
                                Monomial(monomial_flag.clone().unwrap(), Rc::new(Immediate(VarType::Integer(b))))
                            } else {
                                Immediate(VarType::Integer(b))
                            }
                        },
                        Err(_) => match self.get_variable(&a.to_string()) {
                            Some(_) => {
                                if monomial_flag.is_some() {
                                    Monomial(monomial_flag.clone().unwrap(), Rc::new(Variable(a.clone())))
                                } else {
                                    Variable(a.clone())
                                }
                            },
                            None => if Self::is_monomial(&a) {
                                monomial_flag = Some(a.to_string());
                                *pointer += 1;
                                continue;
                            } else {
                                match Self::get_priority(&a) {
                                    Some(_) => ret_err!(InvalidExpressionError::new(format!("Illegal operator \"{}\".", a))),
                                    None => {
                                        let mut t = a.chars();
                                        if t.next() == Some('"') && t.last() == Some('"') {
                                            if monomial_flag.is_some() {
                                                Monomial(monomial_flag.clone().unwrap(), Rc::new(Immediate(VarType::new_string(&a))))
                                            } else {
                                                Immediate(VarType::new_string(&a))
                                            }
                                        } else {
                                            ret_err!(VariableNotFoundError::new(a.clone()))
                                        }
                                    },
                                }
                            }
                        },
                    },
                },
                None => unreachable!(),
            };
            *pointer += 1;
            if let Some(mut a) = list.pop() {
                match self.cmds.get(*pointer) {
                    Some(upcoming) => {
                        if upcoming == ";" || upcoming == ")" || upcoming == "}" {
                            a.1.push_back(n);
                            list.push(a);
                            break;
                        }
                        if upcoming != &a.0 && Self::get_priority(upcoming) >= Self::get_priority(&a.0) {
                            a.1.push_back(n);
                            trace!("{:?}", a);
                            let tmp = self.try_calculate_all(a)?;
                            if tmp.is_empty() {
                                ret_err!(OperationError)
                            } else {
                                list.push((self.cmds.get(*pointer).unwrap().clone(), vec_deque![Immediate(tmp)]));
                            }
                        } else {
                            list.push(a);
                            list.push((self.cmds.get(*pointer).unwrap().clone(), vec_deque!(n)));
                        }
                    }
                    None => {
                        a.1.push_back(n);
                        list.push(a);
                    }
                }
            } else {
                trace!("Upcoming: {:?}", self.cmds.get(*pointer));
                match self.cmds.get(*pointer) {
                    Some(upcoming) => {
                        if upcoming == ";" || upcoming == ")" || upcoming == "}" {
                            return n.to_vartype(self);
                        }
                        list.push((upcoming.clone(), vec_deque!(n)));
                    },
                    None => return n.to_vartype(self),
                }
            }
            *pointer += 1;
        }
        match list.pop() {
            Some(b) => {
                trace!("{:?}", b);
                let mut num = self.try_calculate_all(b);
                loop {
                    if list.is_empty() {
                        return num;
                    }
                    match num {
                        Ok(a) => {
                            if a.is_empty() {
                                ret_err!(OperationError)
                            } else {
                                let mut c = list.pop().unwrap();
                                c.1.push_back(Immediate(a));
                                trace!("{:?}", c);
                                num = self.try_calculate_all(c);
                            }
                        },
                        Err(e) => return Err(e),
                    }
                }
            },
            None => {
                ret_err!(NoInputError);
            },
        }
    }
}


/// 1つの要素が入った `VecDeque` を生成するマクロです。
#[macro_export]
macro_rules! vec_deque {
    ($($x: expr),*) => {
        {
            let mut tmp = VecDeque::new();
            $(tmp.push_back($x);)*
            tmp
        }
    };
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
    fn get(&self) -> &Self {self}
    /// 数値へ変換します。
    /// * `expr` - 関数を呼び出した `ExprParser`
    fn to_vartype(&self, expr: &ExprParser) -> Result<VarType, Box<dyn Error>> {
        match self {
            ElementType::Variable(s) => match expr.get_variable(s) {
                Some(a) => Ok(a.clone()),
                None => Ok(VarType::Void),
            },
            ElementType::Immediate(i) => Ok(i.clone()),
            ElementType::Monomial(s, e) => {
                if s == "&" {
                    match e.get() {
                        Variable(v) => Ok(VarType::Pointer(v.clone())),
                        _ => ret_err!(ReferenceError::invalid_dereference())
                    }
                } else {
                    match e.to_vartype(expr)? {
                        VarType::Uninitialized | VarType::Void => ret_err!(OperationError),
                        VarType::Integer(i) => {
                            match s.as_str() {
                                    "+" => Ok(VarType::Integer(i)),
                                    "-" => Ok(VarType::Integer(-i)),
                                    "~" => Ok(VarType::Integer(!i)),
                                    "!" => Ok(VarType::Integer(if i == 0 {1} else {0})),
                                    _ => unreachable!()
                            }
                        },
                        VarType::String(_) => todo!(),
                        VarType::Pointer(p) => {
                            if s == "*" {
                                match expr.get_variable(&p) {
                                    Some(i) => Ok(i.clone()),
                                    None => Ok(VarType::Void),
                                }
                            } else {
                                unreachable!()
                            }
                        },
                    }
                }
            }
        }
    }

    /// 数値を返す演算を実行します。
    /// - `expr` - 処理を呼び出すパーサのインスタンス
    /// - `right` - 右辺に来る `ElementType` 構造体
    /// - `op` - 具体的な処理内容を記述するクロージャ
    fn op_num<F>(self, expr: &ExprParser, other: ElementType, op: F) -> Result<VarType, Box<dyn Error>>
    where
        F: Fn(i32, i32) -> i32,
    {
        if let VarType::Integer(left) = self.to_vartype(expr)? {
            if let VarType::Integer(right) = other.to_vartype(expr)? {
                return Ok(VarType::Integer(op(left, right)));
            }
        }
        ret_err!(InvalidExpressionError::from("Cannot operate with void"))
    }

    /// 代入処理を実行します。
    /// - `expr` - 処理を呼び出すパーサのインスタンス
    /// - `right` - 右辺に来る `ElementType` 構造体
    /// - `op` - 具体的な処理内容を記述するクロージャ
    fn op_let<F>(self, expr: &mut ExprParser, right: ElementType, op: F) -> Result<VarType, Box<dyn Error>>
    where
        F: Fn(&mut VarType, VarType),
    {
        if let Variable(v) = self {
            let c = right.to_vartype(expr);
            match expr.get_variable_mut(&v) {
                Some(a) => {
                    op(a, c?);
                    return Ok(VarType::Void);
                },
                None => ret_err!(VariableNotFoundError::new(v.clone())),
            }
        }
        ret_err!(InvalidExpressionError::from("The left-hand must be variable."));
    }

}
