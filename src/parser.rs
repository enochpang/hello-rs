use http::Uri;
use std::{collections::HashMap, fmt, sync::OnceLock};

use crate::lexer::{Lexer, Token, TokenKind};

pub static REQUEST_KEYWORDS: OnceLock<HashMap<&str, ReqKind>> = OnceLock::new();
pub static ACTION_KEYWORDS: OnceLock<HashMap<&str, ActionKind>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum RuleItem {
    Filter(FilterRule),
    Setting(SettingRule),
}

#[derive(Debug, Clone)]
pub struct FilterRule {
    pub source: Uri,
    pub destination: Uri,
    pub req_type: ReqKind,
    pub action_type: ActionKind,
}

#[derive(Debug, Clone)]
pub struct SettingRule {
    pub name: String,
    pub location: String,
    pub val: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ReqKind {
    All,
    Image,
    InlineScript,
    FstpScript,
    Thrdp,
    ThrdpScript,
    ThrdpFrame,
}

impl fmt::Display for ReqKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ReqKind::All => write!(f, "*"),
            ReqKind::Image => write!(f, "image"),
            ReqKind::InlineScript => write!(f, "inline-script"),
            ReqKind::FstpScript => write!(f, "1p-script"),
            ReqKind::Thrdp => write!(f, "3p"),
            ReqKind::ThrdpScript => write!(f, "3p-script"),
            ReqKind::ThrdpFrame => write!(f, "3p-frame"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActionKind {
    Block,
    Noop,
    Allow,
}

impl fmt::Display for ActionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ActionKind::Block => write!(f, "block"),
            ActionKind::Noop => write!(f, "noop"),
            ActionKind::Allow => write!(f, "allow"),
        }
    }
}

pub struct Parser<'lex> {
    lexer: &'lex mut Lexer,
    tok: Token,
}

impl<'lex> Parser<'lex> {
    pub fn new(lexer: &mut Lexer) -> Parser<'_> {
        let token = lexer.next_token();
        Parser { lexer, tok: token }
    }

    /// Processes all the text in the lexer.
    pub fn parse(&mut self) -> Option<Vec<RuleItem>> {
        let mut items = Vec::new();

        while self.tok.kind != TokenKind::End {
            let line_parts = self.read_line();

            if line_parts.len() == 4 {
                let part1 = &line_parts[0];
                let part2 = &line_parts[1];
                let part3 = &line_parts[2];
                let part4 = &line_parts[3];

                let url_src = match part1.parse::<Uri>() {
                    Ok(x) => x.clone(),
                    Err(e) => panic!("{}: {}", e, part1),
                };
                let url_dest = match part2.parse::<Uri>() {
                    Ok(x) => x.clone(),
                    Err(e) => panic!("{}: {}", e, part2),
                };
                let req_type = match get_request_keywords().get(part3.as_str()) {
                    Some(x) => *x,
                    None => panic!("Could not parse request: {}", part3),
                };
                let action_type = match get_action_keywords().get(part4.as_str()) {
                    Some(x) => *x,
                    None => panic!("Could not parse action: {}", part4),
                };

                let rule = FilterRule {
                    source: url_src,
                    destination: url_dest,
                    req_type,
                    action_type,
                };

                items.push(RuleItem::Filter(rule));
            } else if line_parts.len() == 3 {
                let setting = SettingRule {
                    name: line_parts[0].clone(),
                    location: line_parts[1].clone(),
                    val: line_parts[2].clone(),
                };

                items.push(RuleItem::Setting(setting))
            } else {
                return None;
            }
        }

        Some(items)
    }

    /// Returns a list of words in the next line
    fn read_line(&mut self) -> Vec<String> {
        let mut items = Vec::new();

        while self.tok.kind != TokenKind::Eol && self.tok.kind != TokenKind::End {
            items.push(self.tok.lexeme.clone());
            self.next();
        }

        // The Eol or END
        self.next();

        items
    }

    /// Move the parser to the next token in the lexer.
    fn next(&mut self) {
        self.tok = self.lexer.next_token();
    }
}

pub fn get_request_keywords() -> &'static HashMap<&'static str, ReqKind> {
    REQUEST_KEYWORDS.get_or_init(|| {
        let mut keywords = HashMap::new();
        keywords.insert("*", ReqKind::All);
        keywords.insert("image", ReqKind::Image);
        keywords.insert("inline-script", ReqKind::InlineScript);
        keywords.insert("1p-script", ReqKind::FstpScript);
        keywords.insert("3p", ReqKind::Thrdp);
        keywords.insert("3p-script", ReqKind::ThrdpScript);
        keywords.insert("3p-frame", ReqKind::ThrdpFrame);
        keywords
    })
}

pub fn get_action_keywords() -> &'static HashMap<&'static str, ActionKind> {
    ACTION_KEYWORDS.get_or_init(|| {
        let mut keywords = HashMap::new();
        keywords.insert("block", ActionKind::Block);
        keywords.insert("noop", ActionKind::Noop);
        keywords.insert("allow", ActionKind::Allow);
        keywords
    })
}
