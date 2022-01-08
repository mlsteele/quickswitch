mod keycodes;
use keycodes::keycode;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rdev::{Event, Key};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::Mutex;

type Rule = ([Key; 3], &'static str);

static KEY_DEPRESSED: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Credit to https://github.com/kb24x7/rustyvibes

const RULES: &[&Rule] = &[
    &([Key::ShiftLeft, Key::MetaLeft, Key::KeyU], "iTerm"),
    &(
        [Key::ShiftLeft, Key::MetaLeft, Key::KeyI],
        "Visual Studio Code",
    ),
    &([Key::ShiftLeft, Key::MetaLeft, Key::KeyO], "Firefox"),
    &([Key::ShiftLeft, Key::MetaLeft, Key::KeyK], "Keybase"),
];

fn main() -> Result<()> {
    let mut triggers: HashMap<i32, Rule> = HashMap::new();
    for rule in RULES {
        for key in rule.0 {
            triggers.insert(keycode(key).ok_or(anyhow!("keycode"))?, **rule);
        }
    }
    rdev::grab(move |event| {
        report_err(event, |event| match event.event_type {
            rdev::EventType::KeyPress(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode"))?;
                // println!("KeyPress: {}", key_code);
                KEY_DEPRESSED.lock().unwrap().insert(key_code);
                let mut capture = false;
                for rule in triggers.get(&key_code) {
                    if rule.0.iter().all(|key| {
                        KEY_DEPRESSED
                            .lock()
                            .unwrap()
                            .get(&keycode(*key).ok_or(anyhow!("keycode")).unwrap())
                            .is_some()
                    }) {
                        println!("focus {}", rule.1);
                        focus(rule.1)?;
                        capture = true;
                    }
                }
                Ok(capture)
            }
            rdev::EventType::KeyRelease(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode {:?}", key))?;
                // println!("KeyRelease: {}", key_code);
                KEY_DEPRESSED.lock().unwrap().remove(&key_code);
                Ok(false)
            }
            _ => Ok(false),
        })
    })
    .map_err(|err| anyhow!("could not listen: {:?}", err))?;
    Ok(())
}

fn report_err<F>(event: Event, f: F) -> Option<Event>
where
    F: FnOnce(&Event) -> Result<bool>,
{
    match f(&event) {
        Ok(true) => None,
        Ok(false) => Some(event),
        Err(err) => {
            eprintln!("error {}", err);
            Some(event)
        }
    }
}

/// Executes the script and passes the provided arguments.
fn focus(app: &str) -> Result<()> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(format!("tell application \"{}\" to activate", app))
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("osascript: {}", String::from_utf8(output.stderr)?))
    }
}
