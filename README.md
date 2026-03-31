# 📁 frence 

frence est un lanceur/explorateur d'applications et de fichiers dynamique, léger et personnalisable conçu pour Windows. Développé en **Rust** et reposant sur le framework graphique **egui** (via eframe/wgpu), il a pour objectif d'offrir une expérience de type *"dock"* ou *"launcher"* sur le bureau, parfaitement intégrée et très esthétique grâce à ses fenêtres transparentes.

## 🚀 Fonctionnalités Clés

- **Rendu natif des icônes Windows** : frence interroge directement les API système (`IShellItemImageFactory`, `CoCreateInstance`, etc.) pour extraire fidèlement les icônes haute résolution associées à vos fichiers.
- **Support avancé des Raccourcis** : Reconnaît et déchiffre les objets COM des raccourcis Windows classiques (`.lnk`) ainsi que les liens internet classiques (`.url`) afin d'offrir l'icône réelle visée par la redirection.
- **Répertoires "Zones" & Pagination** : Gère plusieurs "Zones" (Dossiers) en même temps. Il suffit d’utiliser les flèches du clavier ou les boutons pour basculer instantanément d'une collection d'icônes à l'autre.
- **Filtre de recherche instantané** : Interface de filtrage de contenu très réactive.
- **Apparence et Vitrage (*Glassmorphism*)** : Autorise une application complète sans bordures (`decorations = false`) et un effet de verre ultra-personnalisable avec ombres, opacité et arrondis modifiables à la volée grâce à sa compatibilité avec les textures du canal Alpha.
- **Zoom vectoriel & adaptatif** : Molette ou curseur intégré pour définir la taille exacte des tuiles de la grille à tout instant.

## 🛠️ Analyse Technique Globale

Le projet sépare la logique système de la logique graphique pour produire une application extrêmement légère, sans pour autant sacrifier WGPU.

### 1. Le Cœur Graphique (`src/main.rs`)
Responsable de l'interaction utilisateur et de la rétention d'état (fenêtre sans état pur). 
- Modèle l'application comme un `eframe::App` (`ZoneApp`).
- Maintient un chargeur d'icônes intelligent qui génère à la volée de nouvelles textures `egui::TextureHandle` sans bloquer complètement l'interface grâce à un *time-budget* (budget temps de 8 ms maximum par rafraîchissement d'icône d'affilée).
- Nettoyage sécurisé à chaque basculement de page : pour ne pas surcharger la RAM ni crasher l'intégration WGPU, la libération des pointeurs se trouve isolée hors du thread de validation du cycle d'instructions (*Update function end-tail allocation*).
- Lit en temps réel et applique un fichier de style `.ini`.

### 2. Pont COM / Win32 (`src/win_icon.rs`)
Cœur d'interaction avec le noyau Windows gérant tout un ensemble de pointeurs `unsafe`.
- Transforme finement les balises de bas-niveau `HBITMAP` Windows GDI vers l'implémentation standardisée `ColorImage` d'Egui au travers d'un mapping DIB bit-par-bit pour corriger les failles liées à l'absence de canal de transparence (Alpha Masking) sur les anciennes icônes 32 bits de l'OS.
- Détermine proprement (grâce à l'équilibrage du gestionnaire d'appartements de processus `CoUninitialize`) l'encapsulage des requêtes natives `IShellLinkW` pour résoudre l'empreinte spatiale des cibles Windows.

## ⚙️ Configuration (`frence.ini`)

Frence ne surcharge pas la base de registre et utilise plutôt un fichier `frence.ini` placé dans le répertoire racine du lanceur, avec des options très fluides :

```ini
[window]
; Position et taille de l'UI
width = 440
height = 560
transparent = yes
decorations = no

; Mode sombre / translucide
bg_color = #0E1018
opacity = 40
corner_radius = 10

[icons]
size_px = 118
```

## 🏗️ Compiler et exécuter

L'écosystème Rust standard suffit pour son déploiement complet. Sous Windows, l'option `--release` est conseillée pour maximiser la vitesse du backend WGPU.

```bash
# Lancement simple
cargo run --release

# Compilation pure
cargo build --release
```

> **Note** : Le mode d'exécution cache volontairement le terminal de l'application via le tag `#![windows_subsystem = "windows"]` pour se nicher exclusivement dans la zone de notification (Barre des tâches / Tray Icon).
