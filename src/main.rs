mod keycodes;
use keycodes::keycode;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;
static KEY_DEPRESSED: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Credit to https://github.com/kb24x7/rustyvibes

fn main() -> Result<()> {
    rdev::listen(|event| {
        report_err(|| {
            match event.event_type {
                rdev::EventType::KeyPress(key) => {
                    let key_code = keycode(key).ok_or(anyhow!("keycode"))?;
                    println!("KeyPress: {}", key_code);
                    KEY_DEPRESSED.lock().unwrap().insert(key_code);
                }
                rdev::EventType::KeyRelease(key) => {
                    let key_code = keycode(key).ok_or(anyhow!("keycode"))?;
                    println!("KeyRelease: {}", key_code);
                    KEY_DEPRESSED.lock().unwrap().remove(&key_code);
                }
                _ => {}
            }
            Ok(())
        })
    })
    .map_err(|err| anyhow!("could not listen: {:?}", err))?;
    Ok(())
}

fn report_err<F>(f: F)
where
    F: FnOnce() -> Result<()>,
{
    match f() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("error {}", err)
        }
    }
}
