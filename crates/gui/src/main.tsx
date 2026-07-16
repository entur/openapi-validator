import React from "react";
import ReactDOM from "react-dom/client";
import { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";

// Entur Linje CSS — token CSS first, then component CSS in the prescribed order.
import "@entur/tokens/dist/base.css";
import "@entur/tokens/dist/styles.css";
import "@entur/tokens/dist/semantic.css";
import "@entur/a11y/dist/styles.css";
import "@entur/icons/dist/styles.css";
import "@entur/tab/dist/styles.css";
import "@entur/typography/dist/styles.css";
import "@entur/layout/dist/styles.css";
import "@entur/loader/dist/styles.css";
import "@entur/button/dist/styles.css";
import "@entur/alert/dist/styles.css";
import "@entur/menu/dist/styles.css";
import "@entur/modal/dist/styles.css";
import "@entur/tooltip/dist/styles.css";
import "@entur/form/dist/styles.css";
import "@entur/table/dist/styles.css";
import "@entur/dropdown/dist/styles.css";

import App from "./App";
import "./App.css";

// Bundle monaco instead of loading it from a CDN: the packaged app has no
// network access to jsdelivr, so the default loader would break offline.
self.MonacoEnvironment = {
  getWorker(_workerId: string, label: string) {
    if (label === "json") return new jsonWorker();
    return new editorWorker();
  },
};
loader.config({ monaco });

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
