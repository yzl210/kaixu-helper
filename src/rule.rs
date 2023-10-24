use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::model::channel::Message;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ReplyRule {
    rule_type: String,
    rule: String,
    pub(crate) reply: String,
}

impl ReplyRule {
    pub(crate) fn check(&self, msg: &Message) -> bool {
        match self.rule_type.as_str() {
            "content_regex" => match Regex::new(self.rule.as_str()) {
                Ok(regex) => regex.is_match(msg.content.as_str()),
                Err(_) => {
                    println!("Invalid regex: {}", self.rule);
                    false
                }
            },
            "author_name" => msg.author.name == self.rule,
            "author_id" => msg.author.id.to_string() == self.rule,
            _ => {
                println!("Invalid rule type: {}", self.rule_type);
                false
            }
        }
    }
}
