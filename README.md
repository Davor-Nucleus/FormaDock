# 🚀 Forma Dock
 Un utilitaire de bureau moderne pour Windows permettant d'organiser et d'afficher vos icônes dans des zones personnalisables et transparentes.

---
## 🔍 Aperçu

Forma Dock a été conçu pour résoudre le problème des bureaux Windows encombrés. L'application permet de regrouper vos icônes, raccourcis et dossiers dans de petites boîtes ou "zones" configurables ancrées sur votre bureau (Always On Bottom). Construite en Rust avec `eframe`/`egui`, elle propose de hautes performances grâce à une extraction multithread des icônes natives de Windows et un rendu d'interface élégant sans bordures avec gestion de transparence.

---
## ✨ Fonctionnalités

- ✅ **Zones de bureau ancrées** — S'attache directement au fond d'écran sans interférer avec vos tâches.
- ✅ **Extraction native d'icônes** — Récupération haute-fidélité des icônes Windows via les API Shell pour `.ico`, `.lnk`, et raccourcis web.
- ✅ **Vraie transparence** — Paramétrage fin de l'opacité et de la couleur de l'interface en gardant les éléments visuels nets.
- ✅ **Configuration facile** — Un simple fichier `forma_dock.ini` permet d'ajuster les tailles, les coins et l'apparence à la volée.

---
## 🛠️ Prérequis
Il s'agit d'une application Rust native ciblant Windows.

- [Rust](https://www.rust-lang.org/tools/install) >= 1.70 (outil `cargo` inclus)
- Système d'exploitation Windows (utilise les API natives et `windows_subsystem`)
- Compilateur MSVC (généralement via Visual Studio Build Tools pour Windows)

---
## 📦 Installation

```bash
# Cloner le dépôt
git clone https://github.com/utilisateur/forma-dock.git
cd forma-dock

# Construire/Compiler l'application
cargo build --release

# Configurer l'application (le fichier par défaut forma_dock.ini est déjà présent)
```

---
## 🚀 Utilisation
```bash
# Lancer en développement (avec des informations de débogage éventuelles)
cargo run

# Lancer la version optimisée et finale
cargo run --release
```

### Exemple de base
Une fois lancée, l'application se fixe sur l'écran et s'exécute en arrière-plan. Vous pouvez la fermer via l'icône ajoutée discrètement dans la zone de notification (Tray Icon) près de l'horloge système en faisant un clic droit -> "Quitter".

---
## ⚙️ Configuration

Modifiez ou créez le fichier `forma_dock.ini` à la racine de l'application pour personnaliser votre espace :

| Variable            | Description                               | Valeur par défaut                         |
| ------------------- | ----------------------------------------- | ----------------------------------------- |
| `width`, `height`   | Taille de démarrage de la fenêtre         | `550`, `560`                              |
| `transparent`       | Rend le fond de l'application transparent | `true`                                    |
| `opacity`           | Niveau d'opacité (0 à 100)                | `5`                                       |
| `bg_color`          | Couleur de fond                           | `#0E1018`                                 |
| `corner_radius`     | Rayon des coins de la fenêtre             | `0`                                       |
| `size_px`           | Taille des icônes affichées               | `118`                                     |

---
## 🏗️ Architecture

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
└── README.md
```

---
## 🧪 Tests

```bash
# Lancer tous les tests unitaires et composants
cargo test

# Tests avec vérification du typage et des erreurs sans compilation complète
cargo check
```

---
## 🤝 Contribution



<p align="center">Fait avec ❤️ par <a href="https://github.com/utilisateur">utilisateur</a></p>