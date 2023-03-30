use super::ExprParser;

impl ExprParser {
    /// 入力された文字列を要素毎に分割します。
    /// * `cmd` - 分割する文字列
    pub fn split_elements(&mut self, cmd: &String) {
        let mut tmp = cmd.chars().rev().collect::<Vec<char>>();
        let mut word: Vec<char> = Vec::new();
        let mut is_string = false;
        let mut comment_out: Option<CommentType> = None;
        while !tmp.is_empty() {
            let a = tmp.pop().unwrap();
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
                    },
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
                        if !word.is_empty() && CharType::get_chartype(*word.last().unwrap()) != CharType::Normal {
                            self.cmds.push(String::from_iter(&word));
                            word.clear();
                        }
                        word.push(a);
                    },
                    CharType::Punctuation => {
                        if a == '"' {
                            is_string = !is_string;
                        } else {
                            if !word.is_empty() && (Self::get_priority(String::from_iter([word.clone(), vec![a]].concat()).as_str()).is_none()) {
                                self.cmds.push(String::from_iter(&word));
                                word.clear();
                            }
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