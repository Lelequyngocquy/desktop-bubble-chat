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
    "\n\n",
    include_str!("scripts/modules/08_auto_unselect_thread.js"),
    "\n\n",
    include_str!("scripts/modules/09_pin_button.js"),
    "\n})();\n"
);
pub const MESSENGER_LOGIN_URL: &str = "https://www.messenger.com/";
pub const MESSENGER_BASE_URL: &str = "https://www.messenger.com/";

/// Optimized Chromium flags for WebView2 Messenger instances with hardware acceleration and expanded cache
pub const CHROMIUM_BROWSER_ARGS: &str = concat!(
    "--disable-features=Translate,OptimizationHints,MediaRouter,DialMediaRouteProvider,",
    "CalculateNativeWinOcclusion,InterestFeedContentSuggestions,AutofillServerCommunication,",
    "CertificateTransparencyComponentUpdater,msWebOOUI,msPdfOOUI,msSmartScreenProtection ",
    "--enable-features=CanvasOopRasterization,VaapiVideoDecoder ",
    "--enable-gpu-rasterization ",
    "--enable-zero-copy ",
    "--ignore-gpu-blocklist ",
    "--disable-sync ",
    "--disable-default-apps ",
    "--disable-spell-checking ",
    "--process-per-site ",
    "--disable-usb-keyboard-detect ",
    "--disable-gamepad ",
    "--disable-speech-api ",
    "--autoplay-policy=no-user-gesture-required ",
    "--disk-cache-size=268435456 ",
    "--media-cache-size=67108864"
);
