# Comprendre la Transformation de Fourier Discrète (TFD)

Application interactive pour comprendre et visualiser la Transformation de Fourier Discrète (TFD).

[article](20181204-tfd-introduction/index.md)

## Application Rust / Dioxus

Le projet a été converti de C++/Qt vers **Rust** avec l'interface graphique **Dioxus 0.7.3** (desktop).

### Prérequis système (Linux)

```bash
sudo apt-get install -y \
    libglib2.0-dev libgtk-3-dev \
    libwebkit2gtk-4.1-dev libssl-dev \
    libxdo-dev pkg-config
```

### Compilation et lancement

```bash
# Mode développement
cargo run

# Mode release (recommandé pour le calcul TFD avec N élevé)
cargo run --release
```

### Fonctionnalités

- **Jusqu'à 5 sinusoïdes** configurables indépendamment (ω, φ, fréquence)
- **Visualisation du signal** continu (3 périodes)
- **Visualisation de l'échantillonnage** discret (N points à Fe Hz)
- **Calcul de la TFD** (algorithme O(N²) pédagogique) et affichage du spectre
- **Paramètres** : Fm (fréquence à mesurer), Fe (fréquence d'échantillonnage), N (nombre de points), SA (précision spectrale = Fe/N)

### Architecture

| Fichier | Rôle |
|---|---|
| `src/main.rs` | Interface Dioxus : composants App, SinusConfigRow, LineChart |
| `src/dft.rs`  | Logique de traitement du signal : `SinusConfig`, `sample_signal`, `compute_dft` |

### Tests

```bash
cargo test
```

