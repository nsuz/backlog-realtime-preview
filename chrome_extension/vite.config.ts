import { crx, defineManifest } from "@crxjs/vite-plugin";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const manifest = defineManifest({
  manifest_version: 3,
  name: "Backlog Realtime Preiew",
  description: "Backlog Realtime Preview",
  version: "1.0.0",
  action: {
    default_icon: "favicon.jpg",
  },
  content_scripts: [
    {
      matches: ["https://*.backlog.com/*", "https://*.backlog.jp/*"],
      js: ["main.tsx"],
      run_at: "document_end",
      all_frames: false,
    },
  ],
  web_accessible_resources: [
    {
      matches: ["https://*.backlog.com/*", "https://*.backlog.jp/*"],
      resources: ["pkg/backlog_realtime_preview_bg.wasm"],
    },
  ],
});

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), crx({ manifest })],
});
