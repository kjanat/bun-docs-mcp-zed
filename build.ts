#!/usr/bin/env bun

const result = await Bun.build({
  entrypoints: ["./proxy.ts"],
  outdir: ".",
  target: "node",
  format: "esm",
  minify: true,
  sourcemap: "inline", // "external", "linked", "inline", or "none"
  naming: "[dir]/[name].[ext]",
});

console.log("Build success:", result.success);
console.log(
  "Outputs:",
  result.outputs.map((o) => o.path),
);

if (!result.success) {
  console.error("Build failed:");
  for (const message of result.logs) {
    console.error(message);
  }
}
