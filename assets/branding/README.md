# Tollo owl icons

The sleepy owl and original punk owl are available alongside two brighter punk
palettes. All PNG masters have transparent backgrounds and were prepared using the
built-in image generation tool from the selected concept images.

| Option | Palette | Master |
| --- | --- | --- |
| `sleepy` | Plum and cream sleepy owl | `sleepy-owl.png` |
| `punk` | Original charcoal and yellow punk owl | `punk-owl.png` |
| `punk-ice` | Ice blue and orange punk owl | `punk-ice-owl.png` |
| `punk-copper` | Copper and cream punk owl | `punk-copper-owl.png` |

Palette-edit prompts: retain the original punk owl's shape, expression, pose,
markings and shading style; brighten the outer feather silhouette for the dark
sidebar. Ice uses pale blue feathers, orange accents and cream markings. Copper
uses copper-orange feathers, buttery cream accents and sand-colored markings.

Background-removal prompt: remove only the light background and ground shadow;
preserve the owl's design, colors, expression, proportions, full body, crest,
feet, feather markings and shading; keep cream areas and eye glints opaque;
use clean antialiased edges and a square transparent canvas with clear margins.

From the repository root, after `pnpm install`:

```sh
pnpm icon:select sleepy
# Or, choose any punk palette:
pnpm icon:select punk
pnpm icon:select punk-ice
pnpm icon:select punk-copper
pnpm build:tauri
```

The selection regenerates desktop PNG, ICO and ICNS assets in `src-tauri/icons`
and updates `public/logo.png` (sidebar and browser favicon). Commit these outputs
with any default change: release builds use the checked-in selection.
Run `pnpm icon:select sleepy` to restore the default.

## Testing in development

Stop the running Tauri dev process, select an icon, then run `pnpm dev:tauri`
again. Vite hot reload updates the sidebar and favicon, but the native window
icon is embedded in the Rust executable. The build script tracks `icons/` so
changes regenerate the embedded icon on the next compilation. Tauri uses the
first PNG in `bundle.icon` on Linux; the 512-pixel `icon.png` is listed first
to avoid enlarging a 32-pixel image for the window icon.

On Wayland, the dock may resolve the icon through a `.desktop` launcher instead
of the embedded window icon. Restarting dev mode does not install or update that
launcher. Check `Icon=` in `~/.local/share/applications/tollo.desktop` (or the
system-installed Tollo desktop file). A launcher pointing to another checkout
will keep showing that checkout's icon. A user launcher can also take precedence
over a system-installed launcher. After correcting its icon path, the dock may
need to be restarted to clear its cached image.

## Testing installed packages

Install the rebuilt package to test menu and dock/taskbar icons. This is a
build-time selection, not an in-app setting. All variants retain the same app
identity, so they replace one another rather than installing side by side.
An already installed app or pinned shortcut may retain its cached icon until
it is restarted or its shortcut is refreshed.

Tauri's native installers handle launcher integration. On Debian/Ubuntu, install
the `.deb` for an application-menu entry. An AppImage is portable and needs
separate desktop integration to appear in the application menu. Permanent dock
or taskbar pinning is a user action. Test installed releases, not only Vite or
`tauri dev`, when checking OS integration.
