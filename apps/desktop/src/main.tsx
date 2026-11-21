import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./i18n";
import { platform } from "./lib/platform";
import "./styles/index.css";

// Styles that differ by system key off this (index.css: scrollbars on Windows).
document.documentElement.dataset.platform = platform;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

// Development only: ?demo=<state> opens a named screen for screenshots (see lib/demo.ts).
if (import.meta.env.DEV) void import("./lib/demo").then((demo) => demo.runDemo());
