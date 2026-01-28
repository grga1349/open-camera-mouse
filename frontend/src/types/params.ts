export type MarkerShape = 'circle' | 'square';
export type ClickType = 'left' | 'right' | 'double';

export type TrackingParams = {
    templateSizePx: number;
    searchMarginPx: number;
    scoreThreshold: number;
    adaptiveTemplate: boolean;
    templateUpdateAlpha: number;
    markerShape: MarkerShape;
};

export type PointerAdvancedParams = {
    gainX: number;
    gainY: number;
    smoothing: number;
};

export type PointerParams = {
    sensitivity: number;
    deadzonePx: number;
    maxSpeedPx: number;
    advanced: PointerAdvancedParams | null;
};

export type ClickingParams = {
    dwellEnabled: boolean;
    dwellTimeMs: number;
    dwellRadiusPx: number;
    clickType: ClickType;
    rightClickToggle: boolean;
};

export type AllParams = {
    tracking: TrackingParams;
    pointer: PointerParams;
    clicking: ClickingParams;
};
