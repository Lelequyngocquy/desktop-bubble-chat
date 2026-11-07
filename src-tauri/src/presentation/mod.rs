<<<<<<<< HEAD:src-tauri/src/presentation/mod.rs
﻿pub mod bubble;
pub mod main_window;

========
pub use crate::operating_systems::current::bubble as native_bubble;
pub use crate::presentation::bubble;
pub use crate::presentation::main_window;
>>>>>>>> cc6d28e2c9bdc61f757d7002a28406128c99ab9a:src-tauri/src/windows/mod.rs
pub use bubble::*;
pub use main_window::*;
