<script lang="ts">
  import { onMount } from 'svelte';

  export interface Point {
    x: number; // 0 to 1
    y: number; // 0 to 1
  }

  export interface Stroke {
    color: string;
    width: number;
    points: Point[];
  }

  interface Props {
    interactive?: boolean;
    strokes?: Stroke[];
    currentColor?: string;
    currentWidth?: number;
    activeTool?: 'brush' | 'eraser';
    zoom?: number;
    panX?: number;
    panY?: number;
    onstrokeschange?: (strokes: Stroke[]) => void;
    onundo?: () => void;
    onwheel?: (e: WheelEvent) => void;
    onpanstart?: (e: PointerEvent) => void;
    onpanmove?: (e: PointerEvent) => void;
    onpanend?: () => void;
  }

  let {
    interactive = false,
    strokes = $bindable([]),
    currentColor = '#ef4444',
    currentWidth = 4,
    activeTool = 'brush',
    zoom = 1,
    panX = 0,
    panY = 0,
    onstrokeschange,
    onundo,
    onwheel,
    onpanstart,
    onpanmove,
    onpanend,
  }: Props = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let isDrawing = false;
  let isPanning = false;
  let currentStroke = $state<Stroke | null>(null);
  let pointerPoint = $state<Point | null>(null);
  let lastEraserPoint: Point | null = null;

  function getNormalizedPoint(e: PointerEvent | MouseEvent): Point | null {
    if (!canvasEl) return null;
    const rect = canvasEl.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return null;
    const currentZoom = Math.max(1, zoom);
    const x = Math.max(0, Math.min(1, (e.clientX - rect.left - panX) / (rect.width * currentZoom)));
    const y = Math.max(0, Math.min(1, (e.clientY - rect.top - panY) / (rect.height * currentZoom)));
    return { x, y };
  }

  function distToSegment(p: Point, a: Point, b: Point): number {
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const lenSq = dx * dx + dy * dy;
    if (lenSq === 0) return Math.hypot(p.x - a.x, p.y - a.y);
    let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / lenSq;
    t = Math.max(0, Math.min(1, t));
    const projX = a.x + t * dx;
    const projY = a.y + t * dy;
    return Math.hypot(p.x - projX, p.y - projY);
  }

  function strokeIntersects(stroke: Stroke, p: Point, radiusNorm: number): boolean {
    if (!stroke.points || stroke.points.length === 0) return false;
    if (stroke.points.length === 1) {
      return Math.hypot(p.x - stroke.points[0].x, p.y - stroke.points[0].y) <= radiusNorm;
    }
    for (let i = 0; i < stroke.points.length - 1; i++) {
      if (distToSegment(p, stroke.points[i], stroke.points[i + 1]) <= radiusNorm) {
        return true;
      }
    }
    return false;
  }

  function eraseAtPoint(pt: Point) {
    if (!canvasEl) return;
    const minDim = Math.min(canvasEl.width, canvasEl.height);
    if (minDim <= 0) return;
    const dpr = window.devicePixelRatio || 1;
    const radiusNorm = (currentWidth / 2) / (minDim / dpr);
    const remaining = strokes.filter(s => !strokeIntersects(s, pt, radiusNorm));
    if (remaining.length !== strokes.length) {
      strokes = remaining;
      onstrokeschange?.(strokes);
    }
  }

  function eraseContinuous(fromPt: Point, toPt: Point) {
    const dist = Math.hypot(toPt.x - fromPt.x, toPt.y - fromPt.y);
    if (!canvasEl) return;
    const dpr = window.devicePixelRatio || 1;
    const minDim = Math.min(canvasEl.width, canvasEl.height) / dpr;
    const radiusNorm = (currentWidth / 2) / (minDim > 0 ? minDim : 1);
    const step = Math.max(0.001, radiusNorm / 3);
    const steps = Math.ceil(dist / step);
    for (let i = 0; i <= steps; i++) {
      const t = steps === 0 ? 1 : i / steps;
      const interp: Point = {
        x: fromPt.x + (toPt.x - fromPt.x) * t,
        y: fromPt.y + (toPt.y - fromPt.y) * t,
      };
      eraseAtPoint(interp);
    }
  }

  function handlePointerDown(e: PointerEvent) {
    if (!interactive) return;

    // Middle button: pan (hold-to-pan)
    if (e.button === 1) {
      e.preventDefault();
      e.stopPropagation();
      isPanning = true;
      onpanstart?.(e);

      function onPanMove(ev: PointerEvent) {
        if (!isPanning) return;
        onpanmove?.(ev);
      }

      function onPanUp(ev: PointerEvent | MouseEvent) {
        if (ev.button !== 1) return;
        isPanning = false;
        onpanend?.();
        window.removeEventListener('pointermove', onPanMove);
        window.removeEventListener('pointerup', onPanUp as any);
        window.removeEventListener('pointercancel', onPanCancel);
      }

      function onPanCancel() {
        isPanning = false;
        onpanend?.();
        window.removeEventListener('pointermove', onPanMove);
        window.removeEventListener('pointerup', onPanUp as any);
        window.removeEventListener('pointercancel', onPanCancel);
      }

      window.addEventListener('pointermove', onPanMove);
      window.addEventListener('pointerup', onPanUp as any);
      window.addEventListener('pointercancel', onPanCancel);
      return;
    }

    if (e.button !== 0 && e.pointerType === 'mouse') return;

    const pt = getNormalizedPoint(e);
    if (!pt) return;

    isDrawing = true;
    (e.target as HTMLElement)?.setPointerCapture?.(e.pointerId);

    if (activeTool === 'eraser') {
      pointerPoint = pt;
      lastEraserPoint = pt;
      eraseAtPoint(pt);
    } else {
      currentStroke = {
        color: currentColor,
        width: currentWidth,
        points: [pt],
      };
    }
    drawCanvas();
  }

  function handlePointerMove(e: PointerEvent) {
    if (!interactive) return;
    if (isPanning) return;
    const pt = getNormalizedPoint(e);
    if (!pt) return;
    pointerPoint = pt;

    if (activeTool === 'eraser') {
      if (isDrawing) {
        if (lastEraserPoint) {
          eraseContinuous(lastEraserPoint, pt);
        } else {
          eraseAtPoint(pt);
        }
        lastEraserPoint = pt;
      }
      drawCanvas();
      return;
    }

    if (!isDrawing || !currentStroke) {
      drawCanvas();
      return;
    }

    if (e.shiftKey) {
      // Straight line mode from starting point
      const startPt = currentStroke.points[0];
      currentStroke.points = [startPt, pt];
    } else {
      // Polyline mode
      const lastPt = currentStroke.points[currentStroke.points.length - 1];
      if (!lastPt || Math.hypot(pt.x - lastPt.x, pt.y - lastPt.y) >= 0.0005) {
        currentStroke.points.push(pt);
      }
    }
    drawCanvas();
  }

  function handlePointerUp(e: PointerEvent) {
    if (!interactive || !isDrawing) return;
    if (e.button === 1) return;
    isDrawing = false;
    lastEraserPoint = null;

    if (activeTool === 'brush' && currentStroke && currentStroke.points.length > 0) {
      strokes = [...strokes, currentStroke];
      onstrokeschange?.(strokes);
    }
    currentStroke = null;
    drawCanvas();
  }

  function handlePointerLeave() {
    pointerPoint = null;
    lastEraserPoint = null;
    drawCanvas();
  }

  function drawCanvas() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const width = canvasEl.width;
    const height = canvasEl.height;
    const cssW = width / dpr;
    const cssH = height / dpr;

    ctx.clearRect(0, 0, width, height);
    ctx.save();
    ctx.scale(dpr, dpr);

    // Apply vector 2D scale/translation matching video pan/zoom
    ctx.translate(panX, panY);
    ctx.scale(zoom, zoom);

    const allStrokes = currentStroke ? [...strokes, currentStroke] : strokes;

    for (const stroke of allStrokes) {
      if (!stroke.points || stroke.points.length === 0) continue;

      ctx.beginPath();
      ctx.strokeStyle = stroke.color;
      ctx.lineWidth = stroke.width;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';

      if (stroke.points.length === 1) {
        const p = stroke.points[0];
        ctx.arc(p.x * cssW, p.y * cssH, stroke.width / 2, 0, Math.PI * 2);
        ctx.fillStyle = stroke.color;
        ctx.fill();
      } else {
        const p0 = stroke.points[0];
        ctx.moveTo(p0.x * cssW, p0.y * cssH);

        for (let i = 1; i < stroke.points.length; i++) {
          const p = stroke.points[i];
          ctx.lineTo(p.x * cssW, p.y * cssH);
        }
        ctx.stroke();
      }
    }

    ctx.restore();
  }

  function resizeCanvas() {
    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const newW = Math.round(rect.width * dpr);
    const newH = Math.round(rect.height * dpr);
    if (canvasEl.width !== newW || canvasEl.height !== newH) {
      canvasEl.width = newW;
      canvasEl.height = newH;
      drawCanvas();
    }
  }

  $effect(() => {
    strokes;
    interactive;
    currentColor;
    currentWidth;
    activeTool;
    zoom;
    panX;
    panY;
    drawCanvas();
  });

  onMount(() => {
    resizeCanvas();
    const ro = new ResizeObserver(() => {
      resizeCanvas();
    });
    if (canvasEl) {
      ro.observe(canvasEl);
    }

    function handleKeyDown(e: KeyboardEvent) {
      if (!interactive) return;
      const target = e.target as HTMLElement;
      const tag = target?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

      const isZKey = e.code === 'KeyZ' || e.key.toLowerCase() === 'z' || e.key.toLowerCase() === 'я';
      if ((e.ctrlKey || e.metaKey) && isZKey) {
        e.preventDefault();
        if (onundo) {
          onundo();
        } else if (strokes.length > 0) {
          strokes = strokes.slice(0, -1);
          onstrokeschange?.(strokes);
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      ro.disconnect();
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<canvas
  bind:this={canvasEl}
  class="drawing-canvas"
  class:interactive
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerUp}
  onpointerleave={handlePointerLeave}
  onwheel={onwheel}
></canvas>

{#if interactive && pointerPoint && canvasEl}
  {@const dpr = typeof window !== 'undefined' ? (window.devicePixelRatio || 1) : 1}
  {@const cssW = canvasEl.width / dpr}
  {@const cssH = canvasEl.height / dpr}
  {@const currentZoom = Math.max(1, zoom)}
  {@const cx = pointerPoint.x * cssW * currentZoom + panX}
  {@const cy = pointerPoint.y * cssH * currentZoom + panY}
  {@const size = currentWidth * currentZoom}
  <div
    class="cursor-ring"
    class:eraser={activeTool === 'eraser'}
    style:left="{cx}px"
    style:top="{cy}px"
    style:width="{size}px"
    style:height="{size}px"
    style:--draw-color={currentColor}
  ></div>
{/if}

<style>
  .drawing-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 20;
  }

  .drawing-canvas.interactive {
    pointer-events: auto;
    cursor: none;
    touch-action: none;
  }

  .cursor-ring {
    position: absolute;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    pointer-events: none;
    z-index: 25;
    box-sizing: border-box;
    border: 1.5px solid #ffffff;
    background: var(--draw-color);
    opacity: 0.45;
    box-shadow: 0 0 2px rgba(0, 0, 0, 0.6);
  }

  .cursor-ring.eraser {
    border: 1.5px solid rgba(239, 68, 68, 0.95);
    background: rgba(239, 68, 68, 0.25);
    opacity: 1;
  }
</style>
