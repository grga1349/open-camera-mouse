import { useState, type FC } from "react";
import { Button } from "../../components/Button";
import { ScreenShell } from "../../components/ScreenShell";
import { ChoiceButton } from "../../components/ChoiceButton";
import { SliderField } from "../../components/SliderField";
import { defaultParams } from "../../state/useParams";
import { useSettingsDraft } from "../../state/useSettingsDraft";
import type { Params } from "../../types/params";
import { deepClone } from "../../lib/clone";

const TEMPLATE_SIZES = [30, 45, 60];

type SettingsScreenProps = {
  onSave: (params: Params) => Promise<void>;
  onCancel: () => void;
};

export const SettingsScreen: FC<SettingsScreenProps> = ({ onSave, onCancel }) => {
  const { draft, dirty, update, updateDraft, resetDraft } = useSettingsDraft();
  const [saving, setSaving] = useState(false);

  const handleCancel = () => {
    resetDraft();
    onCancel();
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await onSave(draft);
      resetDraft();
    } catch {
      // failure is already surfaced via the shared app-level error banner;
      // draft stays dirty so the user's edits aren't lost.
    } finally {
      setSaving(false);
    }
  };

  const handleResetDefaults = () => {
    updateDraft(() => deepClone(defaultParams));
  };

  return (
    <ScreenShell
      header={
        <header className="flex items-center gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
          <Button onClick={handleCancel}>Back</Button>
          <div className="text-left">
            <p className="text-[11px] uppercase tracking-[0.2em] text-zinc-400">Settings</p>
            <p className="text-base font-semibold">Configure tracking + input</p>
          </div>
        </header>
      }
      footer={
        <footer className="flex items-center justify-end gap-3 rounded-2xl border border-zinc-900 bg-zinc-900 px-4 py-3">
          <Button onClick={handleResetDefaults} disabled={saving}>
            Reset
          </Button>
          <Button onClick={handleCancel} disabled={saving}>
            Cancel
          </Button>
          <Button variant="action" disabled={!dirty || saving} onClick={handleSave}>
            {saving ? "Saving…" : "Save"}
          </Button>
        </footer>
      }
      mainClassName="gap-4"
    >
      <div className="flex-1 overflow-auto rounded-2xl border border-zinc-900 bg-zinc-950 px-4 py-4">
        <div className="space-y-6 text-sm text-zinc-300">
          <div>
            <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Template size</p>
            <div className="flex gap-2">
              {TEMPLATE_SIZES.map((size) => (
                <ChoiceButton
                  key={size}
                  selected={draft.templateSizePx === size}
                  onClick={() => update({ templateSizePx: size })}
                >
                  {size}px
                </ChoiceButton>
              ))}
            </div>
          </div>

          <SliderField
            label={`Gain (${draft.gainMultiplier.toFixed(1)}x)`}
            min={1}
            max={30}
            step={0.5}
            value={draft.gainMultiplier}
            onChange={(value) => update({ gainMultiplier: value })}
          />

          <SliderField
            label={`Smoothing (${Math.round(draft.smoothing * 100)}%)`}
            min={0}
            max={85}
            step={5}
            value={Math.round(draft.smoothing * 100)}
            onChange={(value) => update({ smoothing: value / 100 })}
          />

          <SliderField
            label={`Dwell time (${draft.dwellTimeMs} ms)`}
            min={200}
            max={1500}
            step={50}
            value={draft.dwellTimeMs}
            onChange={(value) => update({ dwellTimeMs: value })}
          />

          <label className="block text-sm text-zinc-300">
            <div className="flex items-center gap-3 uppercase tracking-wide">
              <input
                type="checkbox"
                className="h-5 w-5 rounded border border-zinc-700 bg-zinc-900 text-emerald-400 accent-emerald-400 focus:ring-emerald-400"
                checked={draft.autoStart}
                onChange={(event) => update({ autoStart: event.target.checked })}
              />
              Autostart camera
            </div>
            <p className="mt-2 text-xs text-zinc-500">Begin capturing automatically when the app launches.</p>
          </label>
        </div>
      </div>
    </ScreenShell>
  );
};
