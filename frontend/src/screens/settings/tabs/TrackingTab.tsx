import type {FC} from 'react';
import {ChoiceButton} from '../../../components/inputs/ChoiceButton';
import {SliderField} from '../../../components/inputs/SliderField';
import type {TabProps} from './types';

const TEMPLATE_SIZES = [30, 40, 50];
const MARKER_SHAPES = ['circle', 'square'] as const;

export const TrackingTab: FC<TabProps> = ({draft, updateDraft}) => {
    const tracking = draft.tracking;
    const updateTracking = (changes: Partial<typeof tracking>) => {
        updateDraft(current => ({
            ...current,
            tracking: {
                ...current.tracking,
                ...changes,
            },
        }));
    };

    return (
        <div className="space-y-4">
            <div>
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Template size</p>
                <div className="flex gap-2">
                    {TEMPLATE_SIZES.map(size => (
                        <ChoiceButton
                            key={size}
                            selected={tracking.templateSizePx === size}
                            onClick={() => updateTracking({templateSizePx: size})}
                        >
                            {size}px
                        </ChoiceButton>
                    ))}
                </div>
            </div>

            <SliderField
                label="Search margin"
                min={10}
                max={120}
                step={5}
                value={tracking.searchMarginPx}
                onChange={value => updateTracking({searchMarginPx: value})}
            />

            <SliderField
                label={`Score threshold (${tracking.scoreThreshold.toFixed(2)})`}
                min={30}
                max={95}
                step={1}
                value={Math.round(tracking.scoreThreshold * 100)}
                onChange={value => updateTracking({scoreThreshold: value / 100})}
            />

            <label className="flex items-center gap-3 text-sm uppercase tracking-wide">
                <input
                    type="checkbox"
                    className="h-4 w-4 rounded border border-zinc-700 bg-zinc-900 text-emerald-400 accent-emerald-400 focus:ring-emerald-400"
                    checked={tracking.adaptiveTemplate}
                    onChange={event => updateTracking({adaptiveTemplate: event.target.checked})}
                />
                Adaptive template
            </label>

            <SliderField
                label={`Template alpha (${tracking.templateUpdateAlpha.toFixed(2)})`}
                min={0}
                max={100}
                step={5}
                value={Math.round(tracking.templateUpdateAlpha * 100)}
                disabled={!tracking.adaptiveTemplate}
                onChange={value => updateTracking({templateUpdateAlpha: value / 100})}
            />

            <div>
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">Marker shape</p>
                <div className="flex gap-2">
                    {MARKER_SHAPES.map(shape => (
                        <ChoiceButton
                            key={shape}
                            selected={tracking.markerShape === shape}
                            onClick={() => updateTracking({markerShape: shape as typeof tracking.markerShape})}
                        >
                            {shape.toUpperCase()}
                        </ChoiceButton>
                    ))}
                </div>
            </div>
        </div>
    );
};

