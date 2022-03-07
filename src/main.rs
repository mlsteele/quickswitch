mod keycodes;
mod rule;
use keycodes::keycode;
use rule::{Rule, SimpleRule, TwoStepRule};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rdev::{Event, Key};
use std::collections::HashSet;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

static KEY_DEPRESSED: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Credit to https://github.com/kb24x7/rustyvibes

fn main() -> Result<()> {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(SimpleRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft, Key::KeyU],
            "iTerm",
        )?),
        Box::new(SimpleRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft, Key::KeyI],
            "Visual Studio Code",
        )?),
        Box::new(SimpleRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft, Key::KeyO],
            "Firefox",
        )?),
        Box::new(SimpleRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft, Key::KeyK],
            "Keybase",
        )?),
        Box::new(TwoStepRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft],
            vec![Key::KeyN],
            "Notion",
        )?),
    ];
    let rules = Mutex::new(rules);

    let mut osascript = OsaScript::new();
    osascript.start()?;
    let osascript = Mutex::new(osascript);

    rdev::grab(move |event| {
        report_err(event, |event| match event.event_type {
            rdev::EventType::KeyPress(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode press: {:?}", key))?;
                // println!("KeyPress: {}", key_code);
                KEY_DEPRESSED.lock().unwrap().insert(key_code);
                let mut capture = false;
                for rule in rules.lock().unwrap().iter_mut() {
                    if rule.change(event, &KEY_DEPRESSED.lock().unwrap()) {
                        println!("focus {}", rule.focus());
                        osascript.lock().unwrap().focus(&rule.focus())?;
                        capture = true;
                    }
                }
                Ok(capture)
            }
            rdev::EventType::KeyRelease(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode release {:?}", key))?;
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
