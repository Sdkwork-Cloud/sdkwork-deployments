import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
export default defineConfig({ plugins: [react()], server: { port: 5181, strictPort: false }, preview: { port: 4181 }, build: { target: "es2022", sourcemap: true } });

