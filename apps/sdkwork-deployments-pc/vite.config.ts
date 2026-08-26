import { resolveBrowserDistOutDir } from '../../../sdkwork-specs/tools/browser-dist-layout.mjs';
function resolveViteEnvironment(mode: string | undefined, processEnv = process.env) {
  const profileMatch = /^(standalone|cloud)\.(development|test|staging|production)$/u.exec(mode ?? '');
  return profileMatch?.[2]
    ?? (['development', 'test', 'staging', 'production'].includes(processEnv.SDKWORK_ENVIRONMENT ?? '')
      ? (processEnv.SDKWORK_ENVIRONMENT ?? 'production')
      : 'production');
}
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
export default defineConfig({ plugins: [react()], server: { port: 5181, strictPort: false }, preview: { port: 4181 }, build: {
      outDir: resolveBrowserDistOutDir(resolveViteEnvironment(mode, process.env)), target: "es2022", sourcemap: true } });

