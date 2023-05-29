import React from "react";
import ReactDOM from "react-dom/client";
import "./main.css";
import init from "./pkg/backlog_realtime_preview";
import App from "./src/App";

init().then(() => {
  const editor = document.getElementById(
    "page.content"
  ) as HTMLTextAreaElement | null;
  const parent = editor?.parentElement;
  if (editor && parent) {
    const preview = document.createElement("div");
    preview.id = "preview";
    preview.className = "loom";
    editor.after(preview);

    const controller = document.createElement("div");
    controller.id = "controller";
    parent.classList.add("parent");
    parent.insertBefore(controller, parent.firstChild);
    ReactDOM.createRoot(controller).render(
      <React.StrictMode>
        <App editor={editor} preview={preview} />
      </React.StrictMode>
    );
  }
});
