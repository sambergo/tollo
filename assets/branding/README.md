# Tollo owl icons

The default is the sleepy owl. The punk owl is retained as an alternate.
Both PNG masters have transparent backgrounds and were prepared using the
built-in image generation tool from the selected concept images.

Background-removal prompt: remove only the light background and ground shadow;
preserve the owl's design, colors, expression, proportions, full body, crest,
feet, feather markings and shading; keep cream areas and eye glints opaque;
use clean antialiased edges and a square transparent canvas with clear margins.

From the repository root, after `pnpm install`:

```sh
pnpm icon:select sleepy
# Or, to try the alternate:
pnpm icon:select punk
pnpm build:tauri
```

The selection regenerates desktop PNG, ICO and ICNS assets in `src-tauri/icons`
and updates `public/logo.png` (sidebar and browser favicon). Commit these outputs
with any default change: release builds use the checked-in selection.
Run `pnpm icon:select sleepy` to restore the default.

Install the rebuilt package to test menu and dock/taskbar icons. This is a
build-time selection, not an in-app setting. Both variants retain the same app
identity, so they replace one another rather than installing side by side.
An already installed app or pinned shortcut may retain its cached icon until
it is restarted or its shortcut is refreshed.

Tauri's native installers handle launcher integration. On Debian/Ubuntu, install
the `.deb` for an application-menu entry. An AppImage is portable and needs
separate desktop integration to appear in the application menu. Permanent dock
or taskbar pinning is a user action. Test installed releases, not only Vite or
`tauri dev`, when checking OS integration.
