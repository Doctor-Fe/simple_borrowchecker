use std::collections::HashMap;
use std::rc::Rc;
use std::{collections::VecDeque, error::Error};

use crate::errors::{BracketError, InvalidExpressionError, NoInputError, OperationError, VariableNotFoundError};
use crate::parser::ElementType::Immediate;
use crate::parser::ElementType::Monomial;
use crate::parser::ElementType::Variable;

use crate::ret_err;
use crate::vec_deque;

/// 式を解釈するパーサです。現時点ではインタプリタとしてのみ動作します。
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

    /// 可変な状態で変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable_mut(&mut self, name: &String) -> Option<&mut VarType> {
        self.variables.get_mut(name)
    }
    
    /// 変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable(&self, name: &String) -> Option<&VarType> {
        self.variables.get(name)
    }

    /// 変数を作成します。
    /// - `name` - 新しく作成する変数名
    pub fn create_variable(&mut self, name: String){
        self.variables.insert(name, VarType::Uninitialized);
    }

    /// 文字列を式として解釈します。
    /// * `cmd` - 式として扱う文字列
    pub fn parse(&mut self, cmd: &String) -> Result<VarType, Box<dyn Error>> {
        self.split_elements(cmd); // 要素単位に分解
        match self.cmds.iter().filter(|a| **a == "(").count().cmp(&self.cmds.iter().filter(|a| **a == ")").count()) {
            std::cmp::Ordering::Less => ret_err!(BracketError::new("(")),
            std::cmp::Ordering::Equal => {
                let mut p = 0;
                loop {
                    match self.parse_middle_phase(&mut p) {
                        Ok(a) => {
                            if p >= self.cmds.len() {
                                return Ok(a);
                            }
                        },
                        Err(b) => return Err(b),
                    }
                }
            },
            std::cmp::Ordering::Greater => ret_err!(BracketError::new(")")),
        }; // かっこが一致することを確認
    }

    /// 入力された文字列を要素毎に分割します。
    /// * `cmd` - 分割する文字列
    fn split_elements(&mut self, cmd: &String) {
        let mut tmp = cmd.chars().rev().collect::<Vec<char>>();
        let mut word: Vec<char> = Vec::new();
        let mut is_string = false;
        let mut comment_out = 0;
        while !tmp.is_empty() {
            let a = tmp.pop().unwrap();
            if comment_out != 0 {
                if comment_out == 1 && a == '\n' {
                    comment_out = 0;
                } else if comment_out == 2 && a == '*' && tmp.last() == Some(&'/') {
                    tmp.pop();
                    comment_out = 0;
                }
                continue;
            }
            
            if a == '/' {
                match tmp.last() {
                    Some(&'/') => {
                        comment_out = 1;
                        continue;
                    }
                    Some(&'*') => {
                        comment_out = 2;
                        continue;
                    },
                    _ => {}
                }
            }
            
            if is_string {
                if a == '"' && if let Some(a) = word.last() {a != &'\\'} else {true} {
                    is_string = !is_string;
                }
                word.push(a);
            } else {
                match CharType::get_chartype(a) {
                    CharType::Normal => {
                        if !word.is_empty() && CharType::get_chartype(*word.last().unwrap()) != CharType::Normal {
                            self.cmds.push(String::from_iter(&word));
                            word.clear();
                        }
                        word.push(a);
                    },
                    CharType::Punctuation => {
                        if a == '"' {
                            is_string = !is_string;
                        } else if !word.is_empty() && (Self::get_priority(String::from_iter([word.clone(), vec![a]].concat()).as_str()).is_none()) {
                            self.cmds.push(String::from_iter(&word));
                            word.clear();
                        }
                        word.push(a);
                    },
                    CharType::WhiteSpace => {
                        if !word.is_empty() {
                            self.cmds.push(String::from_iter(&word));
                            word.clear();
                        }
                    },
                }
            }
        }
        if !word.is_empty() {
            self.cmds.push(String::from_iter(word));
        }
    }

    /// 分割された要素を解釈する関数です。
    fn parse_middle_phase(&mut self, pointer: &mut usize) -> Result<VarType, Box<dyn Error>> {
        let mut list: Vec<(String, VecDeque<ElementType>)> = Vec::new();
        let mut monomial_flag: Option<String> = None;
        while *pointer < self.cmds.len() {
            let n: ElementType = match {*pointer += 1; self.cmds.get(*pointer - 1)} {
                Some(a) => match a.as_str() {
                    ";" | ")" => break,
                    "let" => {
                        if let Some(a) = self.cmds.get(*pointer) {
                            if a.parse::<i32>().is_err() {
                                self.create_variable(a.clone());
                                continue;
                            }
                        }
                        ret_err!(InvalidExpressionError::new("Next of \"let\" keyword must be variable name."))
                    },
                    "(" => Immediate(self.parse_middle_phase({*pointer += 1; pointer})?),
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
                                continue;
                            } else {
                                match Self::get_priority(&a) {
                                    Some(_) => ret_err!(InvalidExpressionError::new("Illegal operator.")),
                                    None => {
                                        let mut t = a.chars();
                                        if t.next() == Some('"') && t.last() == Some('"') {
                                            if monomial_flag.is_some() {
                                                Monomial(monomial_flag.clone().unwrap(), Rc::new(Immediate(VarType::String(a.clone()))))
                                            } else {
                                                Immediate(VarType::String(a.clone()))
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
            if let Some(mut a) = list.pop() {
                match self.cmds.get(*pointer) {
                    Some(upcoming) => {
                        if upcoming == ";" || upcoming == ")" {
                            {*pointer += 1; self.cmds.get(*pointer - 1)};
                            a.1.push_back(n);
                            list.push(a);
                            break;
                        }
                        if upcoming != &a.0 && Self::get_priority(upcoming) >= Self::get_priority(&a.0) {
                            a.1.push_back(n);
                            let tmp = self.try_calculate_all(a)?;
                            if tmp.is_empty() {
                                ret_err!(OperationError)
                            } else {
                                list.push(({*pointer += 1; self.cmds.get(*pointer - 1)}.unwrap().clone(), vec_deque![Immediate(tmp)])),
                            }
                        } else {
                            list.push(a);
                            list.push(({*pointer += 1; self.cmds.get(*pointer - 1)}.unwrap().clone(), vec_deque!(n)));
                        }
                    }
                    None => {
                        a.1.push_back(n);
                        list.push(a);
                    }
                }
            } else {
                match {*pointer += 1;self.cmds.get(*pointer - 1)} {
                    Some(a) => list.push((a.clone(), vec_deque!(n))),
                    None => return n.to_vartype(self),
                }
            }
        }
        match list.pop() {
            Some(b) => {
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

    /// 二項演算子の優先順位を返します。
    /// * `op` - 優先順位を取得する演算子
    fn get_priority(op: &str) -> Option<usize> {
        let priorities = [
            vec!["*", "/", "%"],
            vec!["+", "-"],
            vec![">>", "<<"],
            vec!["&", "|", "^"],
            vec!["==", "!=", ">", "<", ">=", "<="],
            vec!["&&", "||"],
            vec!["=", "+=", "-=", "*=", "/=", "%=", "|=", "&=", "^=", ">>=", "<<="]
        ];
        for a in 0..priorities.len() {
            if priorities[a].contains(&op) {
                return Some(a);
            }
        }
        return None;
    }

    /// 単項演算子か判定する関数です。
    /// - `op` - 判定する演算子
    fn is_monomial(op: &str) -> bool {
        let ops = ["+", "-", "&", "*", "!", "~"];
        return ops.contains(&op);
    }

    fn try_calculate_all(&mut self, mut data: (String, VecDeque<ElementType>)) -> Result<VarType, Box<dyn Error>> {
        if data.0 != "=" {
            let mut num = data.1.pop_front().unwrap();
            while !data.1.is_empty() {
                let d = data.1.pop_front().unwrap();
                num = match self.calculate_binomial(&data.0, num, d)? {
                    VarType::Integer(i) => Immediate(VarType::Integer(i)),
                    _ => return Ok(VarType::Void),
                };
            }
            return num.to_vartype(self);
        } else {
            let mut num = data.1.pop_back().unwrap();
            while !data.1.is_empty() {
                let d = data.1.pop_back().unwrap();
                num = match self.calculate_binomial(&data.0, d, num)? {
                    VarType::Integer(i) => Immediate(VarType::Integer(i)),
                    _ => return Ok(VarType::Void)
                }
            }
            return num.to_vartype(self);
        }
    }

    fn calculate_binomial(&mut self, op: &str, left: ElementType, right: ElementType) -> Result<VarType, Box<dyn Error>> {
        match op {
            "+" => left.op_num(self, right, |a, b| a + b),
            "-" => left.op_num(self, right, |a, b| a - b),
            "*" => left.op_num(self, right, |a, b| a * b),
            "/" => left.op_num(self, right, |a, b| a / b),
            "%" => left.op_num(self, right, |a, b| a % b),
            "|" => left.op_num(self, right, |a, b| a | b),
            "&" => left.op_num(self, right, |a, b| a & b),
            "^" => left.op_num(self, right, |a, b| a ^ b),
            ">>" => left.op_num(self, right, |a, b| a >> b),
            "<<" => left.op_num(self, right, |a, b| a << b),
            "==" => left.op_num(self, right, |a, b| if a == b {1} else {0}),
            "!=" => left.op_num(self, right, |a, b| if a != b {1} else {0}),
            ">" => left.op_num(self, right, |a, b| if a > b {1} else {0}),
            "<" => left.op_num(self, right, |a, b| if a < b {1} else {0}),
            ">=" => left.op_num(self, right, |a, b| if a >= b {1} else {0}),
            "<=" => left.op_num(self, right, |a, b| if a <= b {1} else {0}),
            "&&" => left.op_num(self, right, |a, b| if a != 0 && b != 0 {1} else {0}),
            "||" => left.op_num(self, right, |a, b| if a != 0 || b != 0 {1} else {0}),
            "=" => left.op_let(self, right, |a, b| *a = b),
            // "+=" => left.op_let(self, right, |a, b| *a += b),
            // "-=" => left.op_let(self, right, |a, b| *a -= b),
            // "*=" => left.op_let(self, right, |a, b| *a *= b),
            // "/=" => left.op_let(self, right, |a, b| *a /= b),
            // "%=" => left.op_let(self, right, |a, b| *a %= b),
            // "|=" => left.op_let(self, right, |a, b| *a |= b),
            // "&=" => left.op_let(self, right, |a, b| *a &= b),
            // "^=" => left.op_let(self, right, |a, b| *a ^= b),
            // ">>=" => left.op_let(self, right, |a, b| *a >>= b),
            // "<<=" => left.op_let(self, right, |a, b| *a <<= b),
            _ => unreachable!(),
        }
    }
}

/// `Error` を実装した構造体をボックス化して `Result` 列挙型に入れたものを返すマクロです。
#[macro_export]
macro_rules! ret_err {
    ($x: expr) => {
        return Err(Box::new($x))
    };
}

/// 1つの要素が入った `VecDeque` を生成するマクロです。
#[macro_export]
macro_rules! vec_deque {
    ($x: expr) => {
        {
            let mut tmp = VecDeque::new();
            tmp.push_back($x);
            tmp
        }
    };
}

/// 文字の種類を定義します。要素に分割する時に使用します。
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

/// 式の要素の種類を定義します。
#[derive(Debug, Clone)]
pub enum ElementType {
    Variable(String),
    Immediate(VarType),
    Monomial(String, Rc<ElementType>)
}

impl ElementType {
    fn get(&self) -> &Self {self}
    /// 数値へ変換します。
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
                        _ => ret_err!(OperationError::invalid_dereference())
                    }
                } else {
                    match e.to_vartype(expr)? {
                        VarType::Uninitialized | VarType::Void => ret_err!(OperationError::operate_with_void()),
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
        ret_err!(InvalidExpressionError::new("Cannot operate with void"))
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
                None => {ret_err!(VariableNotFoundError::new(v))},
            }
        }
        ret_err!(InvalidExpressionError::new("The left-hand must be variable."));
    }

}

#[derive(Debug, Clone)]
pub enum VarType {
    Uninitialized,
    Void,
    Integer(i32),
    String(String),
    Pointer(String),
}

impl VarType {
    fn is_empty(&self) -> bool {
        match self {
            VarType::Uninitialized | VarType::Void => true,
            _ => false
        }
    }
}