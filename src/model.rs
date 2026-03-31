use std::path::{Path, PathBuf};
use eframe::egui;

/// Répertoire contenant l’exécutable (racine de déploiement).
/// Sert principalement à trouver `frence.ini` ou les applications.
pub fn exe_root_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .and_then(|p| std::fs::canonicalize(&p).ok().or(Some(p)))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

/// Représente un fichier ou un dossier physique listé dans l'interface (Vue).
pub struct Entry {
    /// Chemin absolu du fichier ou dossier.
    pub path: PathBuf,
    /// Nom affichable du fichier ou dossier.
    pub name: String,
    /// Indique si cette entrée est un répertoire (`true`) ou un fichier classique (`false`).
    pub is_dir: bool,
}

/// Requête asynchrone transmise par le Thread Principal (UI) à un Worker d'arrière-plan.
pub struct IconRequest {
    /// Chemin complet de l'élément dont il faut générer la miniature.
    pub path: PathBuf,
    /// La taille requise (carrée) en pixels pour ce rendu.
    pub size_px: i32,
    /// Identifiant de validité (Génération) pour éviter le traitement si la page a changé.
    pub generation: u32,
}

/// Réponse asynchrone transmise par un Worker d'arrière-plan au Thread Principal (UI).
pub struct IconResponse {
    /// Chemin complet de l'élément ayant fait l'objet de la demande.
    pub path: PathBuf,
    /// L'image brute correspondante extraite du système, si elle a été trouvée.
    pub image: Option<egui::ColorImage>,
    /// Identifiant de validité (Génération). Si la page UI a tourné, l'application jettera ce résultat.
    pub generation: u32,
}

/// Configuration globale du lanceur "frence".
/// Regroupe le thème de la fenêtre, la taille, et les marges définies pas l'utilisateur dans `frence.ini`.
#[derive(Clone)]
pub struct AppConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub window_min_width: f32,
    pub window_min_height: f32,
    pub transparent: bool,
    pub decorations: bool,
    pub icon_size_px: f32,
    /// Opacité globale de l’UI (0 à 100).
    pub opacity_percent: u8,
    /// Couleur de fond (RGB).
    pub bg_rgb: [u8; 3],
    /// Rayon des coins (px) de la fenêtre native.
    pub corner_radius: f32,
    /// Position X de la fenêtre (px, Optionnel).
    pub window_pos_x: Option<f32>,
    /// Position Y de la fenêtre (px, Optionnel).
    pub window_pos_y: Option<f32>,
}

impl AppConfig {
    /// Parse manuellement une chaîne de caractères RVB Hexadécimale (#RRGGBB) ou Liste (R,G,B).
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

        // Cas de secours : texte "R,G,B"
        let parts: Vec<_> = t.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some([r, g, b]);
        }
        None
    }

    /// Extrait et charge le fichier `frence.ini` situé à côté de l'exécutable.
    /// Retourne la configuration par défaut en cas d'absence.
    pub fn load() -> Self {
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

        let cfg_path = exe_root_dir().join("frence.ini");
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

    /// Retourne l'opacité (Facteur 0.0 à 1.0) calculé depuis les pourcentages utilisateur.
    pub fn opacity_factor(&self) -> f32 {
        // Quand opacity_percent = 0, retourner un très petit facteur (pas zéro)
        // pour permettre la super-réduction au lieu de rendre invisible totalement
        if self.opacity_percent == 0 {
            0.01 
        } else {
            (self.opacity_percent as f32 / 100.0).clamp(0.0, 1.0)
        }
    }

    /// Facteur de réduction supplémentaire exponentiel quand l'opacité globale tombe à zéro.
    pub fn zero_opacity_reduction(&self) -> f32 {
        if self.opacity_percent == 0 {
            0.08  // Ultra-transparent: réduit drastiquement (8% de l'alpha)
        } else {
            1.0   // Normal: pas de réduction supplémentaire
        }
    }
}
