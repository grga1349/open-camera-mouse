import React from "react";
import { createRoot } from "react-dom/client";
import "./style.css";
import App from "./App";
import { ParamsProvider } from "./state/useParams";
import { RunningProvider } from "./state/useRunning";
import { StatusProvider } from "./state/useStatus";

const container = document.getElementById("root");

const root = createRoot(container!);

root.render(
  <React.StrictMode>
    <ParamsProvider>
      <RunningProvider>
        <StatusProvider>
          <App />
        </StatusProvider>
      </RunningProvider>
    </ParamsProvider>
  </React.StrictMode>,
);
