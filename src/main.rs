mod keycodes;
mod rule;
use keycodes::keycode;
use rule::{Rule, SimpleRule, TwoStepRule};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rdev::{simulate, Event, EventType, Key};
use std::collections::HashSet;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static KEY_DEPRESSED: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Whether to keys pressed values.
static DEBUG_LOG: bool = false;

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
        Box::new(TwoStepRule::new(
            vec![Key::ShiftLeft, Key::MetaLeft],
            vec![Key::KeyS],
            "Spotify",
        )?),
        Box::new(SimpleRule::new(
            vec![Key::ShiftLeft, Key::ControlLeft, Key::UpArrow],
            "down arrow",
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
                if DEBUG_LOG {
                    println!("KeyPress: {}", key_code);
                } else {
                    println!("KeyPress");
                }
                {
                    let mut m = KEY_DEPRESSED.lock().unwrap();
                    m.insert(key_code);
                    if DEBUG_LOG {
                        println!(": {:?}", m);
                    }
                }
                let mut capture = false;
                for rule in rules.lock().unwrap().iter_mut() {
                    if rule.change(event, &KEY_DEPRESSED.lock().unwrap()) {
                        println!("focus {}", rule.focus());
                        capture = true;
                        match rule.focus() {
                            "down arrow" => {
                                press_key(Key::DownArrow);
                            }
                            _ => {
                                osascript.lock().unwrap().focus(&rule.focus())?;
                            }
                        }
                    }
                }
                Ok(capture)
            }
            rdev::EventType::KeyRelease(key) => {
                let key_code = keycode(key).ok_or(anyhow!("keycode release {:?}", key))?;
                if DEBUG_LOG {
                    println!("KeyRelease: {}", key_code);
                } else {
                    println!("KeyRelease");
                }
                {
                    let mut m = KEY_DEPRESSED.lock().unwrap();
                    m.remove(&key_code);
                    if DEBUG_LOG {
                        println!(": {:?}", m);
                    }
                }
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

/// Press and release a key in the background.
fn press_key(key: Key) {
    thread::spawn(move || {
        if let Err(err) = press_key_inner(key) {
            eprintln!("error pressing key: {:?}: {}", key, err)
        }
    });
}

fn press_key_inner(key: Key) -> Result<()> {
    let delay = Duration::from_millis(20);
    // [hack] hardcoded releasers for down arrow
    simulate(&EventType::KeyRelease(Key::ShiftLeft))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(Key::ControlLeft))?;
    thread::sleep(delay);
    simulate(&EventType::KeyPress(key))?;
    thread::sleep(delay);
    simulate(&EventType::KeyRelease(key))?;
    Ok(())
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
