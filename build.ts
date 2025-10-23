#!/usr/bin/env bun

try {
  const result = await Bun.build({
    entrypoints: ["./proxy.ts"],
    outdir: ".",
    target: "node",
    format: "esm",
    minify: false,
    sourcemap: "inline",
    naming: "[dir]/[name].[ext]",
  });

  console.log(result.outputs);
} catch (e) {
  const error = e as AggregateError;
  console.error("Build failed:");
  for (const msg of error.errors) {
    if ("position" in msg) {
      console.error(
        `${msg.message} at ${msg.position?.file}:${msg.position?.line}:${msg.position?.column}`,
      );
    } else {
      console.error(msg.message);
    }
  }
}
