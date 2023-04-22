use std::{fmt::Display, rc::Rc, collections::BTreeMap};

use log::info;

use super::ExprParser;

impl ExprParser {
    /// 可変な状態で変数を取得します。
    /// - `name` - 変数名
    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VarType> {
        self.variables.get_mut(name)
            .map(|a| a.range_mut(..=self.depth)
                .rev()
                .find(|b| *b.0 == self.depth || b.1 != &VarType::Uninitialized)
                .map(|b| b.1)
            )
            .unwrap_or(None)
        }

        /// 変数を取得します。
        /// - `name` - 変数名
        pub fn get_variable(&self, name: &str) -> &VarType {
            self.variables.get(name)
            .map(|a| a.range(..=self.depth)
                .rev()
                .find(|b| b.1 != &VarType::Uninitialized)
                .map(|a| a.1)
                .unwrap_or(&VarType::Uninitialized)
            )
            .unwrap_or(&VarType::Uninitialized)
    }

    /// 変数が存在するかを確認します。
    /// - `name` - 変数名
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 変数を作成します。
    /// - `name` - 新しく作成する変数名
    pub fn create_variable(&mut self, name: String){
        match self.variables.get_mut(&name) {
            Some(a) => {
                if a.last_key_value().map(|a| *a.0 != self.depth).unwrap_or(true) {
                    info!("Variable \"{}\" was created with a new scope.", name);
                    a.insert(self.depth, VarType::Uninitialized);
                }
            },
            None => {
                info!("Variable \"{}\" was created.", name);
                let mut tmp = BTreeMap::new();
                tmp.insert(self.depth, VarType::Uninitialized);
                self.variables.insert(name, tmp);
            },
        }
        info!("{:?}", self.variables);
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