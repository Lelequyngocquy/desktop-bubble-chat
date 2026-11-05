use std::sync::Arc;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager};

use super::helpers::{get_available_account_indices, is_bubble_visible, is_main_window_open};
use super::TRAY_ID;
use crate::utils::i18n::get_tray_locale;

// Builds dynamic system tray menu using shared localization dictionary
pub fn build_tray_menu(app_handle: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app_handle)?;
    let locale_str = app_handle
        .try_state::<Arc<crate::state::ChatState>>()
        .and_then(|s| s.user_config.lock().ok().map(|c| c.current_locale().to_string()))
        .unwrap_or_else(|| "en".to_string());
    let tray_i18n = get_tray_locale(&locale_str);
    let is_open = is_main_window_open(app_handle);
    let home_title = if is_open {
        &tray_i18n.close_dashboard
    } else {
        &tray_i18n.open_dashboard
    };
    let show_home = MenuItem::with_id(app_handle, "show_home", home_title, true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app_handle)?;
    menu.append(&show_home)?;
    menu.append(&sep1)?;
    let available_accounts = get_available_account_indices(app_handle);
    if !available_accounts.is_empty() {
        let mut any_visible = false;
        for idx in &available_accounts {
            if is_bubble_visible(app_handle, *idx) {
                any_visible = true;
                break;
            }
        }
        let toggle_all_text = if any_visible {
            &tray_i18n.hide_all_bubbles
        } else {
            &tray_i18n.show_all_bubbles
        };
        let toggle_all_item = MenuItem::with_id(app_handle, "toggle_all_bubbles", toggle_all_text, true, None::<&str>)?;
        let sep2 = PredefinedMenuItem::separator(app_handle)?;
        menu.append(&toggle_all_item)?;
        menu.append(&sep2)?;
        for idx in &available_accounts {
            let is_visible = is_bubble_visible(app_handle, *idx);
            let default_label = tray_i18n.account_label.replace("{{index}}", &format!("{:02}", idx));
            let display_name = if let Some(chat_state) = app_handle.try_state::<Arc<crate::state::ChatState>>() {
                if let Some(auth) = chat_state.get_account_auth(*idx) {
                    let trimmed = auth.name.trim();
                    if !trimmed.is_empty() && trimmed != format!("Messenger {:02}", idx) {
                        format!("{} ({})", default_label, trimmed)
                    } else {
                        default_label
                    }
                } else {
                    default_label
                }
            } else {
                default_label
            };
            let item_id = format!("toggle_bubble_{}", idx);
            let check_item = CheckMenuItem::with_id(app_handle, &item_id, display_name, true, is_visible, None::<&str>)?;
            menu.append(&check_item)?;
        }
        let sep3 = PredefinedMenuItem::separator(app_handle)?;
        menu.append(&sep3)?;
    }
    let lang_label = if locale_str == "vi" {
        "English (lang)"
    } else {
        "Tiếng Việt"
    };
    let toggle_lang_item = MenuItem::with_id(app_handle, "toggle_language", lang_label, true, None::<&str>)?;
    let sep_lang = PredefinedMenuItem::separator(app_handle)?;
    menu.append(&toggle_lang_item)?;
    menu.append(&sep_lang)?;
    let quit = MenuItem::with_id(app_handle, "quit", &tray_i18n.quit_app, true, None::<&str>)?;
    menu.append(&quit)?;
    Ok(menu)
}

// Updates system tray menu with current state and language
pub fn update_tray_menu(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
        let menu = build_tray_menu(app_handle)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
