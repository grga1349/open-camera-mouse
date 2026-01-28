import {useCallback, useEffect, useMemo, useState} from 'react';
import type {AllParams} from '../types/params';
import {useAppStore} from './useAppStore';

export type SettingsDraft = {
	snapshot: AllParams;
	draft: AllParams;
	dirty: boolean;
	updateDraft: (updater: (current: AllParams) => AllParams) => void;
	resetDraft: () => void;
	saveDraft: () => void;
};

const cloneParams = (value: AllParams): AllParams => JSON.parse(JSON.stringify(value));

export const useSettingsDraft = (): SettingsDraft => {
	const {params, setParams} = useAppStore();
	const [snapshot, setSnapshot] = useState<AllParams>(params);
	const [draft, setDraft] = useState<AllParams>(params);

	useEffect(() => {
		setSnapshot(params);
		setDraft(params);
	}, [params]);

	const updateDraft = useCallback((updater: (current: AllParams) => AllParams) => {
		setDraft(prev => updater(prev));
	}, []);

	const resetDraft = useCallback(() => {
		setDraft(snapshot);
	}, [snapshot]);

	const saveDraft = useCallback(() => {
		setParams(cloneParams(draft));
	}, [draft, setParams]);

	const dirty = useMemo(() => JSON.stringify(draft) !== JSON.stringify(snapshot), [draft, snapshot]);

	return {
		snapshot,
		draft,
		dirty,
		updateDraft,
		resetDraft,
		saveDraft,
	};
};
