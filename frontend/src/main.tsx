import React from "react";
import { createRoot } from "react-dom/client";
import "./style.css";
import App from "./App";
import { ParamsProvider } from "./state/useParams";
import { RunningProvider } from "./state/useRunning";
import { TelemetryProvider } from "./state/useTelemetry";
import { PreviewProvider } from "./state/usePreview";

const container = document.getElementById("root");

const root = createRoot(container!);

root.render(
  <React.StrictMode>
    <ParamsProvider>
      <RunningProvider>
        <TelemetryProvider>
          <PreviewProvider>
            <App />
          </PreviewProvider>
        </TelemetryProvider>
      </RunningProvider>
    </ParamsProvider>
  </React.StrictMode>,
);
