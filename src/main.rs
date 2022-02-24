mod keycodes;
use keycodes::keycode;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rdev::{Event, Key};
use std::collections::{HashMap, HashSet};
use std::process::{Child, Command, Stdio};
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
            triggers.insert(keycode(key).ok_or(anyhow!("keycode {:?}", key))?, **rule);
        }
    }
    let mut osascript = OsaScript::new();
    osascript.start()?;
    let x = Mutex::new(osascript);
    rdev::grab(move |event| {
        report_err(event, |event| match event.event_type {
            rdev::EventType::KeyPress(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode: {:?}", key))?;
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
                        x.lock().unwrap().focus(rule.1)?;
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

struct OsaScript {
    proc: Option<Child>,
}

impl OsaScript {
    pub fn new() -> Self {
        OsaScript { proc: None }
    }

    pub fn start(&mut self) -> Result<()> {
        self.proc = Some(Self::spawn()?);
        Ok(())
    }

    fn spawn() -> Result<Child> {
        Ok(Command::new("osascript").stdin(Stdio::piped()).spawn()?)
    }

    pub fn focus(&mut self, app: &str) -> Result<()> {
        use std::io::Write;
        let mut child: Child = self.proc.take().expect("child must exist");
        self.proc = Some(Self::spawn()?);
        writeln!(
            child.stdin.take().expect("child stdin must exist"),
            "tell application \"{}\" to activate",
            app
        )?;
        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!("osascript: {}", String::from_utf8(output.stderr)?))
        }
    }
}
