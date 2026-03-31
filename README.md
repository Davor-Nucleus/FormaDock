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

La base de code respecte le cadriciel **MVC (Modèle-Vue-Contrôleur)** couplé à une conception multithreadée :

### 1. Structure Modulaire (`src/`)
- **`model.rs` (Modèle)** : Contient les définitions pures de données (`AppConfig` et le parseur `.ini`, `IconRequest`, `Entry`), dénuées de charge UI ou système.
- **`app.rs` (Contrôleur)** : Cœur logique de `ZoneApp`. Gère la configuration d'état (State), l'ouverture des dossiers, le filtre de recherche, ainsi que le pool de threads en arrière-plan et sa file d'attente.
- **`ui.rs` (Vue)** : Pilote exclusif du moteur WGPU via Egui. Ne stocke rien, lit le `model` via `app`, et rafraichit la fenêtre (Background et Responsive grid).
- **`main.rs`** : Orchestrateur initial et lanceur dans la `tray_icon` (fenêtre d'arrière plan).

### 2. File système Asynchrone Multithreadée (Multi-Threading)
L'application crée **4 Threads Systèmes "Workers" invisibles** au démarrage.
Au moment de charger de gigantesques répertoires d'icônes, l'application soumet des paquets de travail (MPSC).
- **0 Blocage Visuel** : La fenêtre reste fluide à ~144 Hz et vos curseurs/scrolls ne gèleront jamais, même sur de lentes API `IShellItem` réseau.
- **Refoulement intelligent ("Ghost cancellation")** : Lorsqu'un dossier est fermé, le contrôleur émet une nouvelle "Génération de Page". Les threads jèteront gracieusement les milliers d'icônes obsolètes calculées qui n'ont plus lieu d'être sans provoquer de "panic!" mémoire (Memory Leak ou Ghost loading).

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
