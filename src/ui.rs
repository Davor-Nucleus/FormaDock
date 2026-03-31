use eframe::egui;

use crate::app::ZoneApp;

/// Gère le rendu visuel de chaque trame de l'application (View du MVC).
pub fn update_view(app: &mut ZoneApp, ctx: &egui::Context) {
    // 1. Positionnement de la fenêtre lors de la première initialisation (first frame focus)
    if app.first_frame {
        app.first_frame = false;
        if let (Some(x), Some(y)) = (app.config.window_pos_x, app.config.window_pos_y) {
            let scale = ctx.pixels_per_point();
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                x / scale,
                y / scale,
            )));
        }
    }

    // 2. Thème de Couleur & Marges D'interfaces (Opacité Acrylique, etc.)
    let opacity = app.config.opacity_factor();
    let zero_reduction = app.config.zero_opacity_reduction();
    let scale_alpha = |a: u8| -> u8 {
        ((a as f32) * opacity * zero_reduction)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    let radius = app.config.corner_radius;

    ctx.set_visuals({
        let mut v = egui::Visuals::dark();
        v.panel_fill = egui::Color32::TRANSPARENT;
        v.window_fill = egui::Color32::TRANSPARENT;
        v.widgets.noninteractive.bg_fill =
            egui::Color32::from_rgba_unmultiplied(27, 27, 27, scale_alpha(10));
        v.window_rounding = egui::Rounding::same(radius);
        v.menu_rounding = egui::Rounding::same(radius.max(4.0) - 2.0);
        v.widgets.noninteractive.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
        v.widgets.inactive.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
        v.widgets.hovered.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
        v.widgets.active.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
        v.widgets.open.rounding = egui::Rounding::same(radius.max(3.0) - 3.0);
        
        v.selection.bg_fill =
            egui::Color32::from_rgba_unmultiplied(88, 120, 255, scale_alpha(40));
            
        v.faint_bg_color =
            egui::Color32::from_rgba_unmultiplied(24, 28, 38, scale_alpha(15));
        v.extreme_bg_color =
            egui::Color32::from_rgba_unmultiplied(10, 12, 18, scale_alpha(10));
            
        v.window_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 14.0),
            blur: 40.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(scale_alpha(10)),
        };
        v
    });

    // 3. Entrées Claviers et Interactions Invisibles (Zoom Moteur / Touches Directes)
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
        app.go_prev();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
        app.go_next();
    }

    let mut zoom_delta = 1.0;
    ctx.input_mut(|i| {
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
        app.icon_display_px = (app.icon_display_px * zoom_delta).clamp(32.0, 512.0);
    }

    // 4. Interface Haut-Niveau 
    let zone_title = app.title_label();
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
        "{zone_title} — frence"
    )));

    // Dessin du fond d'écran acrylique à dégradé vertical
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

            let base = app.config.bg_rgb;
            let top_rgb = brighten(base, 0.95);
            let mid_rgb = brighten(base, 1.05);
            let bot_rgb = brighten(base, 0.85);

            let top = egui::Color32::from_rgba_unmultiplied(
                top_rgb[0], top_rgb[1], top_rgb[2], scale_alpha(30),
            );
            let mid = egui::Color32::from_rgba_unmultiplied(
                mid_rgb[0], mid_rgb[1], mid_rgb[2], scale_alpha(25),
            );
            let bot = egui::Color32::from_rgba_unmultiplied(
                bot_rgb[0], bot_rgb[1], bot_rgb[2], scale_alpha(20),
            );

            let h = rect.height().max(1.0);
            let r1 = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.min.y + h * 0.32));
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

    // 5. Contenu Interféré Interactif
    let n_zones = app.zones.len();
    let can_nav = n_zones > 1;
    let idx = app.zone_index + 1;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::TRANSPARENT)
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
            // Dessin du Navigateur ("Header")
            let header_h = 46.0;
            let (r, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), header_h),
                egui::Sense::click_and_drag(),
            );
            
            if ui
                .interact(r, ui.id().with("drag_strip"), egui::Sense::drag())
                .drag_started_by(egui::PointerButton::Primary)
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag); // Déplace la fenêtre
            }
            
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(r), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    let prev = ui
                        .add_enabled(can_nav, egui::Button::new(egui::RichText::new("◀").size(18.0)))
                        .on_hover_text(if can_nav { "Lecture précédente (←)" } else { "1 dossier total" });
                        
                    if prev.clicked() {
                        app.go_prev();
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
                        ui.add_space(10.0);
                        let next = ui
                            .add_enabled(can_nav, egui::Button::new(egui::RichText::new("▶").size(18.0)))
                            .on_hover_text(if can_nav { "Lecture suivante (→)" } else { "1 dossier total" });
                            
                        if next.clicked() {
                            app.go_next();
                        }

                        ui.add(
                            egui::Slider::new(&mut app.icon_display_px, 32.0..=512.0)
                                .show_value(false)
                                .text(""),
                        )
                        .on_hover_text(format!("Taille des icônes : {}px", app.icon_display_px.round() as i32));

                        ui.add_space(10.0);
                        let search = ui.add(
                            egui::TextEdit::singleline(&mut app.query)
                                .hint_text("Rechercher…")
                                .desired_width(50.0),
                        );
                        if search.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            app.query.clear();
                            ctx.memory_mut(|m| m.surrender_focus(search.id));
                        }
                        if search.hovered() {
                            search.on_hover_text("Échap pour effacer");
                        }
                    });
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);

            // Important: On appelle les rechargements asynchrones d'images
            // UNIQUEMENT après avoir géré nos boutons Précédent/Suivant, pour éviter le Flush de "Memory Textures".
            app.load_pending_icons(ctx);

            let icon = app.icon_display_px;
            let cell_w = icon + 8.0;

            // 6. Dessin dynamique de la galerie de fichiers (Grille d'Icônes)
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    let q = app.query.trim().to_lowercase();
                    let filtered: Vec<&crate::model::Entry> = if q.is_empty() {
                        app.entries.iter().collect()
                    } else {
                        app.entries
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
                                egui::RichText::new("Essayez un autre mot-clé, ou Échap pour annuler.")
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
                                let tid = app.textures.get(&e.path).map(|t| t.id());
                                let loading = tid.is_none() && !app.failed_icons.contains(&e.path);
                                let failed = app.failed_icons.contains(&e.path);

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
                                            let _ = open::that(&e.path); // Ouvre le système de fichiers externe
                                        }

                                        // Application de l'image (texture) ou chargement paranoïaque 
                                        if let Some(tid) = tid {
                                            let uv = egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0),
                                            );
                                            ui.painter().image(tid, img_rect, uv, egui::Color32::WHITE);
                                        } else if loading {
                                            ui.painter().rect_filled(
                                                img_rect,
                                                4.0,
                                                egui::Color32::from_rgba_unmultiplied(20, 22, 30, 20),
                                            );
                                            ui.allocate_new_ui(
                                                egui::UiBuilder::new()
                                                    .max_rect(img_rect)
                                                    .layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
                                                |ui| { ui.spinner(); },
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

                                        // Ajout de badge / tag Raccourci pour LNK et URL
                                        if !e.is_dir {
                                            let ext = e.path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                                            if ext == "url" || ext == "lnk" {
                                                let overlay_size = 20.0;
                                                let overlay_rect = egui::Rect::from_min_size(
                                                    egui::pos2(img_rect.min.x, img_rect.max.y - overlay_size),
                                                    egui::vec2(overlay_size, overlay_size),
                                                );
                                                ui.painter().rect_filled(overlay_rect, 0.0, egui::Color32::WHITE);
                                                ui.painter().text(
                                                    overlay_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "↗",
                                                    egui::FontId::proportional(16.0),
                                                    egui::Color32::from_rgb(0, 120, 215),
                                                );
                                            }
                                        }

                                        // Raccourcissement de l'affichage du nom du fichier
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
                                        ui.add(egui::Label::new(name_rt).wrap().halign(egui::Align::Center));
                                    },
                                );
                            }
                        });
                        ui.add_space(gap_y);
                    }
                });
        });
}
