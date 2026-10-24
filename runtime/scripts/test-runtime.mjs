import { build } from "esbuild";
import { fileURLToPath } from "node:url";

const entryPoint = fileURLToPath(new URL("./test-handler-event.ts", import.meta.url));
const result = await build({
  entryPoints: [entryPoint],
  bundle: true,
  format: "esm",
  platform: "node",
  write: false,
});
const output = result.outputFiles[0];
if (!output) throw new Error("runtime test bundle was not produced");
const source = Buffer.from(output.contents).toString("base64");
await import(`data:text/javascript;base64,${source}`);