import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { watchTheme } from "@/stores/theme";
import "@fontsource-variable/inter";
import "./index.css";

watchTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
