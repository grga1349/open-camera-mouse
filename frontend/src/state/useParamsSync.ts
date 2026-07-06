import { useCallback } from "react";
import { GetParams, UpdateParams } from "../../wailsjs/go/main/App";
import { useParams } from "./useParams";
import { useAppError } from "./useAppError";
import { fromBackendParams, toBackendParams } from "../lib/params";
import type { Params } from "../types/params";

const commitParams = async (next: Params): Promise<Params> => {
  await UpdateParams(toBackendParams(next));
  const saved = await GetParams();
  return fromBackendParams(saved);
};

/**
 * Two ways to push a Params change to the backend, both refetching the
 * canonical saved value from Go afterward so the frontend never drifts from
 * what's actually on disk:
 *
 * - confirmParams: no optimistic update — the caller (e.g. Settings) is
 *   already showing its own draft and only wants global params to move once
 *   the backend confirms the save. Throws on failure so the caller can keep
 *   its draft/dirty state intact.
 * - setParamsOptimistic: updates global params immediately, then rolls back
 *   to the previous value if the backend save fails. Used for single-field
 *   toggles (e.g. dwell) where instant feedback matters more than waiting.
 *
 * Both report failures to the shared app-level error banner.
 */
export const useParamsSync = () => {
  const { params, setParams } = useParams();
  const { reportError, clearError } = useAppError();

  const confirmParams = useCallback(
    async (next: Params, errorMessage = "Settings could not be saved.") => {
      try {
        const saved = await commitParams(next);
        setParams(saved);
        clearError();
      } catch (err) {
        reportError(errorMessage);
        throw err;
      }
    },
    [setParams, clearError, reportError],
  );

  const setParamsOptimistic = useCallback(
    async (next: Params, errorMessage: string) => {
      const previous = params;
      setParams(next);
      try {
        const saved = await commitParams(next);
        setParams(saved);
        clearError();
      } catch (err) {
        setParams(previous);
        reportError(errorMessage);
        throw err;
      }
    },
    [params, setParams, reportError, clearError],
  );

  return { confirmParams, setParamsOptimistic };
};
