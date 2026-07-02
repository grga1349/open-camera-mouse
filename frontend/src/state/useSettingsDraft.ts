import { useCallback, useEffect, useMemo, useState } from "react";
import type { AllParams } from "../types/params";
import { useParams } from "./useParams";
import { deepClone } from "../lib/clone";

export type SettingsDraft = {
  snapshot: AllParams;
  draft: AllParams;
  dirty: boolean;
  updateDraft: (updater: (current: AllParams) => AllParams) => void;
  resetDraft: () => void;
};

export const useSettingsDraft = (): SettingsDraft => {
  const { params } = useParams();
  const [snapshot, setSnapshot] = useState<AllParams>(params);
  const [draft, setDraft] = useState<AllParams>(params);

  useEffect(() => {
    setSnapshot(params);
    setDraft(params);
  }, [params]);

  const updateDraft = useCallback((updater: (current: AllParams) => AllParams) => {
    setDraft((prev) => updater(prev));
  }, []);

  const resetDraft = useCallback(() => {
    setDraft(deepClone(snapshot));
  }, [snapshot]);

  const dirty = useMemo(() => JSON.stringify(draft) !== JSON.stringify(snapshot), [draft, snapshot]);

  return {
    snapshot,
    draft,
    dirty,
    updateDraft,
    resetDraft,
  };
};
