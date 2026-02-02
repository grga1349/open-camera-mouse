import type { FC } from "react";
import { Button } from "../Button";
import { cn } from "../../lib/cn";

type SettingsTabsProps<T extends string> = {
  tabs: readonly T[];
  activeTab: T;
  onChange: (tab: T) => void;
};

export const SettingsTabs = <T extends string>({ tabs, activeTab, onChange }: SettingsTabsProps<T>) => (
  <nav className="grid grid-cols-2 gap-2 border-b border-zinc-900 px-4 py-3">
    {tabs.map((tab) => {
      const isActive = tab === activeTab;
      return (
        <Button
          key={tab}
          fullWidth
          variant={isActive ? "action" : "highlight"}
          className={cn(
            "text-xs font-semibold uppercase tracking-wide",
            !isActive && "border-zinc-900 bg-zinc-950 text-zinc-400 hover:bg-zinc-900"
          )}
          onClick={() => onChange(tab)}
          aria-current={isActive ? "page" : undefined}
        >
          {tab}
        </Button>
      );
    })}
  </nav>
);
