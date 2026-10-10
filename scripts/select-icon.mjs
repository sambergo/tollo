import { spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const variant = process.argv[2];
const variants = ["sleepy", "punk", "punk-ice", "punk-copper"];
if (process.argv.length !== 3 || !variants.includes(variant)) {
  console.error(`Usage: pnpm icon:select <${variants.join("|")}>`);
  process.exit(1);
}

const root = fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(import.meta.url);
const temporary = await mkdtemp(join(tmpdir(), "tollo-icons-"));
try {
  const result = spawnSync(
    process.execPath,
    [
      require.resolve("@tauri-apps/cli/tauri.js"),
      "icon",
      join(root, "assets", "branding", `${variant}-owl.png`),
      "--output",
      temporary,
    ],
    { cwd: root, stdio: "inherit" },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error("Tauri icon generation failed");

  // Keep only desktop assets; Tauri also generates unused mobile icons.
  const destination = join(root, "src-tauri", "icons");
  await mkdir(destination, { recursive: true });
  for (const name of [
    "32x32.png",
    "64x64.png",
    "128x128.png",
    "128x128@2x.png",
    "icon.png",
    "icon.ico",
    "icon.icns",
  ]) {
    await copyFile(join(temporary, name), join(destination, name));
  }
  await copyFile(
    join(temporary, "128x128@2x.png"),
    join(root, "public", "logo.png"),
  );
  console.log(
    `Selected ${variant} owl. Restart pnpm dev:tauri to refresh the native window icon.`,
  );
  console.log(
    "Menu and Wayland dock icons come from your desktop launcher; rebuild and reinstall for installed app icons.",
  );
} finally {
  await rm(temporary, { recursive: true, force: true });
}
