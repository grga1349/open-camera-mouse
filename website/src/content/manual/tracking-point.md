---
title: Selecting a tracking point
description: How to pick the spot on your face that the camera follows, which spots work well, and what the ring around it tells you.
order: 5
---

The app follows one small spot on your face, the **target**, and moves the pointer with
it. You choose that spot yourself, by clicking it in the camera preview or by
[recentering](../recentering/).

## Picking a point

When the camera is on, hover over the preview: a dashed white circle follows the mouse and
says "Click to select". Click the spot you want to track. A thin green ring marks it, and
the status changes to **Tracking**.

You can click another point at any time while the camera runs, also after the target was
lost.

Right after the click your hand is still on the mouse, so the mouse keeps the pointer for
a moment: keep it still for 4 seconds and the camera takes over. The pointer then moves to
the middle of the screen (the [main monitor](../using-the-pointer/#several-monitors)),
and from there it follows your head. See [Mouse handover](../mouse-handover/).

## Which spot works well

Pick a point with clear detail that keeps its shape when you move:

- the **tip of the nose**, which works well;
- the point **between the eyebrows**, or an eyebrow.

Avoid plain areas with little detail, such as the middle of a cheek: they are harder to
follow, and if the target is lost the app can't find it again by itself.

Keep your face roughly in the middle of the picture: near the edge, the tracker holds the
pointer until you move back.

## What the ring shows

The marker and the status line always say the same thing in words, not only in colour.

| Marker | Status | Meaning |
|---|---|---|
| Green ring | Tracking | The pointer follows your head. With dwell click on, a thin arc around the ring fills up as you hold still. |
| Dashed green ring, "Holding" | Holding · moved too fast | You moved faster than the tracker can follow. The pointer waits; move a little slower and tracking continues by itself. |
| Dashed green ring, "Holding" | Holding · near the edge | The target is close to the edge of the picture and moving out. Move back toward the middle. |
| Grey ring with a cross, "Lost" | Target lost | The app no longer sees the point. The mouse has the pointer. |

The status line also names who moves the pointer: "Pointer: camera", "held still",
"mouse", "needs permission" or "not available".

## When the target is lost

By default, a lost target stays lost until you act: click the same point on your face
again, or press **Recenter** (<kbd>F12</kbd>). Until then you have the mouse.

With **Find the target again** on (**Options → Camera**, off by default), the app looks
for the point by itself. The ring grows and fades, the status reads **Looking for the
target…**, and the mouse keeps control. Face the camera; when the point is found, tracking
continues. If the pointer would have to jump far to its new position, the camera only
takes over after the mouse has been still for 4 seconds.

If the point you picked has too little detail to be found again, the status says so and
asks you to click a point again; the tip of the nose or an eyebrow works better.
