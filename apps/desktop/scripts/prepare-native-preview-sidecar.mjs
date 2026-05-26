import { execFileSync, spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const desktopRoot = resolve(scriptDir, "..");
const workspaceRoot = resolve(desktopRoot, "..", "..");
const tauriRoot = resolve(desktopRoot, "src-tauri");
const sidecarBaseName = "tiles-native-preview";

const args = process.argv.slice(2);

function hasFlag(name) {
  return args.includes(name);
}

function optionValue(name) {
  const prefix = `${name}=`;
  const inline = args.find((arg) => arg.startsWith(prefix));

  if (inline) {
    return inline.slice(prefix.length);
  }

  const index = args.indexOf(name);

  if (index >= 0) {
    return args[index + 1];
  }

  return undefined;
}

function usage() {
  console.log(`Prepare the native preview Tauri sidecar.

Usage:
  npm run sidecar:prepare -- [--release] [--profile <name>] [--target <triple>] [--dry-run]

Examples:
  npm run sidecar:prepare -- --release
  npm run sidecar:prepare -- --release --target x86_64-pc-windows-msvc

The copied binary is named ${sidecarBaseName}-<target-triple>[.exe] under src-tauri/binaries/.`);
}

function hostTargetTriple() {
  const rustc = process.env.RUSTC ?? "rustc";
  const tuple = spawnSync(rustc, ["--print", "host-tuple"], {
    cwd: workspaceRoot,
    encoding: "utf8",
  });

  if (tuple.status === 0 && tuple.stdout.trim()) {
    return tuple.stdout.trim();
  }

  let version;

  try {
    version = execFileSync(rustc, ["-vV"], {
      cwd: workspaceRoot,
      encoding: "utf8",
    });
  } catch (error) {
    throw new Error(
      `Could not run ${rustc} to determine the host target triple. ` +
        `Pass --target <triple> or set RUSTC to an accessible rustc binary. ${error.message}`,
    );
  }
  const hostLine = version
    .split(/\r?\n/)
    .find((line) => line.trim().startsWith("host:"));

  if (!hostLine) {
    throw new Error("Could not determine the Rust host target triple.");
  }

  return hostLine.split(/\s+/)[1];
}

function binaryExtension(targetTriple) {
  return targetTriple.includes("windows") ? ".exe" : "";
}

function sidecarFileName(targetTriple) {
  return `${sidecarBaseName}-${targetTriple}${binaryExtension(targetTriple)}`;
}

function sourceBinaryPath({ targetTriple, profile, targetWasExplicit }) {
  const profileDir = profile === "release" ? "release" : profile;
  const fileName = `${sidecarBaseName}${binaryExtension(targetTriple)}`;

  if (targetWasExplicit) {
    return join(workspaceRoot, "target", targetTriple, profileDir, fileName);
  }

  return join(workspaceRoot, "target", profileDir, fileName);
}

if (hasFlag("--help") || hasFlag("-h")) {
  usage();
  process.exit(0);
}

const explicitTarget =
  optionValue("--target") ??
  process.env.CARGO_BUILD_TARGET ??
  process.env.TARGET ??
  undefined;
const targetTriple = explicitTarget ?? hostTargetTriple();
const profile =
  optionValue("--profile") ?? (hasFlag("--release") ? "release" : "debug");
const release = profile === "release";
const targetWasExplicit = Boolean(explicitTarget);
const dryRun = hasFlag("--dry-run");

const sourcePath = sourceBinaryPath({
  targetTriple,
  profile,
  targetWasExplicit,
});
const destinationDir = join(tauriRoot, "binaries");
const destinationPath = join(destinationDir, sidecarFileName(targetTriple));

if (dryRun) {
  console.log(
    JSON.stringify({ targetTriple, profile, sourcePath, destinationPath }, null, 2),
  );
  process.exit(0);
}

const cargoArgs = ["build", "-p", sidecarBaseName];

if (release) {
  cargoArgs.push("--release");
} else if (profile !== "debug") {
  cargoArgs.push("--profile", profile);
}

if (targetWasExplicit) {
  cargoArgs.push("--target", targetTriple);
}

console.log(`Building native preview sidecar: cargo ${cargoArgs.join(" ")}`);

const build = spawnSync("cargo", cargoArgs, {
  cwd: workspaceRoot,
  stdio: "inherit",
});

if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

if (!existsSync(sourcePath)) {
  throw new Error(
    `Expected native preview binary was not found at ${sourcePath}.`,
  );
}

mkdirSync(destinationDir, { recursive: true });
copyFileSync(sourcePath, destinationPath);

if (process.platform !== "win32") {
  chmodSync(destinationPath, 0o755);
}

console.log(`Prepared Tauri sidecar: ${destinationPath}`);
