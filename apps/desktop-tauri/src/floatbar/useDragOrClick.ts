import { useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const DRAG_THRESHOLD_PX = 5;

export function useDragOrClick() {
  const startPointRef = useRef<{ x: number; y: number } | null>(null);
  const draggedRef = useRef(false);

  const onMouseDown = useCallback((event: React.MouseEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return;
    }

    startPointRef.current = { x: event.clientX, y: event.clientY };
    draggedRef.current = false;
  }, []);

  const onMouseMove = useCallback((event: React.MouseEvent<HTMLElement>) => {
    const startPoint = startPointRef.current;
    if (!startPoint || draggedRef.current) {
      return;
    }

    const distance = Math.hypot(event.clientX - startPoint.x, event.clientY - startPoint.y);
    if (distance < DRAG_THRESHOLD_PX) {
      return;
    }

    draggedRef.current = true;
    void getCurrentWindow()
      .startDragging()
      .catch(() => {});
  }, []);

  const onMouseUp = useCallback((event: React.MouseEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return;
    }

    const shouldToggle = startPointRef.current !== null && !draggedRef.current;
    startPointRef.current = null;
    draggedRef.current = false;

    if (shouldToggle) {
      void invoke("toggle_detail").catch(() => {});
    }
  }, []);

  return { onMouseDown, onMouseMove, onMouseUp };
}
