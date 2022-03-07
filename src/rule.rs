use crate::keycodes::keycode;
use anyhow::{anyhow, Result};
use rdev::Key;
use std::collections::HashSet;

pub trait Rule {
    fn change(&self, state: &HashSet<i32>) -> bool;
    fn focus(&self) -> &str;
}

pub struct SimpleRule {
    keys: Vec<i32>,
    focus: String,
}

impl SimpleRule {
    pub fn new(keys: Vec<Key>, focus: &str) -> Result<Self> {
        Ok(Self {
            keys: keys
                .iter()
                .map(|key| keycode(*key).ok_or(anyhow!("rule keycode {:?}", key)))
                .collect::<Result<_>>()?,
            focus: focus.to_owned(),
        })
    }
}

impl Rule for SimpleRule {
    fn change(&self, state: &HashSet<i32>) -> bool {
        self.keys.iter().all(|key| state.get(key).is_some())
    }
    fn focus(&self) -> &str {
        &self.focus
    }
}
