import React from "react";
import { createRoot } from "react-dom/client";
import "./style.css";
import App from "./App";
import { AppProvider } from "./state/useAppStore";

const container = document.getElementById("root");

const root = createRoot(container!);

root.render(
  <React.StrictMode>
    <AppProvider>
      <App />
    </AppProvider>
  </React.StrictMode>,
);
