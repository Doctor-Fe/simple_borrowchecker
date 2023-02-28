use std::{collections::VecDeque, error::Error};

use crate::{ret_err, parser::errors::InvalidExpressionError};

use super::{ExprParser, ElementType, VarType};

impl ExprParser {
    /// 二項演算子の優先順位を返します。
    /// * `op` - 優先順位を取得する演算子
    pub fn get_priority(op: &str) -> Option<usize> {
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
    pub fn is_monomial(op: &str) -> bool {
        let ops = ["+", "-", "&", "*", "!", "~"];
        return ops.contains(&op);
    }

    pub fn try_calculate_all(&mut self, mut data: (String, VecDeque<ElementType>)) -> Result<VarType, Box<dyn Error>> {
        if data.0 != "=" {
            let mut num = data.1.pop_front().unwrap();
            while !data.1.is_empty() {
                let d = data.1.pop_front().unwrap();
                num = match self.calculate_binomial(&data.0, num, d)? {
                    VarType::Integer(i) => ElementType::Immediate(VarType::Integer(i)),
                    _ => return Ok(VarType::Void),
                };
            }
            return num.to_vartype(self);
        } else {
            let mut num = data.1.pop_back().unwrap();
            while !data.1.is_empty() {
                let d = data.1.pop_back().unwrap();
                num = match self.calculate_binomial(&data.0, d, num)? {
                    VarType::Integer(i) => ElementType::Immediate(VarType::Integer(i)),
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
            a => ret_err!(InvalidExpressionError::new(format!("Invalid operator \"{}\".", a))),
        }
    }
}
