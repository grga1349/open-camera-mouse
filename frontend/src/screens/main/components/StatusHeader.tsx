import type { FC } from "react";
import { Button } from "../../../components/Button";
import { cn } from "../../../lib/cn";

const statusColor = (lost: boolean) => (lost ? "text-red-400" : "text-emerald-400");

type StatusHeaderProps = {
  lost: boolean;
  fps: number;
  onOpenSettings: () => void;
};

export const StatusHeader: FC<StatusHeaderProps> = ({ lost, fps, onOpenSettings }) => (
  <header className="flex items-center justify-between rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
    <div className="text-left">
      <p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Open Camera Mouse</p>
      <p className={cn("text-base font-semibold", statusColor(lost))}>
        {lost ? "LOST" : "OK"} • {fps.toFixed(1)} fps
      </p>
    </div>
    <Button onClick={onOpenSettings}>Settings</Button>
  </header>
);
