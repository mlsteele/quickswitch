use crate::keycodes::keycode;
use anyhow::{anyhow, Result};
use rdev;
use rdev::{Event, Key};
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub trait Rule {
    fn change(&mut self, event: &Event, state: &HashSet<i32>) -> bool;
    fn focus(&self) -> &str;
}

pub struct SimpleRule {
    keys: Vec<i32>,
    focus: String,
}

impl SimpleRule {
    pub fn new(keys: Vec<Key>, focus: &str) -> Result<Self> {
        Ok(Self {
            keys: keys_to_codes(&keys)?,
            focus: focus.to_owned(),
        })
    }
}

impl Rule for SimpleRule {
    fn change(&mut self, _event: &Event, state: &HashSet<i32>) -> bool {
        self.keys.iter().all(|key| state.get(key).is_some())
    }
    fn focus(&self) -> &str {
        &self.focus
    }
}

pub struct TwoStepRule {
    keys1: Vec<i32>,
    keys2: Vec<i32>,
    focus: String,
    last_match: Option<Instant>,
}

impl TwoStepRule {
    pub fn new(keys1: Vec<Key>, keys2: Vec<Key>, focus: &str) -> Result<Self> {
        Ok(Self {
            keys1: keys_to_codes(&keys1)?,
            keys2: keys_to_codes(&keys2)?,
            focus: focus.to_owned(),
            last_match: Default::default(),
        })
    }
}

impl Rule for TwoStepRule {
    fn change(&mut self, event: &Event, state: &HashSet<i32>) -> bool {
        let match1 = self.keys1.iter().all(|key| state.get(key).is_some());
        let match2 = self.keys2.iter().all(|key| state.get(key).is_some());
        if match1 {
            self.last_match = Some(Instant::now());
        }
        if let rdev::EventType::KeyPress(key) = event.event_type {
            if let Some(key) = keycode(key) {
                if !self.keys1.contains(&key) && !self.keys2.contains(&key) {
                    // Disccard readiness if anything irrelevant is pressed.
                    self.last_match = None
                }
            }
        }
        !match1
            && match2
            && self.last_match.is_some()
            && Instant::now().duration_since(self.last_match.unwrap()) < Duration::from_millis(750)
    }
    fn focus(&self) -> &str {
        &self.focus
    }
}

fn keys_to_codes(keys: &Vec<Key>) -> Result<Vec<i32>> {
    keys.iter()
        .map(|key| keycode(*key).ok_or(anyhow!("rule keycode {:?}", key)))
        .collect::<Result<_>>()
}
