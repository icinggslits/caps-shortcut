use std::borrow::Borrow;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::sync::{atomic, Arc, Mutex, OnceLock, RwLock};
use std::sync::atomic::AtomicBool;
use lock_keys::LockKeyWrapper;
use rdev::Key;
use winapi::um::winuser::{CallNextHookEx, GetAsyncKeyState, GetKeyState, GetMessageW, SendInput, SetWindowsHookExW, HC_ACTION, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYEVENTF_KEYUP, VK_CAPITAL, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP};
use winapi::shared::minwindef::{LRESULT, WPARAM, LPARAM, HINSTANCE};
use winapi::um::libloaderapi::GetModuleHandleW;


#[derive(Debug, Clone, Copy)]
pub struct Keyboard {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub key: Key,
}


static PRESSED_CTRL: AtomicBool = AtomicBool::new(false);
static PRESSED_SHIFT: AtomicBool = AtomicBool::new(false);
static PRESSED_ALT: AtomicBool = AtomicBool::new(false);
static PRESSED_META: AtomicBool = AtomicBool::new(false);
static PRESSED_CAPITAL: AtomicBool = AtomicBool::new(false);
/// 需要还原大写锁定
static NEED_RESTORE: AtomicBool = AtomicBool::new(false);


struct Win;

impl Win {
    fn key_pressed(vk: i32) -> bool {
        unsafe {
            // 检查按键的状态
            GetAsyncKeyState(vk) & 0x8000u16 as i16 != 0
        }
    }

    fn ctrl() -> bool {
        Self::key_pressed(VK_CONTROL)
    }

    fn shift() -> bool {
        Self::key_pressed(VK_SHIFT)
    }

    fn alt() -> bool {
        Self::key_pressed(VK_MENU)
    }

    fn meta() -> bool {
        Self::key_pressed(VK_LWIN) || Self::key_pressed(VK_RWIN)
    }

    fn capital() -> bool {
        unsafe {
            GetKeyState(VK_CAPITAL) & 0x0001 == 0
        }
    }
    fn input_caps() {
        unsafe {
            // 获取Caps Lock键的状态
            // let caps_lock_state = GetKeyState(VK_CAPITAL) & 0x0001;
    
            // 如果Caps Lock是开启的，就按下再释放Caps Lock键进行切换
            // if caps_lock_state == 0 {
            //     println!("Caps Lock is off, turning it on...");
            // } else {
            //     println!("Caps Lock is on, turning it off...");
            // }
    
            // println!("{}", caps_lock_state);
    
            // 模拟按下Caps Lock键
            let mut inputs = [
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed(),
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed(),
                }
            ];
    
            // 设置按下键的输入
            inputs[0].u.ki_mut().wVk = VK_CAPITAL as u16;
    
            // 设置释放键的输入
            inputs[1].u.ki_mut().wVk = VK_CAPITAL as u16;
            inputs[1].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
    
            // 发送输入事件
            SendInput(2, inputs.as_mut_ptr(), size_of::<INPUT>() as i32);
        }
    }

    fn keyboard_keyed(code: u32, keydown: bool) -> Keyboard {
        let key =  code_to_key(code);

        let mut keyboard = Keyboard { 
            ctrl: Win::ctrl(),
            shift: Win::shift(),
            alt: Win::alt(),
            meta: Win::meta(),
            key: code_to_key(code),
        };

        match key {
            Key::ControlLeft | Key::ControlRight => {
                keyboard.ctrl = keydown;
            },
            Key::ShiftLeft | Key::ShiftRight => {
                keyboard.shift = keydown;
            },
            Key::Alt | Key::AltGr => {
                keyboard.alt = keydown;
            }
            Key::MetaLeft | Key::MetaRight => {
                keyboard.meta = keydown;
            },
            _ => (),
        }

        keyboard
    }
}


unsafe extern "system" fn hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    
    if code == HC_ACTION as i32 {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        

        if w_param == WM_KEYDOWN as WPARAM || w_param == 260 {
            let vk_code = kb_struct.vkCode as i32;
            let vk_code_u32 = kb_struct.vkCode;

            let mut intercept = false;

            match vk_code {
                VK_LCONTROL | VK_RCONTROL | VK_CONTROL => {
                    // PRESSED_CTRL.store(true, atomic::Ordering::Relaxed);
                },
                VK_LSHIFT | VK_RSHIFT | VK_SHIFT => {
                    // PRESSED_SHIFT.store(true, atomic::Ordering::Relaxed);
                },
                VK_LMENU | VK_RMENU | VK_MENU => {
                    // PRESSED_ALT.store(true, atomic::Ordering::Relaxed);
                },
                VK_LWIN | VK_RWIN => {
                    // PRESSED_META.store(true, atomic::Ordering::Relaxed);
                },
                VK_CAPITAL => {
                    PRESSED_CAPITAL.store(true, atomic::Ordering::Relaxed);
                }
                _ => ()
            }

            // {
            //     let keyboard = Win::keyboard_keyed(vk_code_u32, true);
            //     println!("按下 ctrl: {}, shift: {}, alt: {}, meta: {}, key: {:?}", keyboard.ctrl, keyboard.shift, keyboard.alt, keyboard.meta, keyboard.key);
            // }


            
            let caps = PRESSED_CAPITAL.load(atomic::Ordering::Relaxed);
            // let caps = Win::capital();
            if caps {
                let mut caps_listener = caps_listener_global().write().unwrap();
                for f in caps_listener.iter_mut() {
                    
                    let keyboard = Win::keyboard_keyed(vk_code_u32, true);
                    
                    let _intercept = f(keyboard);
    
                    if !intercept && _intercept {
                        
                        intercept = true;
                    }
                    
                }
            }

            if intercept {
                NEED_RESTORE.store(true, atomic::Ordering::Relaxed);
                return 1
            }
            
        } else if w_param == WM_KEYUP as WPARAM || w_param == 261 {

            let vk_code = kb_struct.vkCode as i32;
            // let vk_code_u32 = kb_struct.vkCode;
            
            // let Keyboard {ctrl, shift, alt, meta, ..} = &Win::keyboard_keyed(vk_code_u32, false);
            
            // println!("松开 ctrl: {}, shift: {}, alt: {}, meta: {}, CAPS {}", ctrl, shift, alt, meta, Win::capital());

            
            match vk_code {
                VK_LCONTROL | VK_RCONTROL | VK_CONTROL => {
                    // PRESSED_CTRL.store(false, atomic::Ordering::Relaxed);
                },
                VK_LSHIFT | VK_RSHIFT | VK_SHIFT => {
                    // PRESSED_SHIFT.store(false, atomic::Ordering::Relaxed);
                }
                VK_LMENU | VK_RMENU | VK_MENU => {
                    // PRESSED_ALT.store(false, atomic::Ordering::Relaxed);
                },
                VK_LWIN | VK_RWIN => {
                    // PRESSED_META.store(false, atomic::Ordering::Relaxed);
                },
                VK_CAPITAL => {
                    PRESSED_CAPITAL.store(false, atomic::Ordering::Relaxed);
                    if NEED_RESTORE.load(atomic::Ordering::Relaxed) {
                        std::thread::spawn(|| {
                            Win::input_caps();
                        });
                        NEED_RESTORE.store(false, atomic::Ordering::Relaxed);
                    }
                }
                _ => (),
            }
            
        }
    }
    // 继续执行下一个钩子
    CallNextHookEx(null_mut(), code, w_param, l_param)
}

static KEY_AND_FN: OnceLock<RwLock<HashMap<(u32, Vec<u32>), Box<dyn FnMut() + Send + Sync>>>> = OnceLock::new();

static CAPS_LISTENER: OnceLock<RwLock<Vec<Box<dyn FnMut(Keyboard) -> bool + Send + Sync>>>> = OnceLock::new();

fn key_and_fn_global() -> &'static RwLock<HashMap<(u32, Vec<u32>), Box<dyn FnMut() + Send + Sync>>> {
    &KEY_AND_FN.get_or_init(|| {
        RwLock::new(HashMap::new())
    })
}

fn caps_listener_global() -> &'static RwLock<Vec<Box<dyn FnMut(Keyboard) -> bool + Send + Sync>>> {
    CAPS_LISTENER.get_or_init(|| {
        RwLock::new(Vec::new())
    })
}


pub fn caps_with<F: FnMut() + 'static>(key: Key, f: F) {
    caps_of_modifier_key_with(key, [], f);
}

pub fn caps_of_modifier_key_with<I: IntoIterator<Item = Key>, F: FnMut() + 'static>(key: Key, modifier_key: I, f: F) {
    let code = key_to_code(key);
    let mut modifier_key_list = Vec::new();

    for key in modifier_key {
        let key = match key {
            Key::ControlLeft | 
            Key::ControlRight |
            Key::Alt |
            Key::ShiftLeft |
            Key::ShiftRight
            => vec![key_to_code(key)],

            Key::AltGr => vec![key_to_code(Key::ControlLeft), key_to_code(Key::Alt)],

            _ => panic!(),
        };

        modifier_key_list.extend(key);
    }

    let key_and_fn = key_and_fn_global();
    let mut key_and_fn = key_and_fn.write().unwrap();
   


    let p = &f as *const F;
    let p = p as usize;

    


    todo!()
}


struct AtListenerFn {
    f: Box<dyn FnMut(Keyboard) -> bool + 'static>,
}

impl AtListenerFn {
    fn new<F: FnMut(Keyboard) -> bool + 'static>(f: F) -> Self {
        Self {
            f: Box::new(f),
        }
    }
}

pub fn caps_listener_with<F: FnMut(Keyboard) -> bool + Send + Sync + 'static>(f: F) {
    let mut caps_listener = caps_listener_global().write().unwrap();
    // let at_fn = AtListenerFn::new(f);
    // let p = &at_fn as *const AtListenerFn;
    // let p = p as usize;
    // caps_listener.push(p);
    // println!("caps_listener_with ptr -> {}", p);


    caps_listener.push(Box::new(f));
}


pub fn run() {
    unsafe {
        let h_instance: HINSTANCE = GetModuleHandleW(null_mut());
        let _hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), h_instance, 0);
        let mut msg = std::mem::zeroed();
        GetMessageW(&mut msg, null_mut(), 0, 0);
    }
}


fn key_to_code(key: Key) -> u32 {
    use Key::*;
    
    match key {
        Alt => 164,
        AltGr => 165,
        Backspace => 0x08,
        CapsLock => 20,
        ControlLeft => 162,
        ControlRight => 163,
        Delete => 46,
        DownArrow => 40,
        End => 35,
        Escape => 27,
        F1 => 112,
        F10 => 121,
        F11 => 122,
        F12 => 123,
        F2 => 113,
        F3 => 114,
        F4 => 115,
        F5 => 116,
        F6 => 117,
        F7 => 118,
        F8 => 119,
        F9 => 120,
        Home => 36,
        LeftArrow => 37,
        MetaLeft => 91,
        PageDown => 34,
        PageUp => 33,
        Return => 0x0D,
        RightArrow => 39,
        ShiftLeft => 160,
        ShiftRight => 161,
        Space => 32,
        Tab => 0x09,
        UpArrow => 38,
        PrintScreen => 44,
        ScrollLock => 145,
        Pause => 19,
        NumLock => 144,
        BackQuote => 192,
        Num1 => 49,
        Num2 => 50,
        Num3 => 51,
        Num4 => 52,
        Num5 => 53,
        Num6 => 54,
        Num7 => 55,
        Num8 => 56,
        Num9 => 57,
        Num0 => 48,
        Minus => 189,
        Equal => 187,
        KeyQ => 81,
        KeyW => 87,
        KeyE => 69,
        KeyR => 82,
        KeyT => 84,
        KeyY => 89,
        KeyU => 85,
        KeyI => 73,
        KeyO => 79,
        KeyP => 80,
        LeftBracket => 219,
        RightBracket => 221,
        KeyA => 65,
        KeyS => 83,
        KeyD => 68,
        KeyF => 70,
        KeyG => 71,
        KeyH => 72,
        KeyJ => 74,
        KeyK => 75,
        KeyL => 76,
        SemiColon => 186,
        Quote => 222,
        BackSlash => 220,
        IntlBackslash => 226,
        KeyZ => 90,
        KeyX => 88,
        KeyC => 67,
        KeyV => 86,
        KeyB => 66,
        KeyN => 78,
        KeyM => 77,
        Comma => 188,
        Dot => 190,
        Slash => 191,
        Insert => 45,
        //KP_RETURN, 13,
        KpMinus => 109,
        KpPlus => 107,
        KpMultiply => 106,
        KpDivide => 111,
        Kp0 => 96,
        Kp1 => 97,
        Kp2 => 98,
        Kp3 => 99,
        Kp4 => 100,
        Kp5 => 101,
        Kp6 => 102,
        Kp7 => 103,
        Kp8 => 104,
        Kp9 => 105,
        KpDelete => 110,
        MetaRight | KpReturn | Function => panic!(),
        Unknown(code) => code,
    }
}


fn code_to_key<N: Into<u32>>(code: N) -> Key {
    let code = code.into();
    use Key::*;
    
    match code {
        164 => Alt,
        165 => AltGr,
        0x08 => Backspace,
        20 => CapsLock,
        162 => ControlLeft,
        163 => ControlRight,
        46 => Delete,
        40 => DownArrow,
        35 => End,
        27 => Escape,
        112 => F1,
        121 => F10,
        122 => F11,
        123 => F12,
        113 => F2,
        114 => F3,
        115 => F4,
        116 => F5,
        117 => F6,
        118 => F7,
        119 => F8,
        120 => F9,
        36 => Home,
        37 => LeftArrow,
        91 => MetaLeft,
        34 => PageDown,
        33 => PageUp,
        0x0D => Return,
        39 => RightArrow,
        160 => ShiftLeft,
        161 => ShiftRight,
        32 => Space,
        0x09 => Tab,
        38 => UpArrow,
        44 => PrintScreen,
        145 => ScrollLock,
        19 => Pause,
        144 => NumLock,
        192 => BackQuote,
        49 => Num1,
        50 => Num2,
        51 => Num3,
        52 => Num4,
        53 => Num5,
        54 => Num6,
        55 => Num7,
        56 => Num8,
        57 => Num9,
        48 => Num0,
        189 => Minus,
        187 => Equal,
        81 => KeyQ,
        87 => KeyW,
        69 => KeyE,
        82 => KeyR,
        84 => KeyT,
        89 => KeyY,
        85 => KeyU,
        73 => KeyI,
        79 => KeyO,
        80 => KeyP,
        219 => LeftBracket,
        221 => RightBracket,
        65 => KeyA,
        83 => KeyS,
        68 => KeyD,
        70 => KeyF,
        71 => KeyG,
        72 => KeyH,
        74 => KeyJ,
        75 => KeyK,
        76 => KeyL,
        186 => SemiColon,
        222 => Quote,
        220 => BackSlash,
        226 => IntlBackslash,
        90 => KeyZ,
        88 => KeyX,
        67 => KeyC,
        86 => KeyV,
        66 => KeyB,
        78 => KeyN,
        77 => KeyM,
        188 => Comma,
        190 => Dot,
        191 => Slash,
        45 => Insert,
        //KP_RETURN, 13,
        109 => KpMinus,
        107 => KpPlus,
        106 => KpMultiply,
        111 => KpDivide,
        96 => Kp0,
        97 => Kp1,
        98 => Kp2,
        99 => Kp3,
        100 => Kp4,
        101 => Kp5,
        102 => Kp6,
        103 => Kp7,
        104 => Kp8,
        105 => Kp9,
        110 => KpDelete,
        code => Unknown(code),
    }
}