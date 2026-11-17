import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./i18n";
import "./styles/index.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

// Development only: ?demo=<state> opens a named screen for screenshots (see lib/demo.ts).
if (import.meta.env.DEV) void import("./lib/demo").then((demo) => demo.runDemo());
