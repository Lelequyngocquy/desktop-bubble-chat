use tauri::WebviewWindow;

pub mod autostart;
pub mod cursor;
pub mod dialog;
pub mod memory;
pub mod sound;
pub mod theme;
pub mod tray;

pub use autostart::MacOSAutostartManager as AutostartManager;
pub use cursor::MacOSCursorTracker as CursorTracker;
pub use dialog::MacOSDialogManager as DialogManager;
pub use memory::MacOSMemoryOptimizer as MemoryOptimizer;
pub use sound::MacOSSoundPlayer as SoundPlayer;
pub use theme::MacOSThemeManager as ThemeManager;
pub use tray::MacOSMenuManager as MenuManager;

pub mod bubble;
pub use bubble::*;
