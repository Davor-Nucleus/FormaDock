mod win_icon;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use eframe::egui;

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
}

impl ZoneApp {
    fn new() -> Self {
        let exe_root = exe_root_dir();
        let mut zones = discover_zones(&exe_root);
        if zones.is_empty() {
            zones.push(exe_root);
        }

        let folder = zones[0].clone();
        let mut s = Self {
            zones,
            zone_index: 0,
            folder,
            entries: Vec::new(),
            textures: HashMap::new(),
            icon_display_px: 118.0,
            failed_icons: HashSet::new(),
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
            self.entries.push(Entry {
                path,
                name,
                is_dir,
            });
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
        let mut loads = 0;
        for e in &self.entries {
            let p = &e.path;
            if self.textures.contains_key(p) || self.failed_icons.contains(p) {
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
            if loads >= 128 {
                break;
            }
        }
    }
}

impl eframe::App for ZoneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.go_prev();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.go_next();
        }

        self.load_pending_icons(ctx);

        let zone_title = self.title_label();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{zone_title} — frence")));

        let frame_fill = egui::Color32::from_rgba_unmultiplied(18, 20, 28, 200);
        let n_zones = self.zones.len();
        let can_nav = n_zones > 1;
        let idx = self.zone_index + 1;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(frame_fill)
                    .rounding(egui::Rounding::same(10.0))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_white_alpha(40),
                    ))
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                let header_h = 40.0;
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
                    ui.horizontal_centered(|ui| {
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
                            egui::RichText::new(format!("Zone « {zone_title} »"))
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
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(8.0);

                let icon = self.icon_display_px;
                let cell_w = icon;

                let tex_ids: HashMap<PathBuf, egui::TextureId> = self
                    .textures
                    .iter()
                    .map(|(k, v)| (k.clone(), v.id()))
                    .collect();

                // Grille explicite : une ligne horizontale par rangée (egui::Grid seule ne passe pas
                // automatiquement à la ligne sans `end_row()` après chaque rangée).
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let avail = ui.available_width();
                    let gap_x = 18.0;
                    let gap_y = 26.0;
                    let cols = ((avail + gap_x) / (cell_w + gap_x)).floor() as usize;
                    let cols = cols.max(1);

                    for row in self.entries.chunks(cols) {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(gap_x, gap_y);
                            for e in row {
                                let tid = tex_ids.get(&e.path).copied();
                                let loading =
                                    tid.is_none() && !self.failed_icons.contains(&e.path);
                                let failed = self.failed_icons.contains(&e.path);

                                ui.allocate_ui_with_layout(
                                    egui::vec2(cell_w, 0.0),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
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
                                            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
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
                                                egui::Color32::from_rgba_unmultiplied(20, 22, 30, 255),
                                            );
                                            ui.allocate_new_ui(
                                                egui::UiBuilder::new()
                                                    .max_rect(img_rect)
                                                    .layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
                                                |ui| { ui.spinner(); }
                                            );
                                        } else if failed {
                                            let fallback = if e.is_dir { "📁" } else { "📄" };
                                            ui.painter().text(
                                                img_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                fallback,
                                                egui::FontId::proportional(icon * 0.55),
                                                egui::Color32::WHITE,
                                            );
                                        }

                                        if !e.is_dir {
                                            let ext = e.path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                                            if ext == "url" || ext == "lnk" {
                                                let overlay_size = 20.0;
                                                let overlay_rect = egui::Rect::from_min_size(
                                                    egui::pos2(img_rect.min.x, img_rect.max.y - overlay_size),
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

                                        ui.add_space(6.0);

                                        // Retrait de l'extension du nom pour l'affichage
                                        let mut display_name = e.name.clone();
                                        if e.name.to_lowercase().ends_with(".url") || e.name.to_lowercase().ends_with(".lnk") {
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
                                                .halign(egui::Align::Center)
                                        );
                                    }
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("frence — zone dossier")
            .with_inner_size([440.0, 560.0])
            .with_min_inner_size([320.0, 280.0])
            .with_transparent(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "frence",
        options,
        Box::new(|_cc| Ok(Box::new(ZoneApp::new()) as Box<dyn eframe::App>)),
    )
}
