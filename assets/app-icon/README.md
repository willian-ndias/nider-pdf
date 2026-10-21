# Nider PDF app icon

<img src="pdfcraft-small.svg" alt="Nider PDF app icon: a sheet with a red PDF label on a graphite tile, the Nider mark in the corner" width="128">

**Design:** the Nider app family (Nider PDF, Nider Text, Nider Etapas): a graphite tile with a silver edge,
the app's symbol in the middle (a sheet with a red **PDF** label) and the Nider mark (hexagon, six
vertex hexagons, the Southern Cross) cropped into the lower-right corner in the app's colour.

**Palette:** graphite `#232830`→`#0b0e12`, silver edge `#3a414b`, paper `#E9ECF0`, Nider PDF red `#F2564F`.

**Tile:** `viewBox="0 0 256 256"`, a rounded square (`rx=56`) with a small margin; everything inside it.
The lettering is outlines (no font dependency), so every renderer draws it the same.

**Provenance:** original artwork by the project owner (Nider), drawn as code. Licence:
[LICENSE.txt](LICENSE.txt) (`MIT OR Apache-2.0`, like the repo). The lettering's outlines come from
Liberation Sans Bold (SIL OFL 1.1), which allows this use.

**Crispness rule:** in the app the icon is always drawn from the vector (`widgets::nider_brand`); PNGs are
only for the system (taskbar, Dock, Linux theme). Never scale a small PNG down for the UI.

## Files

| File | What it is |
|---|---|
| `pdfcraft.svg` | the master vector (traced at 2048 px); every PNG, `.ico` and `.icns` is rendered from it |
| `pdfcraft-small.svg` | a lighter vector (traced at 1024 px) for places where size matters, such as this README |
| `pdfcraft-1024.png` | 1024 px on Apple's grid; also the runtime Dock icon on macOS |
| `pdfcraft.icns` | macOS icon (16–1024 px) |
| `pdfcraft.ico` | Windows icon (16–256 px), embedded in `pdfcraft.exe` by `apps/pdfcraft/build.rs` |
| `pdfcraft-document.svg` | Windows PDF file icon master: a Lucide document outline, green badge and outlined PDF lettering; no font dependency |
| `pdfcraft-document.ico` | Windows PDF file icon (16–256 px), embedded as resource 2 in `pdfcraft.exe` |
| `hicolor/<n>x<n>/apps/ai.storyteller.pdfcraft.png` | Linux hicolor theme, 16–512 px; the 256 px one is the runtime icon on Windows and Linux |
| `hicolor/scalable/apps/ai.storyteller.pdfcraft.svg` | Linux scalable icon (copy of the master) |

Where it shows: `apps/pdfcraft/src/main.rs` sets the window icon (Dock, taskbar, Alt-Tab, launcher) and the
Wayland app id `ai.storyteller.pdfcraft`; `packaging/linux/ai.storyteller.pdfcraft.desktop` names the
hicolor icon.

Windows PDF files use resource 2 (`"pdfcraft.exe",-2`) when PdfCraft is chosen as the default
reader; the app keeps resource 1. The document outline is adapted from Lucide `file-text`
1.49.0 (ISC); its badge and outlined lettering are original (`MIT OR Apache-2.0`), covered by
[LICENSE-document.txt](LICENSE-document.txt). Both icons are embedded, requiring no runtime
sidecar. This provides a file type icon; first-page thumbnails still need a thumbnail provider.

## Regenerate

```sh
packaging/icons.sh        # needs resvg and python3; iconutil (macOS) for the .icns
cargo xtask assets        # then update the sha256 values in ATTRIBUTION.toml and run with --write
```
