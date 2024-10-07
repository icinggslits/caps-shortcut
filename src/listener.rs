#![allow(unused)]

#[cfg(target_os = "windows")]
pub mod windows;

use rdev::Key;
#[cfg(target_os = "windows")]
pub use windows::caps_with;
#[cfg(target_os = "windows")]
pub use windows::run;
#[cfg(target_os = "windows")]
pub use windows::caps_of_modifier_key_with;
#[cfg(target_os = "windows")]
pub use windows::caps_listener_with;

pub fn is_modifier_key(key: Key) -> bool {
    use Key::*;
    
    match key {
        ControlLeft |
        ControlRight |
        Alt |
        AltGr |
        ShiftLeft |
        ShiftRight |
        MetaRight |
        MetaLeft
        => true,
        _ => false,
    }
}