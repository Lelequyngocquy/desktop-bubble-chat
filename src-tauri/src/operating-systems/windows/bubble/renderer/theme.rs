#[derive(Debug, Clone, Copy)]
pub struct AccountTheme {
    pub primary: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
}

/// Retrieves color palette for account by index (1 -> 5).
pub fn get_account_theme(index: u32) -> AccountTheme {
    match ((index.max(1) - 1) % 5) + 1 {
        1 => AccountTheme {
            primary: (6, 182, 212),     // #06B6D4
            secondary: (8, 145, 178),   // #0891B2
        },
        2 => AccountTheme {
            primary: (236, 72, 153),    // #EC4899
            secondary: (219, 39, 119),  // #DB2777
        },
        3 => AccountTheme {
            primary: (249, 115, 22),    // #F97316
            secondary: (234, 88, 12),   // #EA580C
        },
        4 => AccountTheme {
            primary: (139, 92, 246),    // #8B5CF6
            secondary: (124, 58, 237),  // #7C3AED
        },
        _ => AccountTheme {
            primary: (16, 185, 129),    // #10B981
            secondary: (5, 150, 105),   // #059669
        },
    }
}
