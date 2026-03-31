use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;

use crate::model::{exe_root_dir, AppConfig, Entry, IconRequest, IconResponse};
use crate::ui;
use crate::win_icon;

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

/// État global de l'application (Contrôleur).
/// Stocke les textures, les entrées actuelles du dossier, et gère la logique de navigation et d'arrière-plan.
pub struct ZoneApp {
    /// Dossiers à parcourir (sous-dossiers de la racine exe).
    pub zones: Vec<PathBuf>,
    /// Index du dossier actuellement scanné et affiché.
    pub zone_index: usize,
    /// Dossier physiquement ouvert.
    pub folder: PathBuf,
    /// Liste des entrées de fichiers/dossiers dans le dossier.
    pub entries: Vec<Entry>,
    /// Cache des textures chargées par Egui.
    pub textures: HashMap<PathBuf, egui::TextureHandle>,
    /// Taille d’affichage imposée des icônes (px).
    pub icon_display_px: f32,
    /// Les icônes pour lesquelles l'extraction a échouée ou qui n'existent pas.
    pub failed_icons: HashSet<PathBuf>,
    /// File d'attente MPSC pour éviter les doublons d'extraction de ce côté.
    pub queued_icons: HashSet<PathBuf>,
    /// Filtre de recherche.
    pub query: String,
    /// Configuration parsée depuis `.ini`.
    pub config: AppConfig,
    /// Dernière taille de texture mise en cache (utile pour l'invalidation sur changement d'échelle).
    pub last_tex_sz: i32,
    /// Détecteur de First-Frame (chargement du viewport).
    pub first_frame: bool,
    
    // Canaux Multithreads : Envois ou Réceptions depuis les workers.
    pub tx_req: Sender<IconRequest>,
    pub rx_res: Receiver<IconResponse>,
    /// Génération courante (compteur permettant d'annuler les scans fantômes du thread).
    pub generation: Arc<AtomicU32>,
}

impl ZoneApp {
    /// Initialise le Contrôleur et son "Modèle" interne en spawnant le Pool de threads système.
    pub fn new(config: &AppConfig, ctx: egui::Context) -> Self {
        let exe_root = exe_root_dir();
        let mut zones = discover_zones(&exe_root);
        if zones.is_empty() {
            zones.push(exe_root);
        }

        let folder = zones[0].clone();
        let icon_display_px = config.icon_size_px;

        let (tx_req, rx_req) = channel::<IconRequest>();
        let (tx_res, rx_res) = channel::<IconResponse>();
        let generation = Arc::new(AtomicU32::new(1));
        
        let shared_rx = Arc::new(Mutex::new(rx_req));
        
        // Démarrer 4 workers d'extraction lourde (ex: IShellItemImageFactory)
        for _ in 0..4 {
            let rx = shared_rx.clone();
            let tx = tx_res.clone();
            let gen_clone = generation.clone();
            let thread_ctx = ctx.clone();
            
            std::thread::spawn(move || {
                win_icon::init_com_worker_thread();
                loop {
                    // Verrouillage minimal du receveur pour extraire la demande
                    let req = match rx.lock().unwrap().recv() {
                        Ok(r) => r,
                        Err(_) => break, // Destruction du Controller
                    };
                    
                    // Sécurité : Skip l'extraction si la page actuelle n'est plus demandée
                    if req.generation != gen_clone.load(Ordering::Relaxed) {
                        continue;
                    }
                    
                    let img = win_icon::icon_for_path(&req.path, req.size_px);
                    
                    // Double check après un calcul long
                    if req.generation == gen_clone.load(Ordering::Relaxed) {
                        if tx.send(IconResponse {
                            path: req.path,
                            image: img,
                            generation: req.generation,
                        }).is_err() {
                            break;
                        }
                        // Demande manuelle à egui de redessiner la vue.
                        thread_ctx.request_repaint();
                    }
                }
            });
        }

        let mut s = Self {
            zones,
            zone_index: 0,
            folder,
            entries: Vec::new(),
            textures: HashMap::new(),
            icon_display_px,
            failed_icons: HashSet::new(),
            queued_icons: HashSet::new(),
            query: String::new(),
            config: config.clone(),
            last_tex_sz: 0,
            first_frame: true,
            tx_req,
            rx_res,
            generation,
        };
        s.rescan();
        s
    }

    /// Permet de revenir au répertoire configuré précédent via un modulo index.
    pub fn go_prev(&mut self) {
        if self.zones.len() <= 1 {
            return;
        }
        self.zone_index = (self.zone_index + self.zones.len() - 1) % self.zones.len();
        self.folder = self.zones[self.zone_index].clone();
        self.rescan();
    }

    /// Avance au dossier suivant et force le rafraichissement interne.
    pub fn go_next(&mut self) {
        if self.zones.len() <= 1 {
            return;
        }
        self.zone_index = (self.zone_index + 1) % self.zones.len();
        self.folder = self.zones[self.zone_index].clone();
        self.rescan();
    }

    /// Parcourt en temps réel le dossier local pour trouver les fichiers, 
    /// en incrémentant la génération logicielle.
    pub fn rescan(&mut self) {
        // Incrément de génération : les workers en arrière-plan abandonneront immédiatement la charge restante de l'ancienne page.
        self.generation.fetch_add(1, Ordering::Relaxed);
        self.entries.clear();
        self.textures.clear();
        self.failed_icons.clear();
        self.queued_icons.clear();

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
                continue; // Ignore les cachés
            }
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            self.entries.push(Entry { path, name, is_dir });
        }
    }

    /// Assigne le Titre d'interface en fonction du chemin ou du dossier en cours.
    pub fn title_label(&self) -> String {
        self.folder
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.folder.display().to_string())
    }

    /// Scanne de manière asynchrone toutes les entrées qui n'ont pas de texture dans ce cycle de Vue.
    /// Charge les requêtes dans le Channel MPSC pour le thread. Ne bloque pas MainUI.
    pub fn load_pending_icons(&mut self, ctx: &egui::Context) {
        let tex_sz = (self.icon_display_px * 4.0 / 3.0)
            .round()
            .clamp(64.0, 128.0) as i32;

        if self.last_tex_sz != 0 && self.last_tex_sz != tex_sz {
            self.generation.fetch_add(1, Ordering::Relaxed);
            self.textures.clear();
            self.failed_icons.clear();
            self.queued_icons.clear();
        }
        self.last_tex_sz = tex_sz;

        let current_gen = self.generation.load(Ordering::Relaxed);

        // Réception non bloquante du thread en arrière-plan
        while let Ok(res) = self.rx_res.try_recv() {
            if res.generation == current_gen {
                if let Some(img) = res.image {
                    let tex = ctx.load_texture(
                        format!("icn_{}", res.path.display()),
                        img,
                        egui::TextureOptions::LINEAR,
                    );
                    self.textures.insert(res.path.clone(), tex);
                } else {
                    self.failed_icons.insert(res.path.clone());
                }
            }
        }

        let q = self.query.trim().to_lowercase();
        let filter_active = !q.is_empty();

        let mut sent_this_frame = 0;
        for e in &self.entries {
            let p = &e.path;
            if self.textures.contains_key(p) || self.failed_icons.contains(p) || self.queued_icons.contains(p) {
                continue; // Icône déjà prête ou demandée.
            }
            if filter_active && !e.name.to_lowercase().contains(&q) {
                continue; // Pas besoin de le traiter car exclu de l'écran.
            }
            
            let _ = self.tx_req.send(IconRequest {
                path: p.clone(),
                size_px: tex_sz,
                generation: current_gen,
            });
            self.queued_icons.insert(p.clone());
            
            sent_this_frame += 1;
            // Ne pas saturer le queue mutuelle si la vue est massive.
            if sent_this_frame >= 64 {
                break;
            }
        }
    }
}

// Implémente le point d'entrée d'Egui sur notre contrôleur en déléguant à la Vue `ui.rs`.
impl eframe::App for ZoneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Délègue entièrement le dessin du visuel aux fonctions externes (View).
        ui::update_view(self, ctx);
    }
}
