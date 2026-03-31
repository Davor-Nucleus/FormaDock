#![windows_subsystem = "windows"]

mod app;
mod model;
mod ui;
mod win_icon;

use eframe::egui;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon as TrayIconImage, TrayIcon, TrayIconBuilder};

use app::ZoneApp;
use model::AppConfig;

/// Point d'entrée principal de l'application.
/// Gère la création de la fenêtre native, de la boucle WGPU et de l'icône système cachée (Tray Icon).
fn main() -> eframe::Result<()> {
    // 1. Chargement du Modèle Statique
    let config = AppConfig::load();

    // 2. Création de l'icône de zone de notification (simple carré lumineux de base).
    let tray_icon: Option<TrayIcon> = {
        let w = 16;
        let h = 16;
        let mut rgba = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
                let (r, g, b, a) = if edge {
                    (20u8, 200u8, 255u8, 255u8)
                } else {
                    (10u8, 40u8, 60u8, 255u8)
                };
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }

        let quit_item = MenuItem::new("Quitter", true, None);
        let tray_menu = Menu::new();
        let _ = tray_menu.append(&quit_item);

        tray_icon::menu::MenuEvent::set_event_handler(Some(
            |_event: tray_icon::menu::MenuEvent| {
                std::process::exit(0);
            },
        ));

        if let Ok(icon) = TrayIconImage::from_rgba(rgba, w as u32, h as u32) {
            TrayIconBuilder::new()
                .with_tooltip("frence")
                .with_icon(icon)
                .with_menu(Box::new(tray_menu))
                .build()
                .ok()
        } else {
            None
        }
    };

    // 3. Configuration du Conteneur de la Vue Eframe
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("frence — zone dossier")
        .with_inner_size([config.window_width, config.window_height])
        .with_min_inner_size([config.window_min_width, config.window_min_height])
        .with_transparent(config.transparent)
        .with_decorations(config.decorations)
        .with_taskbar(false) // Cache l'appli de la barre des fenêtres
        .with_window_level(egui::WindowLevel::AlwaysOnBottom);

    if let (Some(x), Some(y)) = (config.window_pos_x, config.window_pos_y) {
        viewport = viewport.with_position([x, y]);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app_config = config.clone();

    // Garde l'icône vivante tout au long de la boucle
    let _tray_guard = tray_icon;

    // 4. Lancement du Point d'Attache de l'App
    eframe::run_native(
        "frence",
        options,
        Box::new(move |cc| {
            Ok(Box::new(ZoneApp::new(&app_config, cc.egui_ctx.clone())) as Box<dyn eframe::App>)
        }),
    )
}
