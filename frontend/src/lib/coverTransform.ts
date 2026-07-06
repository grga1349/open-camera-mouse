export type CoverTransform = {
  scale: number;
  offsetX: number;
  offsetY: number;
};

/**
 * Maps source-image pixel space to on-screen container pixel space for an
 * element rendered with CSS `object-fit: cover` — the larger of the two
 * axis scales wins (so the image fully covers the container), and the
 * other axis is cropped symmetrically, hence the offset.
 */
export const computeCoverTransform = (
  containerWidth: number,
  containerHeight: number,
  sourceWidth: number,
  sourceHeight: number,
): CoverTransform | null => {
  if (containerWidth <= 0 || containerHeight <= 0 || sourceWidth <= 0 || sourceHeight <= 0) {
    return null;
  }
  const scale = Math.max(containerWidth / sourceWidth, containerHeight / sourceHeight);
  return {
    scale,
    offsetX: (sourceWidth * scale - containerWidth) / 2,
    offsetY: (sourceHeight * scale - containerHeight) / 2,
  };
};

export const sourceToContainerPoint = (transform: CoverTransform, x: number, y: number) => ({
  x: x * transform.scale - transform.offsetX,
  y: y * transform.scale - transform.offsetY,
});

export const containerToSourcePoint = (transform: CoverTransform, x: number, y: number) => ({
  x: (x + transform.offsetX) / transform.scale,
  y: (y + transform.offsetY) / transform.scale,
});
