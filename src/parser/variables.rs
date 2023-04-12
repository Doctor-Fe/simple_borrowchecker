use std::{fmt::Display, rc::Rc};

use log::info;

use super::ExprParser;

impl ExprParser {
    /// 可変な状態で変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VarType> {
        self.variables.get_mut(name)
    }
    
    /// 変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable(&self, name: &str) -> Option<&VarType> {
        self.variables.get(name)
    }

    /// 変数が存在するかを確認します。
    /// - `name` - 変数名
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 変数を作成します。
    /// - `name` - 新しく作成する変数名
    pub fn create_variable(&mut self, name: String){
        info!("Variable \"{}\" was created.", name);
        if !self.variables.contains_key(&name) {
            self.variables.insert(name, VarType::Uninitialized);
        }
    }
}

/// 変数として格納可能な値を保持する構造体です。
#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    /// 未初期化の変数であることを表します。
    Uninitialized,
    /// 値がないことを表します。
    Void,
    /// 整数であることを表します。
    Integer(i32),
    /// 文字列であることを表します。
    String(String),
    /// ポインタであることを表します。
    Pointer(Rc<VarType>),
}

impl VarType {
    /// 値が空であるかを取得する関数です。
    pub fn is_empty(&self) -> bool {
        matches!(self, VarType::Uninitialized | VarType::Void)
    }

    /// 新しくVarType::Stringを作成します。
    /// * `data` - 文字列の内容
    pub fn new_string(data: &str) -> Self {
        VarType::String(data.trim_matches('"').to_string())
    }
}

impl Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PartialOrd for VarType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VarType::Integer(a), VarType::Integer(b)) => Some(a.cmp(b)),
            _ => None
        }
    }
}