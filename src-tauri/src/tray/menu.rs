use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    AppHandle,
};

use super::types::*;

pub fn load_tray_icon<'a>(app_handle: &'a AppHandle) -> Image<'a> {
    app_handle
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| {
            Image::from_bytes(include_bytes!("../../icons/32x32.png"))
                .expect("embedded 32x32 tray icon must be valid png")
        })
}

pub fn build_tray_menu(
    app_handle: &AppHandle,
) -> Result<(Menu<tauri::Wry>, MenuItem<tauri::Wry>, MenuItem<tauri::Wry>), Box<dyn std::error::Error>> {
    let status_item = MenuItem::with_id(
        app_handle,
        MENU_STATUS,
        "Status: Idle",
        false,
        None::<&str>,
    )?;

    let sep1 = PredefinedMenuItem::separator(app_handle)?;

    let toggle_item = MenuItem::with_id(
        app_handle,
        MENU_TOGGLE,
        "Start Recording",
        true,
        None::<&str>,
    )?;

    let show_main_item = MenuItem::with_id(
        app_handle,
        MENU_SHOW_MAIN,
        "Show Main Window",
        true,
        None::<&str>,
    )?;

    let show_widget_item = MenuItem::with_id(
        app_handle,
        MENU_SHOW_WIDGET,
        "Show Floating Widget",
        true,
        None::<&str>,
    )?;

    let settings_item = MenuItem::with_id(
        app_handle,
        MENU_SETTINGS,
        "Settings",
        true,
        None::<&str>,
    )?;

    let sep2 = PredefinedMenuItem::separator(app_handle)?;

    let exit_item = MenuItem::with_id(
        app_handle,
        MENU_EXIT,
        "Exit",
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app_handle,
        &[
            &status_item,
            &sep1,
            &toggle_item,
            &show_main_item,
            &show_widget_item,
            &settings_item,
            &sep2,
            &exit_item,
        ],
    )?;

    Ok((menu, status_item, toggle_item))
}