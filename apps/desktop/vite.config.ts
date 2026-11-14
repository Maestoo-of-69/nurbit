import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vite builds the interface. In development Tauri loads it from this dev server
// *inside the app window* (hot reload). In the installed app the built files in
// dist/ are embedded into the .exe and no server is running.
export default defineConfig({
  plugins: [react()],
  clearScreen: false, // keep Rust compiler output visible
  server: {
    port: 1420,
    strictPort: true, // tauri.conf.json expects exactly this port
    host: "127.0.0.1",
    watch: { ignored: ["**/src-tauri/**"] }, // Rust changes are handled by Tauri
  },
  build: {
    target: "chrome120", // WebView2 on Windows is Chromium-based
    outDir: "dist",
  },
});
