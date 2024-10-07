# caps-shortcut

将 CapsLock 作为修饰键的快捷键库。

该库仅是低限度、低抽象的实现。

仅支持 Windows。

## 例子

```rust
use caps_shortcut::Key;

fn main() {
   caps_shortcut::caps_with(Key::KeyU, || {
        println!("KeyU pressed");
   });

   caps_shortcut::caps_of_modifier_key_with(Key::KeyI, [Key::Alt, Key::ControlLeft], || {
        println!("KeyI with Alt + Control pressed");
   });

   caps_shortcut::caps_listener_with(|keyboard| {
        if keyboard.key == Key::KeyU
        && !keyboard.ctrl 
        && !keyboard.shift 
        && keyboard.alt 
        && !keyboard.meta 
        {
            println!("Cpas pressed and only KeyU with Alt pressed");
            true
        } else {
            false
        }
   });

   caps_shortcut::run();
}
```

## 注意

该库调整了 CapsLock 的使用逻辑。

按下 CapsLock 时，如果没有按下其他键，松开切换大小写锁定。

按下 CapsLock 时，如果按下了其他键，松开不会切换大小写锁定。