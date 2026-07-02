import { useCallback, useMemo, useState, useEffect } from "react";
import type { Params } from "../types/params";
import { useParams } from "./useParams";
import { deepClone } from "../lib/clone";

export type SettingsDraft = {
  snapshot: Params;
  draft: Params;
  dirty: boolean;
  updateDraft: (updater: (current: Params) => Params) => void;
  resetDraft: () => void;
};

export const useSettingsDraft = (): SettingsDraft => {
  const { params } = useParams();
  const [snapshot, setSnapshot] = useState<Params>(params);
  const [draft, setDraft] = useState<Params>(params);

  useEffect(() => {
    setSnapshot(params);
    setDraft(params);
  }, [params]);

  const updateDraft = useCallback((updater: (current: Params) => Params) => {
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
