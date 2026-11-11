// The camera preview container is a fixed size by design (not resizable),
// so both the click-to-source mapping (usePickPoint) and the overlay
// drawing (CameraPreview) must use these exact same numbers rather than
// each independently measuring/declaring the container size — any drift
// between the two is amplified by the cover-transform scale factor and
// throws off where the tracking box appears relative to where you clicked.
export const PREVIEW_WIDTH_PX = 360;
export const PREVIEW_HEIGHT_PX = 270;
