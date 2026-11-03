pub const MESSENGER_INJECT_SCRIPT: &str = concat!(
    "(function () {\n",
    "  const TARGET_MESSENGER_URL = \"https://www.messenger.com/\";\n\n",
    include_str!("scripts/modules/01_ipc_bridge.js"),
    "\n\n",
    include_str!("scripts/modules/02_notification_interceptor.js"),
    "\n\n",
    include_str!("scripts/modules/03_identity_auth.js"),
    "\n\n",
    include_str!("scripts/modules/04_navigation_guard.js"),
    "\n\n",
    include_str!("scripts/modules/05_offline_handler.js"),
    "\n\n",
    include_str!("scripts/modules/06_styles.js"),
    "\n\n",
    include_str!("scripts/modules/07_lifecycle.js"),
    "\n})();\n"
);
pub const MESSENGER_LOGIN_URL: &str = "https://www.messenger.com/";
pub const MESSENGER_BASE_URL: &str = "https://www.messenger.com/";
