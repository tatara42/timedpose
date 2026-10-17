# Timed Pose

A high-performance, cross-platform gesture and figure drawing companion built in **Rust** with **eframe** (`egui` + `wgpu`). Designed specifically for digital and traditional artists practicing timed gesture, anatomy, and rhythm drawing.

---

## Highlights

- **Cross-Platform & Native:** Runs natively on **Linux** (Wayland & X11), **macOS**, and **Windows** with full hardware GPU acceleration.
- **Perceptual Drawing Aids:** Built-in perceptual filters (Squint Mode, Value Threshold / Silhouette, Grayscale, Canvas Inversion) designed to break symbol drawing and strengthen form analysis.
- **Academic Warmup & Custom Presets:** Structured practice routines—from rapid 30-second gesture drills to progressive academic warmups with scheduled rest breaks.
- **Smart Audio Engine:** Monotonic drift-free timer with ascending 3-2-1 countdown cues, pose transition chimes, rest break bells, and completion fanfares. Supports procedural synthesis or custom **`.wav`** and **`.ogg`** sound packs.
- **Instantaneous Image Switching:** Asynchronous dual lookahead preloader caches both upcoming and previous poses in the background for zero-latency navigation.
- **Stylus & Tablet Friendly:** Left-handed ergonomic shortcuts allow full navigation and tool toggles without lifting your drawing hand from the stylus or tablet.
- **Companion Floating Mode:** Toggle **Always-on-Top** and **Zen Mode** to float Timed Pose seamlessly beside Photoshop, Clip Studio Paint, Krita, or Blender.

---

## Perceptual Artist Tools

Timed Pose includes non-destructive real-time visual filters tailored to how artists learn to see:

| Feature | Key | Description |
| :--- | :---: | :--- |
| **Squint Mode (Blur)** | `B` | Applies a Gaussian box blur to eliminate micro-details, forcing your eyes to focus exclusively on major rhythm, gesture line-of-action, and big silhouette shapes. |
| **Value Threshold / Silhouette** | `S` | Converts the reference into a high-contrast binary 2-value graphic mask to study positive/negative space and silhouette clarity. |
| **Grayscale Mode** | `G` | Strips hue and saturation to evaluate tonal value hierarchy and lighting keys without color distraction. |
| **Horizontal Flip** | `F` | Mirrors the image horizontally to break eye habituation and immediately reveal drawing proportion errors. |
| **Upside-Down Inversion** | `U` | Inverts the image upside-down (the classic Betty Edwards technique) to bypass symbolic brain processing and draw purely abstract spatial relationships. |
| **Rule-of-Thirds Grid** | `R` | Overlays a subtle composition grid for alignment and spatial proportion referencing. |
| **Reset Pan / Zoom** | `Z` / `0` | Resets canvas zoom and offsets back to 100% center framing. |

---

## Training Formats & Presets

- **Classic Academic Warmup:** Progressive regimen modeled after traditional atelier figure drawing classes:
  - 10 poses $\times$ 30s (Gesture rhythms)
  - 5 poses $\times$ 1m (Structure & forms)
  - 2 poses $\times$ 5m (Volume & anatomy)
  - 5m Rest Break (Eye & wrist fatigue prevention)
  - 1 pose $\times$ 20m (Deep study & value rendering)
- **Quick Gestures:** Rapid 30s or 60s drills for loosening up before illustration sessions.
- **Deep Study:** Fixed 5-minute, 10-minute, or 20-minute sessions for sustained structural drawings.
- **Endless Mode:** Open-ended session with manual pose switching (`D` / `A`) and no time constraints.
- **Orientation Matching:** Filter reference folders on-the-fly to match your canvas (Portrait-only, Landscape-only, or All).

---

## Audio System & Custom Sounds

Timed Pose includes an integrated procedural sound synthesizer with zero external audio assets required out of the box, as well as full custom audio support:

- **Ascending 3-2-1 Countdown:** Synthesizes ascending pitch ticks (659Hz $\to$ 784Hz $\to$ 880Hz) during the final three seconds of every pose.
- **Pose Advancement:** Two-tone major chime (C5 $\to$ G5).
- **Rest Break Gong:** Deep harmonic resonant bell (E4 $\to$ B3) when rest breaks begin.
- **Session Finished:** Rising 4-note celebration fanfare (C5 $\to$ E5 $\to$ G5 $\to$ C6).
- **Custom Sound Packs:** Add your own custom **`.wav`** or **`.ogg`** files in the Home menu for any sound event. All custom sounds are loaded directly into an in-memory cache for 0ms disk latency, with interactive preview buttons and one-click reset to defaults.

---

## Hotkey Reference

Designed for single-handed left-hand use on keyboards and tablet macro pads:

| Shortcut | Action |
| :--- | :--- |
| `Space` | Pause / Resume Timer |
| `D` or `Right Arrow` | Skip to Next Pose |
| `A` or `Left Arrow` | Return to Previous Pose |
| `F` | Toggle Horizontal Mirror Flip |
| `U` | Toggle Upside-Down Inversion |
| `G` | Toggle Grayscale Mode |
| `S` | Toggle Value Threshold / Silhouette |
| `B` | Toggle Squint Blur Mode |
| `R` | Toggle Composition Grid Overlay |
| `K` | Bookmark Current Pose |
| `T` | Toggle Always-on-Top Floating Window |
| `Tab` | Toggle Zen Mode (Hides all HUD overlays) |
| `Z` or `Num 0` | Reset Pan & Zoom to Center |
| `Esc` | Return to Home / Exit Session |
| `Middle Click + Drag` | Pan Canvas |
| `Scroll Wheel` | Zoom in / Zoom out |

---

## Session Review & Bookmarking

- Press `K` (or click the Bookmark icon) at any time during a session to bookmark challenging or inspiring poses.
- Upon session completion, Timed Pose generates an interactive **Review Gallery** detailing total practice time, completed poses, and a gallery of your poses.
- Instantly start a focused repeat drill containing only your bookmarked poses with a single click.

---

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.80 or newer recommended)
- Standard C compiler and platform libraries:
  - **Linux (Ubuntu/Debian):**
    ```bash
    sudo apt-get install -y libasound2-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libwayland-dev
    ```
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`).
  - **Windows:** MSVC toolchain (`rustup toolchain install stable-x86_64-pc-windows-msvc`).

### Build & Run

```bash
# Clone the repository
git clone https://github.com/username/timedpose.git
cd timedpose

# Run the app in release mode
cargo run -p timedpose-ui --release

# Run the test suite
cargo test --workspace
```

---

## Architecture

The codebase is organized as a modular Rust workspace:

```text
timedpose/
├── crates/
│   ├── timedpose-core/     # Pure deterministic core: Timer, Deck, Presets, Session state machine
│   ├── timedpose-infra/    # Platform infrastructure: AudioService, AppConfig, ImageScanner, ImagePreloader
│   └── timedpose-ui/       # Presentation layer: eframe/egui views, Canvas renderer, HUD, Hotkeys, Review
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
