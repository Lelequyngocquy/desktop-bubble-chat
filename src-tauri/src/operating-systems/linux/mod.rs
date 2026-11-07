use tauri::WebviewWindow;

pub mod autostart;
pub mod bubble;
pub mod cursor;
pub mod dialog;
pub mod memory;
pub mod sound;
pub mod theme;
pub mod tray;

pub use autostart::LinuxAutostartManager as AutostartManager;
pub use bubble::*;
pub use cursor::LinuxCursorTracker as CursorTracker;
pub use dialog::LinuxDialogManager as DialogManager;
pub use memory::LinuxMemoryOptimizer as MemoryOptimizer;
pub use sound::LinuxSoundPlayer as SoundPlayer;
pub use theme::LinuxThemeManager as ThemeManager;
pub use tray::LinuxMenuManager as MenuManager;
