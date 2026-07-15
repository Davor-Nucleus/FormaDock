<div align="center">

  # Forma Dock

  **Un utilitaire de bureau moderne pour Windows, organisez vos icônes, raccourcis et dossiers dans des zones transparentes et personnalisables, ancrées sur votre fond d'écran.**

  [![Langage principal](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](#)
  [![Framework](https://img.shields.io/badge/egui-eframe-8B5CF6?style=for-the-badge)](#)
  [![Licence](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](#)
</div>

---

Construit en Rust avec `eframe`/`egui`, Forma Dock propose de hautDavor Nucleuses performances grâce à une extraction multithread des icônes natives de Windows et un rendu d'interface élégant sans bordures avec gestion de transparence.

## 📋 Sommaire

- [Vue d'ensemble](#-vue-densemble)
- [Prérequis](#-prérequis)
- [Installation](#-installation)
- [Architecture](#-architecture)
- [Configuration](#-configuration)
- [Utilisation](#-utilisation)
- [Dépannage](#-dépannage)
- [Contribution](#-contribution)

---

## 🔭 Vue d'ensemble

Forma Dock a été conçu pour résoudre le problème des bureaux Windows encombrés. L'application permet de regrouper vos icônes, raccourcis et dossiers dans de petites boîtes ou "zones" configurables ancrées sur votre bureau.

1. 🌐 **Zones de bureau ancrées** — S'attache directement au fond d'écran (Always On Bottom) sans interférer avec vos tâches.
2. ⚡ **Extraction native d'icônes** — Récupération haute-fidélité des icônes Windows via les API Shell pour `.ico`, `.lnk`, et raccourcis web.
3. 💾 **Vraie transparence** — Paramétrage fin de l'opacité et de la couleur de l'interface en gardant les éléments visuels nets.
4. 🎨 **Configuration facile** — Un simple fichier `forma_dock.ini` permet d'ajuster les tailles, les coins et l'apparence à la volée.

---

## ⚡ Prérequis

> [!IMPORTANT]
> Assurez-vous d'avoir les éléments suivants installés sur votre machine avant de continuer.

- **[Rust](https://www.rust-lang.org/tools/install)** >= 1.70 (outil `cargo` inclus)
- **Système d'exploitation Windows** (utilise les API natives et `windows_subsystem`)
- **Compilateur MSVC** (généralement via Visual Studio Build Tools pour Windows)

---

## 🚀 Installation

```bash
# 1. Cloner le dépôt
git clone https://github.com/votre-compte/forma-dock.git
cd forma-dock

# 2. Compiler l'application
cargo build --release

# 3. Le fichier de configuration par défaut forma_dock.ini est déjà présent
```

---

## 🏗️ Architecture

<details>
<summary><b>Cliquez pour dérouler l'arborescence du projet</b></summary>

```
forma-dock/
├── src/
│   ├── app.rs          # Logique centrale de la zone (Contrôleur)
│   ├── model.rs        # Parsing du .ini et modèle de données
│   ├── ui.rs           # Rendu visuel de l'interface avec egui
│   ├── win_icon.rs     # Extraction Windows Shell des icônes en multithread
│   └── main.rs         # Point d'entrée, création Tray Icon et fenêtre eframe
├── forma_dock.ini      # Fichier de configuration utilisateur
├── Cargo.toml          # Dépendances (eframe, tray-icon, windows, etc.)
└── README.md           # Ce fichier
```
</details>

---

## ⚙️ Configuration

Modifiez ou créez le fichier `forma_dock.ini` à la racine de l'application pour personnaliser votre espace.

> [!NOTE]
> Les valeurs ci-dessous sont les valeurs par défaut. Si le fichier est absent, l'application les utilise automatiquement.

<details>
<summary><b>Exemple de fichier <code>forma_dock.ini</code></b></summary>

```ini
[window]
width=440
height=560
min_width=320
min_height=280
transparent=true
decorations=false
opacity=40
bg_color=#0E1018
corner_radius=10

[icons]
size_px=118
```
</details>

| Variable            | Description                               | Valeur par défaut |
| ------------------- | ----------------------------------------- | ----------------- |
| `width`, `height`   | Taille de démarrage de la fenêtre         | `440`, `560`      |
| `min_width`, `min_height` | Taille minimale de la fenêtre        | `320`, `280`      |
| `transparent`       | Rend le fond de l'application transparent | `true`            |
| `decorations`       | Affiche les bordures natives de Windows   | `false`           |
| `opacity`           | Niveau d'opacité (0 à 100)                | `40`              |
| `bg_color`          | Couleur de fond (hexadécimal)             | `#0E1018`         |
| `corner_radius`     | Rayon des coins de la fenêtre             | `10`              |
| `size_px`           | Taille des icônes affichées               | `118`             |

---

## 💻 Utilisation

### Lancement standard

```bash
# Lancement en développement
cargo run

# Lancement optimisé
cargo run --release
```

### Cas d'usage

- **Navigation entre zones** : utilisez les flèches ◀ ▶ dans l'interface, ou les touches `←` et `→` du clavier.
- **Zoom sur les icônes** : molette de souris ou trackpad (pincer/zoom) pour agrandir/rétrécir les icônes.
- **Recherche** : champ de recherche en haut à droite pour filtrer les fichiers et dossiers par nom.
- **Fermeture** : clic droit sur l'icône dans la zone de notification (près de l'horloge système) → "Quitter".

---

## 🛠️ Dépannage

- **La fenêtre ne s'affiche pas ?**
  Vérifiez que votre bureau est accessible et qu'aucune autre application ne bloque les fenêtres sans bordures.
- **Les icônes ne s'affichent pas ?**
  L'extraction utilise les API Shell Windows. Certains fichiers très spéciaux peuvent ne pas avoir d'icône associée. Un emoji de remplacement (📁 ou 📄) sera alors affiché.
- **La compilation échoue ?**
  Assurez-vous d'utiliser la dernière version de Rust : `rustup update`.

> [!WARNING]
> Sous Windows, le sous-système `windows_subsystem = "windows"` masque la console. Utilisez `cargo run` (sans `--release` ?) pour voir les éventuels messages d'erreur en développement.

---

## 🤝 Contribution

Les contributions (issues, pull requests) sont toujours les bienvenues ! N'hésitez pas à proposer vos améliorations.

---

<div align="center">
  <i>Développé avec ❤️  par Davor Nucleus</i>
</div>