#![windows_subsystem = "windows"]

mod win_icon;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use eframe::egui;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon as TrayIconImage, TrayIcon, TrayIconBuilder};

/// Répertoire contenant l’exécutable (racine de déploiement).
fn exe_root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .and_then(|p| std::fs::canonicalize(&p).ok().or(Some(p)))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

/// Sous-dossiers directs à la racine de l’exécutable (tri par nom), exclus les dossiers cachés (`.*`).
fn discover_zones(exe_root: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(exe_root) else {
        return Vec::new();
    };

    let mut dirs: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| !s.starts_with('.'))
                .unwrap_or(true)
        })
        .collect();

    dirs.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(&b.file_name().unwrap_or_default())
    });

    for p in &mut dirs {
        if let Ok(c) = std::fs::canonicalize(&p) {
            *p = c;
        }
    }

    dirs
}

struct Entry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

struct ZoneApp {
    /// Dossiers à parcourir (sous-dossiers de la racine exe), ou la racine seule si aucun.
    zones: Vec<PathBuf>,
    zone_index: usize,
    folder: PathBuf,
    entries: Vec<Entry>,
    textures: HashMap<PathBuf, egui::TextureHandle>,
    /// Taille d’affichage des icônes (px).
    icon_display_px: f32,
    failed_icons: HashSet<PathBuf>,
    /// Filtre de recherche (nom de fichier/dossier).
    query: String,
    /// Configuration globale (pour l’opacité, etc.).
    config: AppConfig,
    /// Taille tex (px) utilisée pour les `textures`; permet d'invalider si le slider change.
    last_tex_sz: i32,
    /// Indique s'il s'agit de la première frame (pour forcer la position initiale).
    first_frame: bool,
}

#[derive(Clone)]
struct AppConfig {
    window_width: f32,
    window_height: f32,
    window_min_width: f32,
    window_min_height: f32,
    transparent: bool,
    decorations: bool,
    icon_size_px: f32,
    /// Opacité globale de l’UI (0–100).
    opacity_percent: u8,
    /// Couleur de fond (RGB).
    bg_rgb: [u8; 3],
    /// Rayon des coins (px).
    corner_radius: f32,
    /// Position X de la fenêtre (px).
    window_pos_x: Option<f32>,
    /// Position Y de la fenêtre (px).
    window_pos_y: Option<f32>,
}

impl AppConfig {
    fn parse_rgb(s: &str) -> Option<[u8; 3]> {
        let t = s.trim();
        if let Some(hex) = t.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some([r, g, b]);
            }
        }

        // fallback "R,G,B"
        let parts: Vec<_> = t.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some([r, g, b]);
        }
        None
    }

    fn load() -> Self {
        let default = Self {
            window_width: 440.0,
            window_height: 560.0,
            window_min_width: 320.0,
            window_min_height: 280.0,
            transparent: true,
            decorations: false,
            icon_size_px: 118.0,
            opacity_percent: 40,
            bg_rgb: [14, 16, 24],
            corner_radius: 10.0,
            window_pos_x: None,
            window_pos_y: None,
        };

        let mut out = default.clone();

        let cfg_path = exe_root_dir().join("forma_dock.ini");
        let Ok(text) = std::fs::read_to_string(&cfg_path) else {
            return out;
        };

        let mut section = String::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') && line.len() > 2 {
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }
            let Some(eq) = line.find('=') else {
                continue;
            };
            let key = line[..eq].trim();
            let val = line[eq + 1..].trim();

            match (section.as_str(), key) {
                ("window", "width") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_width = v.max(200.0);
                    }
                }
                ("window", "height") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_height = v.max(200.0);
                    }
                }
                ("window", "min_width") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_min_width = v.max(160.0);
                    }
                }
                ("window", "min_height") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_min_height = v.max(160.0);
                    }
                }
                ("window", "transparent") => {
                    let v = val.to_ascii_lowercase();
                    out.transparent = matches!(v.as_str(), "1" | "true" | "yes" | "on" | "vrai");
                }
                ("window", "decorations") => {
                    let v = val.to_ascii_lowercase();
                    out.decorations = matches!(v.as_str(), "1" | "true" | "yes" | "on" | "vrai");
                }
                ("icons", "size_px") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.icon_size_px = v.clamp(32.0, 156.0);
                    }
                }
                ("window", "opacity") => {
                    if let Ok(v) = val.parse::<i32>() {
                        let clamped = v.clamp(0, 100) as u8;
                        out.opacity_percent = clamped;
                    }
                }
                ("window", "bg_color") | ("theme", "bg_color") => {
                    if let Some(rgb) = Self::parse_rgb(val) {
                        out.bg_rgb = rgb;
                    }
                }
                ("window", "corner_radius") | ("theme", "corner_radius") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.corner_radius = v.clamp(0.0, 40.0);
                    }
                }
                ("window", "x") | ("window", "pos_x") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_pos_x = Some(v);
                    }
                }
                ("window", "y") | ("window", "pos_y") => {
                    if let Ok(v) = val.parse::<f32>() {
                        out.window_pos_y = Some(v);
                    }
                }
                _ => {}
            }
        }

        out
    }

    fn opacity_factor(&self) -> f32 {
        // Conversion de pourcentage d'opacité en facteur de 0.0 à 1.0
        // Quand opacity_percent = 0, retourner un très petit facteur (pas zéro)
        // pour permettre la super-réduction au lieu de rendre invisible totalement
        if self.opacity_percent == 0 {
            0.01  // 1% : permet une transparence extrême avec contrôle
        } else {
            (self.opacity_percent as f32 / 100.0).clamp(0.0, 1.0)
        }
    }

    /// Facteur de réduction supplémentaire quand opacity = 0 pour extreme transparency
    fn zero_opacity_reduction(&self) -> f32 {
        if self.opacity_percent == 0 {
            0.08  // Ultra-transparent: réduit drastiquement (8% de l'alpha)
        } else {
            1.0   // Normal: pas de réduction supplémentaire
        }
    }
}

impl ZoneApp {
    fn new(config: &AppConfig) -> Self {
        let exe_root = exe_root_dir();
        let mut zones = discover_zones(&exe_root);
        if zones.is_empty() {
            zones.push(exe_root);
        }

        let folder = zones[0].clone();
        let icon_display_px = config.icon_size_px;

        let mut s = Self {
            zones,
            zone_index: 0,
            folder,
            entries: Vec::new(),
            textures: HashMap::new(),
            icon_display_px,
            failed_icons: HashSet::new(),
            query: String::new(),
            config: config.clone(),
            last_tex_sz: 0,
            first_frame: true,
        };
        s.rescan();
        s
    }

    fn go_prev(&mut self) {
        if self.zones.len() <= 1 {
            return;
        }
        self.zone_index = (self.zone_index + self.zones.len() - 1) % self.zones.len();
        self.folder = self.zones[self.zone_index].clone();
        self.rescan();
    }

    fn go_next(&mut self) {
        if self.zones.len() <= 1 {
            return;
        }
        self.zone_index = (self.zone_index + 1) % self.zones.len();
        self.folder = self.zones[self.zone_index].clone();
        self.rescan();
    }

    fn rescan(&mut self) {
        self.entries.clear();
        self.textures.clear();
        self.failed_icons.clear();

        if let Ok(c) = std::fs::canonicalize(&self.folder) {
            self.folder = c;
        }

        let Ok(rd) = std::fs::read_dir(&self.folder) else {
            return;
        };

        let mut items: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        items.sort_by(|a, b| {
            let da = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let db = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            match (da, db) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for e in items {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            self.entries.push(Entry { path, name, is_dir });
        }
    }

    fn title_label(&self) -> String {
        self.folder
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.folder.display().to_string())
    }

    fn load_pending_icons(&mut self, ctx: &egui::Context) {
        // Texture plus grande que l’affichage pour un rendu plus net une fois réduit.
        let tex_sz = (self.icon_display_px * 4.0 / 3.0)
            .round()
            .clamp(64.0, 128.0) as i32;

        // Si la taille d’icône a changé, on invalide l’ancien cache.
        if self.last_tex_sz != 0 && self.last_tex_sz != tex_sz {
            self.textures.clear();
            self.failed_icons.clear();
        }
        self.last_tex_sz = tex_sz;

        // Limite de temps par frame pour éviter les freeze UI.
        let time_budget = std::time::Duration::from_millis(8);
        let start = std::time::Instant::now();

        let q = self.query.trim().to_lowercase();
        let filter_active = !q.is_empty();

        let mut loads = 0;
        for e in &self.entries {
            let p = &e.path;
            if self.textures.contains_key(p) || self.failed_icons.contains(p) {
                continue;
            }
            if filter_active && !e.name.to_lowercase().contains(&q) {
                continue;
            }
            let Some(img) = win_icon::icon_for_path(p, tex_sz) else {
                self.failed_icons.insert(p.clone());
                continue;
            };
            let tex = ctx.load_texture(
                format!("icn_{}", p.display()),
                img,
                egui::TextureOptions::LINEAR,
            );
            self.textures.insert(p.clone(), tex);
            loads += 1;
            if loads >= 128 || start.elapsed() > time_budget {
                break;
            }
        }
    }
}

impl eframe::App for ZoneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            if let (Some(x), Some(y)) = (self.config.window_pos_x, self.config.window_pos_y) {
                // Application de la position de la fenêtre via commande au premier rendu
                // (contourne les bugs fréquents de fenêtres sans bordures sous winit/Windows)
                let scale = ctx.pixels_per_point();
                // Si l'utilisateur pense en pixels physiques (comme stipulé dans forma_dock.ini),
                // on les convertit en points logiques pour egui.
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    x / scale,
                    y / scale,
                )));
            }
        }

        let opacity = self.config.opacity_factor();
        let zero_reduction = self.config.zero_opacity_reduction();
        // Mise à l'échelle de l'alpha avec réduction spéciale si opacity = 0
        let scale_alpha = |a: u8| -> u8 { ((a as f32) * opacity * zero_reduction).round().clamp(0.0, 255.0) as u8 };
        let radius = self.config.corner_radius;

        // Visuel global (thème sombre + coins arrondis) — appliqué à chaque frame pour suivre
        // d’éventuels changements (DPI, etc.) sans gérer d’état global ailleurs.
        ctx.set_visuals({
            let mut v = egui::Visuals::dark();
            v.panel_fill = egui::Color32::TRANSPARENT;
            v.window_fill = egui::Color32::TRANSPARENT;
            // Fond des widgets avec transparence maximale (très léger, presque invisible)
            v.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(27, 27, 27, scale_alpha(10));
            v.window_rounding = egui::Rounding::same(radius);
            v.menu_rounding = egui::Rounding::same(radius.max(4.0) - 2.0);
            v.widgets.noninteractive.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
            v.widgets.inactive.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
            v.widgets.hovered.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
            v.widgets.active.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
            v.widgets.open.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
            // Sélection avec transparence modérée pour rester visible
            v.selection.bg_fill =
                egui::Color32::from_rgba_unmultiplied(88, 120, 255, scale_alpha(40));
            // Fonds faibles : extrêmement transparents pour l'effet vitre dépoli
            v.faint_bg_color = egui::Color32::from_rgba_unmultiplied(24, 28, 38, scale_alpha(15));
            v.extreme_bg_color = egui::Color32::from_rgba_unmultiplied(10, 12, 18, scale_alpha(10));
            // Ombre de fenêtre très légère (presque imperceptible avec transparence)
            v.window_shadow = egui::Shadow {
                offset: egui::vec2(0.0, 14.0),
                blur: 40.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(scale_alpha(10)),
            };
            v
        });

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.go_prev();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.go_next();
        }

        // Intercepter le zoom (Ctrl + Molette ou trackpad pinch) pour la taille des icônes
        let mut zoom_delta = 1.0;
        ctx.input_mut(|i| {
            // Enlever les événements de zoom pour que l'interface ne soit pas redimensionnée
            let mut retained = Vec::new();
            for event in i.events.drain(..) {
                if let egui::Event::Zoom(z) = event {
                    zoom_delta *= z;
                } else {
                    retained.push(event);
                }
            }
            i.events = retained;
        });

        if zoom_delta != 1.0 {
            self.icon_display_px = (self.icon_display_px * zoom_delta).clamp(32.0, 512.0);
        }

        self.load_pending_icons(ctx);

        let zone_title = self.title_label();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{zone_title} — Forma Dock"
        )));

        let n_zones = self.zones.len();
        let can_nav = n_zones > 1;
        let idx = self.zone_index + 1;

        // Fond "verre" + léger dégradé (surtout visible avec la fenêtre transparente).
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter();

                let brighten = |rgb: [u8; 3], mul: f32| -> [u8; 3] {
                    [
                        ((rgb[0] as f32) * mul).round().clamp(0.0, 255.0) as u8,
                        ((rgb[1] as f32) * mul).round().clamp(0.0, 255.0) as u8,
                        ((rgb[2] as f32) * mul).round().clamp(0.0, 255.0) as u8,
                    ]
                };

                let base = self.config.bg_rgb;
                let top_rgb = brighten(base, 0.95);
                let mid_rgb = brighten(base, 1.05);
                let bot_rgb = brighten(base, 0.85);

                let top = egui::Color32::from_rgba_unmultiplied(
                    top_rgb[0],
                    top_rgb[1],
                    top_rgb[2],
                    scale_alpha(30),
                );
                let mid = egui::Color32::from_rgba_unmultiplied(
                    mid_rgb[0],
                    mid_rgb[1],
                    mid_rgb[2],
                    scale_alpha(25),
                );
                let bot = egui::Color32::from_rgba_unmultiplied(
                    bot_rgb[0],
                    bot_rgb[1],
                    bot_rgb[2],
                    scale_alpha(20),
                );

                // Trois bandes verticales pour simuler un dégradé (simple et stable).
                let h = rect.height().max(1.0);
                let r1 = egui::Rect::from_min_max(
                    rect.min,
                    egui::pos2(rect.max.x, rect.min.y + h * 0.32),
                );
                let r2 = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.min.y + h * 0.32),
                    egui::pos2(rect.max.x, rect.min.y + h * 0.72),
                );
                let r3 = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.min.y + h * 0.72),
                    rect.max,
                );

                painter.rect_filled(r1, 14.0, top);
                painter.rect_filled(r2, 14.0, mid);
                painter.rect_filled(r3, 14.0, bot);
            });

        let frame_fill = egui::Color32::TRANSPARENT;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(frame_fill)
                    .rounding(egui::Rounding::same(radius))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(scale_alpha(5))))
                    .inner_margin(egui::Margin {
                        left: 12.0,
                        right: 0.0,
                        top: 12.0,
                        bottom: 12.0,
                    }),
            )
            .show(ctx, |ui| {
                let header_h = 46.0;
                let (r, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), header_h),
                    egui::Sense::click_and_drag(),
                );
                if ui
                    .interact(r, ui.id().with("drag_strip"), egui::Sense::drag())
                    .drag_started_by(egui::PointerButton::Primary)
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(r), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        let prev = ui
                            .add_enabled(
                                can_nav,
                                egui::Button::new(egui::RichText::new("◀").size(18.0)),
                            )
                            .on_hover_text(if can_nav {
                                "Lecture précédente (←)"
                            } else {
                                "Un seul dossier à la racine de l’exécutable"
                            });
                        if prev.clicked() {
                            self.go_prev();
                        }

                        ui.label(
                            egui::RichText::new(format!("{zone_title}"))
                                .strong()
                                .size(17.0)
                                .color(egui::Color32::from_gray(240)),
                        );

                        if can_nav {
                            ui.label(
                                egui::RichText::new(format!("({idx}/{n_zones})"))
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(160)),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0); // Marge/Padding pour le bouton le plus à droite
                            let next = ui
                                .add_enabled(
                                    can_nav,
                                    egui::Button::new(egui::RichText::new("▶").size(18.0)),
                                )
                                .on_hover_text(if can_nav {
                                    "Lecture suivante (→)"
                                } else {
                                    "Un seul dossier à la racine de l’exécutable"
                                });
                            if next.clicked() {
                                self.go_next();
                            }

                            // Taille d’icône (juste à gauche du bouton suivant)
                            ui.add(
                                egui::Slider::new(&mut self.icon_display_px, 32.0..=512.0)
                                    .show_value(false)
                                    .text(""),
                            )
                            .on_hover_text(format!(
                                "Taille des icônes : {}px",
                                self.icon_display_px.round() as i32
                            ));

                            ui.add_space(10.0);

                            // Recherche (filtre instantané) juste à gauche du slider.
                            let search = ui.add(
                                egui::TextEdit::singleline(&mut self.query)
                                    .hint_text("Rechercher…")
                                    .desired_width(50.0),
                            );
                            if search.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                                self.query.clear();
                                ctx.memory_mut(|m| m.surrender_focus(search.id));
                            }
                            if search.hovered() {
                                search.on_hover_text("Échap pour effacer et quitter le champ");
                            }
                        });
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(8.0);

                let icon = self.icon_display_px;
                let cell_w = icon + 8.0;

                // Grille explicite : une ligne horizontale par rangée (egui::Grid seule ne passe pas
                // automatiquement à la ligne sans `end_row()` après chaque rangée).
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        let q = self.query.trim().to_lowercase();
                        let filtered: Vec<&Entry> = if q.is_empty() {
                            self.entries.iter().collect()
                        } else {
                            self.entries
                                .iter()
                                .filter(|e| e.name.to_lowercase().contains(&q))
                                .collect()
                        };

                        let avail = ui.available_width();
                        let gap_x = 12.0;
                        let gap_y = 26.0;
                        let cols = ((avail + gap_x * 0.5) / (cell_w + gap_x)).floor() as usize;
                        let cols = cols.max(1);

                        if filtered.is_empty() {
                            ui.add_space(36.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Aucun résultat")
                                        .size(18.0)
                                        .strong()
                                        .color(egui::Color32::from_gray(230)),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(
                                        "Essayez un autre mot-clé, ou Échap pour effacer.",
                                    )
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(170)),
                                );
                            });
                            return;
                        }

                        for row in filtered.chunks(cols) {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(gap_x, gap_y);
                                for e in row {
                                    let tid = self.textures.get(&e.path).map(|t| t.id());
                                    let loading =
                                        tid.is_none() && !self.failed_icons.contains(&e.path);
                                    let failed = self.failed_icons.contains(&e.path);

                                    ui.allocate_ui_with_layout(
                                        egui::vec2(cell_w, 0.0),
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.spacing_mut().item_spacing.y = 2.0;

                                            let tip = if e.is_dir {
                                                format!("Dossier — {}\nClic pour ouvrir", e.name)
                                            } else {
                                                format!("{}\nClic pour ouvrir", e.name)
                                            };

                                            let (img_rect, img_resp) = ui.allocate_exact_size(
                                                egui::vec2(icon, icon),
                                                egui::Sense::click(),
                                            );
                                            let img_resp = img_resp.on_hover_text(&tip);
                                            if img_resp.clicked() {
                                                let _ = open::that(&e.path);
                                            }

                                            if let Some(tid) = tid {
                                                let uv = egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                );
                                                ui.painter().image(
                                                    tid,
                                                    img_rect,
                                                    uv,
                                                    egui::Color32::WHITE,
                                                );
                                            } else if loading {
                                                ui.painter().rect_filled(
                                                    img_rect,
                                                    4.0,
                                                    egui::Color32::from_rgba_unmultiplied(
                                                        20, 22, 30, 20,
                                                    ),
                                                );
                                                ui.allocate_new_ui(
                                                    egui::UiBuilder::new()
                                                        .max_rect(img_rect)
                                                        .layout(
                                                            egui::Layout::centered_and_justified(
                                                                egui::Direction::TopDown,
                                                            ),
                                                        ),
                                                    |ui| {
                                                        ui.spinner();
                                                    },
                                                );
                                            } else if failed {
                                                let fallback =
                                                    if e.is_dir { "📁" } else { "📄" };
                                                ui.painter().text(
                                                    img_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    fallback,
                                                    egui::FontId::proportional(icon * 0.55),
                                                    egui::Color32::WHITE,
                                                );
                                            }

                                            if !e.is_dir {
                                                let ext = e
                                                    .path
                                                    .extension()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("")
                                                    .to_lowercase();
                                                if ext == "url" || ext == "lnk" {
                                                    let overlay_size = 20.0;
                                                    let overlay_rect = egui::Rect::from_min_size(
                                                        egui::pos2(
                                                            img_rect.min.x,
                                                            img_rect.max.y - overlay_size,
                                                        ),
                                                        egui::vec2(overlay_size, overlay_size),
                                                    );
                                                    ui.painter().rect_filled(
                                                        overlay_rect,
                                                        0.0,
                                                        egui::Color32::WHITE,
                                                    );
                                                    ui.painter().text(
                                                        overlay_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        "↗",
                                                        egui::FontId::proportional(16.0),
                                                        egui::Color32::from_rgb(0, 120, 215),
                                                    );
                                                }
                                            }

                                            // Retrait de l'extension du nom pour l'affichage
                                            let mut display_name = e.name.clone();
                                            if e.name.to_lowercase().ends_with(".url")
                                                || e.name.to_lowercase().ends_with(".lnk")
                                            {
                                                if let Some(idx) = display_name.rfind('.') {
                                                    display_name.truncate(idx);
                                                }
                                            }

                                            let name_rt = if e.is_dir {
                                                egui::RichText::new(&display_name)
                                                    .size(12.0)
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(232, 236, 255))
                                            } else {
                                                egui::RichText::new(&display_name)
                                                    .size(12.0)
                                                    .color(egui::Color32::WHITE)
                                            };
                                            ui.add(
                                                egui::Label::new(name_rt)
                                                    .wrap()
                                                    .halign(egui::Align::Center),
                                            );
                                        },
                                    );
                                }
                            });
                            ui.add_space(gap_y);
                        }
                    });
            });
    }
}

fn main() -> eframe::Result<()> {
    let config = AppConfig::load();

    // Icône de zone de notification (simple carré lumineux).
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
                .with_tooltip("Forma Dock")
                .with_icon(icon)
                .with_menu(Box::new(tray_menu))
                .build()
                .ok()
        } else {
            None
        }
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Forma Dock — zone dossier")
        .with_inner_size([config.window_width, config.window_height])
        .with_min_inner_size([config.window_min_width, config.window_min_height])
        .with_transparent(config.transparent)
        .with_decorations(config.decorations)
        .with_taskbar(false)
        .with_window_level(egui::WindowLevel::AlwaysOnBottom);

    if let (Some(x), Some(y)) = (config.window_pos_x, config.window_pos_y) {
        viewport = viewport.with_position([x, y]);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app_config = config.clone();

    let _tray_guard = tray_icon;

    eframe::run_native(
        "Forma Dock",
        options,
        Box::new(move |_cc| Ok(Box::new(ZoneApp::new(&app_config)) as Box<dyn eframe::App>)),
    )
}
