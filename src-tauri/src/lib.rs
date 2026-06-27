pub mod config;
pub mod keychain;
pub mod providers;
pub mod types;

use config::{load_widget_config, save_widget_config};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    ActivationPolicy, Emitter, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;
use types::{AccountsConfig, WidgetConfig, WidgetSnapshot};

const TRAY_ID: &str = "main-tray";
const MENU_SHOW: &str = "tray_show_panel";
const MENU_HIDE: &str = "tray_hide_panel";
const MENU_REFRESH: &str = "tray_refresh";
const MENU_QUIT: &str = "tray_quit";

fn show_main_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn hide_main_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn toggle_main_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(true) {
            hide_main_panel(app);
        } else {
            show_main_panel(app);
        }
    }
}

fn apply_window_prefs(app: &tauri::AppHandle) {
    let config = load_widget_config();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(config.always_on_top);
        let _ = window.set_visible_on_all_workspaces(true);

        let has_saved_position = config.window_x.is_some() && config.window_y.is_some();

        if let (Some(x), Some(y)) = (config.window_x, config.window_y) {
            if x > -100.0 && y > -100.0 && x < 10000.0 && y < 10000.0 {
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: x as i32,
                    y: y as i32,
                }));
            }
        } else if !has_saved_position {
            let _ = window.center();
        }

        if let (Some(w), Some(h)) = (config.window_width, config.window_height) {
            if w >= 260.0 && h >= 200.0 {
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width: w as u32,
                    height: h as u32,
                }));
            }
        }
    }
}

#[tauri::command]
async fn fetch_usage() -> Result<WidgetSnapshot, String> {
    Ok(providers::fetch_all_usage().await)
}

#[tauri::command]
fn get_widget_config() -> WidgetConfig {
    load_widget_config()
}

#[tauri::command]
fn save_widget_config_cmd(config: WidgetConfig) -> Result<(), String> {
    save_widget_config(&config)
}

#[tauri::command]
fn get_accounts_config() -> AccountsConfig {
    config::load_accounts_config()
}

#[tauri::command]
fn save_accounts_config_cmd(config: AccountsConfig) -> Result<(), String> {
    config::save_accounts_config(&config)
}

#[tauri::command]
async fn set_always_on_top(window: tauri::Window, always_on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_launch_at_login(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    let mut config = load_widget_config();
    config.launch_at_login = enabled;
    save_widget_config(&config)
}

#[tauri::command]
async fn save_window_position(window: tauri::Window) -> Result<(), String> {
    let mut config = load_widget_config();
    if let Ok(pos) = window.outer_position() {
        config.window_x = Some(pos.x as f64);
        config.window_y = Some(pos.y as f64);
    }
    if let Ok(size) = window.outer_size() {
        config.window_width = Some(size.width as f64);
        config.window_height = Some(size.height as f64);
    }
    save_widget_config(&config)
}

#[tauri::command]
fn hide_panel(app: tauri::AppHandle) -> Result<(), String> {
    hide_main_panel(&app);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            apply_window_prefs(app.handle());
            show_main_panel(app.handle());

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("tray icon bytes");

            let tray_builder = TrayIconBuilder::with_id(TRAY_ID)
                .icon(tray_icon)
                .tooltip("AI Usage")
                .icon_as_template(true)
                .show_menu_on_left_click(false);

            let show_item = MenuItem::with_id(app, MENU_SHOW, "Show Panel", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, MENU_HIDE, "Hide Panel", true, None::<&str>)?;
            let refresh_item =
                MenuItem::with_id(app, MENU_REFRESH, "Refresh Now", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&show_item, &hide_item, &refresh_item, &quit_item],
            )?;

            let app_handle = app.handle().clone();
            tray_builder
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    MENU_SHOW => show_main_panel(app),
                    MENU_HIDE => hide_main_panel(app),
                    MENU_REFRESH => {
                        let _ = app.emit("refresh-usage", ());
                    }
                    MENU_QUIT => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_panel(&app_handle);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            fetch_usage,
            get_widget_config,
            save_widget_config_cmd,
            get_accounts_config,
            save_accounts_config_cmd,
            set_always_on_top,
            set_launch_at_login,
            save_window_position,
            hide_panel,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Reopen { .. } = event {
                show_main_panel(app_handle);
            }
        });
}
