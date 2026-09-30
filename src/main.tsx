import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { CommandProvider } from "./lib/commands";
import "./index.css";
import "./ansiapps-theme.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <CommandProvider>
      <App />
    </CommandProvider>
  </React.StrictMode>,
);
