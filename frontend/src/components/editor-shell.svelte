<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../services/api-client';

  type EditorTool = 'brush' | 'eraser' | 'heal' | 'select';

  interface SelectionRect {
    x: number;
    y: number;
    width: number;
    height: number;
  }

  let canvasEl: HTMLCanvasElement;
  let hydrated = false;

  let initialImageData: ImageData | null = null;
  let workingImageData: ImageData | null = null;

  let undoStack: ImageData[] = [];
  let redoStack: ImageData[] = [];

  let activeFileName = 'Untitled image';
  let activeTool: EditorTool = 'brush';
  let brushSize = 18;
  let brushColor = '#ff2d55';

  let blackPoint = 0;
  let whitePoint = 255;
  let hueShift = 0;
  let saturationPercent = 100;
  let blurRadius = 0;
  let isRemovingBackground = false;
  let backgroundError: string | null = null;
  let backgroundStatus: string | null = null;

  let isDrawing = false;
  let selectionStart: { x: number; y: number } | null = null;
  let selectionRect: SelectionRect | null = null;
  type InspectorTab = 'properties' | 'adjustments' | 'layers';
  let inspectorTab: InspectorTab = 'adjustments';

  const selectionStroke = '#00a8ff';

  onMount(() => {
    hydrated = true;

    const onShortcut = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const isTypingField =
        target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || Boolean(target?.isContentEditable);
      const usesModifier = event.metaKey || event.ctrlKey;
      const key = event.key.toLowerCase();

      if (usesModifier && key === 'z') {
        event.preventDefault();
        if (event.shiftKey) {
          redo();
        } else {
          undo();
        }
        return;
      }

      if (usesModifier && key === 'y') {
        event.preventDefault();
        redo();
        return;
      }

      if (usesModifier && key === 'e') {
        event.preventDefault();
        exportImage();
        return;
      }

      if (isTypingField || usesModifier) {
        return;
      }

      if (key === 'b') {
        selectTool('brush');
      } else if (key === 'e') {
        selectTool('eraser');
      } else if (key === 'h') {
        selectTool('heal');
      } else if (key === 's') {
        selectTool('select');
      } else if (key === '[') {
        brushSize = clamp(brushSize - 2, 2, 80);
      } else if (key === ']') {
        brushSize = clamp(brushSize + 2, 2, 80);
      }
    };

    window.addEventListener('keydown', onShortcut);
    return () => window.removeEventListener('keydown', onShortcut);
  });

  $: hasImage = Boolean(workingImageData);

  function index(width: number, x: number, y: number, channel: number): number {
    return (y * width + x) * 4 + channel;
  }

  function readChannel(data: Uint8ClampedArray, offset: number): number {
    return data[offset] ?? 0;
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
  }

  function cloneImageData(imageData: ImageData): ImageData {
    return new ImageData(Uint8ClampedArray.from(imageData.data), imageData.width, imageData.height);
  }

  function drawEditorSurface(): void {
    if (!canvasEl || !workingImageData) {
      return;
    }

    const context = canvasEl.getContext('2d');
    if (!context) {
      return;
    }

    context.putImageData(workingImageData, 0, 0);

    if (selectionRect) {
      context.save();
      context.strokeStyle = selectionStroke;
      context.lineWidth = Math.max(1, Math.round(canvasEl.width / 240));
      context.setLineDash([6, 4]);
      context.strokeRect(selectionRect.x + 0.5, selectionRect.y + 0.5, selectionRect.width, selectionRect.height);
      context.restore();
    }
  }

  function pushUndoSnapshot(): void {
    if (!workingImageData) {
      return;
    }

    undoStack = [...undoStack, cloneImageData(workingImageData)];
    redoStack = [];
  }

  function applyNewWorkingImage(nextImage: ImageData): void {
    workingImageData = nextImage;
    canvasEl.width = nextImage.width;
    canvasEl.height = nextImage.height;
    drawEditorSurface();
  }

  function undo(): void {
    if (!workingImageData || undoStack.length === 0) {
      return;
    }

    const previous = undoStack[undoStack.length - 1];
    undoStack = undoStack.slice(0, -1);
    redoStack = [...redoStack, cloneImageData(workingImageData)];

    if (!previous) {
      return;
    }

    applyNewWorkingImage(previous);
  }

  function redo(): void {
    if (!workingImageData || redoStack.length === 0) {
      return;
    }

    const next = redoStack[redoStack.length - 1];
    redoStack = redoStack.slice(0, -1);
    undoStack = [...undoStack, cloneImageData(workingImageData)];

    if (!next) {
      return;
    }

    applyNewWorkingImage(next);
  }

  function wrapHue(value: number): number {
    let next = value % 360;
    if (next < 0) {
      next += 360;
    }
    return next;
  }

  function rgbToHsv(red: number, green: number, blue: number): [number, number, number] {
    const r = red / 255;
    const g = green / 255;
    const b = blue / 255;
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const delta = max - min;

    let hue = 0;
    if (delta !== 0) {
      if (max === r) {
        hue = 60 * (((g - b) / delta) % 6);
      } else if (max === g) {
        hue = 60 * ((b - r) / delta + 2);
      } else {
        hue = 60 * ((r - g) / delta + 4);
      }
    }

    const saturation = max === 0 ? 0 : delta / max;
    return [wrapHue(hue), saturation, max];
  }

  function hsvToRgb(hue: number, saturation: number, value: number): [number, number, number] {
    const chroma = value * saturation;
    const segment = hue / 60;
    const second = chroma * (1 - Math.abs((segment % 2) - 1));
    const match = value - chroma;

    let r1 = 0;
    let g1 = 0;
    let b1 = 0;

    if (segment >= 0 && segment < 1) {
      r1 = chroma;
      g1 = second;
    } else if (segment >= 1 && segment < 2) {
      r1 = second;
      g1 = chroma;
    } else if (segment >= 2 && segment < 3) {
      g1 = chroma;
      b1 = second;
    } else if (segment >= 3 && segment < 4) {
      g1 = second;
      b1 = chroma;
    } else if (segment >= 4 && segment < 5) {
      r1 = second;
      b1 = chroma;
    } else {
      r1 = chroma;
      b1 = second;
    }

    return [
      Math.round((r1 + match) * 255),
      Math.round((g1 + match) * 255),
      Math.round((b1 + match) * 255)
    ];
  }

  function applyLevels(pixels: Uint8ClampedArray, low: number, high: number): Uint8ClampedArray {
    if (low >= high) {
      return Uint8ClampedArray.from(pixels);
    }

    const output = Uint8ClampedArray.from(pixels);
    const range = Math.max(1, high - low);

    for (let pixelOffset = 0; pixelOffset < output.length; pixelOffset += 4) {
      for (let channel = 0; channel < 3; channel += 1) {
        const value = readChannel(output, pixelOffset + channel);
        const normalized = clamp((value - low) / range, 0, 1);
        output[pixelOffset + channel] = Math.round(normalized * 255);
      }
    }

    return output;
  }

  function applyHueSaturation(
    pixels: Uint8ClampedArray,
    shiftDegrees: number,
    saturationScale: number
  ): Uint8ClampedArray {
    const output = Uint8ClampedArray.from(pixels);

    for (let pixelOffset = 0; pixelOffset < output.length; pixelOffset += 4) {
      const [hue, saturation, value] = rgbToHsv(
        readChannel(output, pixelOffset),
        readChannel(output, pixelOffset + 1),
        readChannel(output, pixelOffset + 2)
      );

      const [red, green, blue] = hsvToRgb(
        wrapHue(hue + shiftDegrees),
        clamp(saturation * saturationScale, 0, 1),
        value
      );

      output[pixelOffset] = red;
      output[pixelOffset + 1] = green;
      output[pixelOffset + 2] = blue;
    }

    return output;
  }

  function applyBoxBlur(
    pixels: Uint8ClampedArray,
    width: number,
    height: number,
    radius: number
  ): Uint8ClampedArray {
    if (radius <= 0) {
      return Uint8ClampedArray.from(pixels);
    }

    const horizontal = new Uint8ClampedArray(pixels.length);
    const output = new Uint8ClampedArray(pixels.length);

    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        for (let channel = 0; channel < 4; channel += 1) {
          let total = 0;
          let count = 0;

          for (let sampleX = x - radius; sampleX <= x + radius; sampleX += 1) {
            const clampedX = clamp(sampleX, 0, width - 1);
            total += readChannel(pixels, index(width, clampedX, y, channel));
            count += 1;
          }

          horizontal[index(width, x, y, channel)] = Math.round(total / count);
        }
      }
    }

    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        for (let channel = 0; channel < 4; channel += 1) {
          let total = 0;
          let count = 0;

          for (let sampleY = y - radius; sampleY <= y + radius; sampleY += 1) {
            const clampedY = clamp(sampleY, 0, height - 1);
            total += readChannel(horizontal, index(width, x, clampedY, channel));
            count += 1;
          }

          output[index(width, x, y, channel)] = Math.round(total / count);
        }
      }
    }

    return output;
  }

  function hexToRgb(hexColor: string): [number, number, number] {
    const normalized = hexColor.replace('#', '').trim();
    const safeHex = normalized.length === 6 ? normalized : 'ff2d55';
    return [
      Number.parseInt(safeHex.slice(0, 2), 16),
      Number.parseInt(safeHex.slice(2, 4), 16),
      Number.parseInt(safeHex.slice(4, 6), 16)
    ];
  }

  function toCanvasPoint(event: PointerEvent): { x: number; y: number } {
    const bounds = canvasEl.getBoundingClientRect();
    const scaleX = canvasEl.width / Math.max(1, bounds.width);
    const scaleY = canvasEl.height / Math.max(1, bounds.height);

    const x = clamp(Math.floor((event.clientX - bounds.left) * scaleX), 0, Math.max(0, canvasEl.width - 1));
    const y = clamp(Math.floor((event.clientY - bounds.top) * scaleY), 0, Math.max(0, canvasEl.height - 1));
    return { x, y };
  }

  function paintAtPoint(point: { x: number; y: number }): void {
    if (!workingImageData) {
      return;
    }

    const data = workingImageData.data;
    const width = workingImageData.width;
    const height = workingImageData.height;
    const [red, green, blue] = hexToRgb(brushColor);
    const radius = Math.max(1, Math.floor(brushSize / 2));

    for (let y = point.y - radius; y <= point.y + radius; y += 1) {
      for (let x = point.x - radius; x <= point.x + radius; x += 1) {
        if (x < 0 || y < 0 || x >= width || y >= height) {
          continue;
        }

        const distance = Math.hypot(x - point.x, y - point.y);
        if (distance > radius) {
          continue;
        }

        const offset = index(width, x, y, 0);

        if (activeTool === 'brush') {
          data[offset] = red;
          data[offset + 1] = green;
          data[offset + 2] = blue;
          data[offset + 3] = 255;
        } else if (activeTool === 'eraser') {
          data[offset + 3] = 0;
        } else if (activeTool === 'heal') {
          const ringRadius = Math.max(3, radius * 2);
          let sampleRed = 0;
          let sampleGreen = 0;
          let sampleBlue = 0;
          let sampleAlpha = 0;
          let samples = 0;

          for (let ringY = y - ringRadius; ringY <= y + ringRadius; ringY += 1) {
            for (let ringX = x - ringRadius; ringX <= x + ringRadius; ringX += 1) {
              if (ringX < 0 || ringY < 0 || ringX >= width || ringY >= height) {
                continue;
              }

              const ringDistance = Math.hypot(ringX - x, ringY - y);
              if (ringDistance < radius || ringDistance > ringRadius) {
                continue;
              }

              const ringOffset = index(width, ringX, ringY, 0);
              sampleRed += readChannel(data, ringOffset);
              sampleGreen += readChannel(data, ringOffset + 1);
              sampleBlue += readChannel(data, ringOffset + 2);
              sampleAlpha += readChannel(data, ringOffset + 3);
              samples += 1;
            }
          }

          if (samples > 0) {
            data[offset] = Math.round(sampleRed / samples);
            data[offset + 1] = Math.round(sampleGreen / samples);
            data[offset + 2] = Math.round(sampleBlue / samples);
            data[offset + 3] = Math.round(sampleAlpha / samples);
          }
        }
      }
    }

    drawEditorSurface();
  }

  function normalizedSelection(
    start: { x: number; y: number },
    end: { x: number; y: number }
  ): SelectionRect | null {
    const x = Math.min(start.x, end.x);
    const y = Math.min(start.y, end.y);
    const width = Math.abs(start.x - end.x);
    const height = Math.abs(start.y - end.y);

    if (width < 2 || height < 2) {
      return null;
    }

    return { x, y, width, height };
  }

  function handlePointerDown(event: PointerEvent): void {
    if (!workingImageData || !canvasEl) {
      return;
    }

    canvasEl.setPointerCapture(event.pointerId);
    const point = toCanvasPoint(event);

    if (activeTool === 'select') {
      selectionStart = point;
      selectionRect = null;
      drawEditorSurface();
      return;
    }

    pushUndoSnapshot();
    isDrawing = true;
    paintAtPoint(point);
  }

  function handlePointerMove(event: PointerEvent): void {
    if (!workingImageData) {
      return;
    }

    const point = toCanvasPoint(event);

    if (activeTool === 'select' && selectionStart) {
      selectionRect = normalizedSelection(selectionStart, point);
      drawEditorSurface();
      return;
    }

    if (isDrawing && (activeTool === 'brush' || activeTool === 'eraser' || activeTool === 'heal')) {
      paintAtPoint(point);
    }
  }

  function handlePointerUp(event: PointerEvent): void {
    if (canvasEl.hasPointerCapture(event.pointerId)) {
      canvasEl.releasePointerCapture(event.pointerId);
    }

    isDrawing = false;

    if (activeTool === 'select') {
      selectionStart = null;
    }
  }

  function selectTool(tool: EditorTool): void {
    activeTool = tool;
    if (tool !== 'select') {
      selectionRect = null;
      drawEditorSurface();
    }
  }

  function applyAdjustments(): void {
    if (!workingImageData) {
      return;
    }

    pushUndoSnapshot();

    const leveled = applyLevels(workingImageData.data, blackPoint, whitePoint);
    const adjusted = applyHueSaturation(leveled, hueShift, saturationPercent / 100);
    const blurred = applyBoxBlur(adjusted, workingImageData.width, workingImageData.height, blurRadius);

    workingImageData = new ImageData(Uint8ClampedArray.from(blurred), workingImageData.width, workingImageData.height);
    selectionRect = null;
    drawEditorSurface();
  }

  async function decodeImageFromFile(file: File): Promise<HTMLImageElement> {
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject(new Error('Failed to read image file'));
      reader.readAsDataURL(file);
    });

    return decodeImageFromDataUrl(dataUrl);
  }

  async function decodeImageFromDataUrl(dataUrl: string): Promise<HTMLImageElement> {
    const image = new Image();
    await new Promise<void>((resolve, reject) => {
      image.onload = () => resolve();
      image.onerror = () => reject(new Error('Failed to decode image'));
      image.src = dataUrl;
    });

    return image;
  }

  async function removeBackground(): Promise<void> {
    if (!canvasEl || !workingImageData || isRemovingBackground) {
      return;
    }

    const encodedImage = canvasEl.toDataURL('image/png').split(',')[1];
    if (!encodedImage) {
      backgroundError = 'Could not serialize the current image for local AI processing.';
      return;
    }

    isRemovingBackground = true;
    backgroundError = null;
    backgroundStatus = null;

    try {
      const result = await apiClient.removeBackground(encodedImage);
      const decoded = await decodeImageFromDataUrl(`data:image/png;base64,${result.image_base64}`);
      const width = Math.max(1, decoded.naturalWidth);
      const height = Math.max(1, decoded.naturalHeight);

      const context = canvasEl.getContext('2d');
      if (!context) {
        throw new Error('Canvas context is unavailable');
      }

      pushUndoSnapshot();
      canvasEl.width = width;
      canvasEl.height = height;
      context.drawImage(decoded, 0, 0, width, height);

      workingImageData = context.getImageData(0, 0, width, height);
      selectionRect = null;
      drawEditorSurface();

      backgroundStatus = `Background removed locally (${result.provider}) in ${result.processing_ms}ms.`;
    } catch (error) {
      backgroundError =
        error instanceof Error
          ? error.message
          : 'Local AI background removal failed. Ensure the AI service is running on this machine.';
    } finally {
      isRemovingBackground = false;
    }
  }

  async function importImage(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];

    if (!file || !canvasEl) {
      return;
    }

    activeFileName = file.name;
    try {
      let source: CanvasImageSource;
      let width = 0;
      let height = 0;

      try {
        const bitmap = await createImageBitmap(file);
        source = bitmap;
        width = bitmap.width;
        height = bitmap.height;
      } catch {
        const image = await decodeImageFromFile(file);
        source = image;
        width = image.naturalWidth;
        height = image.naturalHeight;
      }

      width = Math.max(1, width);
      height = Math.max(1, height);

      canvasEl.width = width;
      canvasEl.height = height;

      const context = canvasEl.getContext('2d');
      if (!context) {
        return;
      }

      context.drawImage(source, 0, 0, width, height);
      const loaded = context.getImageData(0, 0, width, height);

      initialImageData = cloneImageData(loaded);
      workingImageData = loaded;
      undoStack = [];
      redoStack = [];
      selectionRect = null;
      backgroundError = null;
      backgroundStatus = null;
      resetControls();
      drawEditorSurface();
    } finally {
      input.value = '';
    }
  }

  function resetControls(): void {
    blackPoint = 0;
    whitePoint = 255;
    hueShift = 0;
    saturationPercent = 100;
    blurRadius = 0;
    brushSize = 18;
  }

  function resetImage(): void {
    if (!initialImageData) {
      return;
    }

    workingImageData = cloneImageData(initialImageData);
    undoStack = [];
    redoStack = [];
    selectionRect = null;
    backgroundError = null;
    backgroundStatus = null;
    resetControls();
    drawEditorSurface();
  }

  function cropSelection(): void {
    if (!workingImageData || !selectionRect) {
      return;
    }

    pushUndoSnapshot();

    const width = Math.max(1, Math.floor(selectionRect.width));
    const height = Math.max(1, Math.floor(selectionRect.height));
    const source = workingImageData.data;
    const sourceWidth = workingImageData.width;
    const cropped = new Uint8ClampedArray(width * height * 4);

    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        const sourceX = clamp(selectionRect.x + x, 0, workingImageData.width - 1);
        const sourceY = clamp(selectionRect.y + y, 0, workingImageData.height - 1);

        const sourceOffset = index(sourceWidth, sourceX, sourceY, 0);
        const destinationOffset = index(width, x, y, 0);

        cropped[destinationOffset] = readChannel(source, sourceOffset);
        cropped[destinationOffset + 1] = readChannel(source, sourceOffset + 1);
        cropped[destinationOffset + 2] = readChannel(source, sourceOffset + 2);
        cropped[destinationOffset + 3] = readChannel(source, sourceOffset + 3);
      }
    }

    const next = new ImageData(Uint8ClampedArray.from(cropped), width, height);
    workingImageData = next;
    initialImageData = cloneImageData(next);
    selectionRect = null;
    drawEditorSurface();
  }

  function exportImage(): void {
    if (!canvasEl || !workingImageData) {
      return;
    }

    const link = document.createElement('a');
    const stem = activeFileName.replace(/\.[^.]+$/, '') || 'edited-image';
    link.download = `${stem}-edited.png`;
    link.href = canvasEl.toDataURL('image/png');
    link.click();
  }
</script>

<div class="page">
  <main class="workspace" aria-live="polite" data-hydrated={hydrated ? 'true' : 'false'}>
    <header class="command-bar">
      <div class="brand">
        <p class="badge">PixelForge Editor</p>
        <h1>Import, retouch, add, remove, and crop.</h1>
        <p class="subtitle">Photoshop-familiar controls, simplified for speed.</p>
      </div>

      <div class="command-groups">
        <div class="command-group">
          <label for="import-image" class="button command-primary">Import image</label>
          <input id="import-image" type="file" accept="image/*" onchange={importImage} />
          <button
            class="button button-muted"
            aria-label="Remove background (local AI)"
            onclick={removeBackground}
            disabled={!hasImage || isRemovingBackground}
          >
            {isRemovingBackground ? 'Removing...' : 'Remove background (local AI)'}
          </button>
        </div>

        <div class="command-group">
          <button class="button button-muted" aria-label="Undo" onclick={undo} disabled={undoStack.length === 0}>Undo</button>
          <button class="button button-muted" aria-label="Redo" onclick={redo} disabled={redoStack.length === 0}>Redo</button>
          <button class="button button-muted" aria-label="Reset" onclick={resetImage} disabled={!hasImage}>Reset</button>
          <button class="button command-primary" aria-label="Export PNG" onclick={exportImage} disabled={!hasImage}>
            Export PNG
          </button>
        </div>
      </div>

      {#if backgroundError}
        <p class="status error" role="alert">{backgroundError}</p>
      {:else if backgroundStatus}
        <p class="status" role="status" aria-live="polite">{backgroundStatus}</p>
      {/if}
    </header>

    <section class="editor-shell">
      <aside class="panel tool-rail" aria-label="Tool rail">
        <div class="panel-title-row">
          <h2>Tools</h2>
          <p>B, E, H, S</p>
        </div>

        <div class="tool-stack">
          <button
            class="tool-button"
            class:active={activeTool === 'brush'}
            aria-label="Brush"
            aria-pressed={activeTool === 'brush'}
            onclick={() => selectTool('brush')}
          >
            <span class="tool-glyph" aria-hidden="true">BR</span>
            <span class="tool-name">Brush</span>
            <span class="tool-shortcut" aria-hidden="true">B</span>
          </button>

          <button
            class="tool-button"
            class:active={activeTool === 'eraser'}
            aria-label="Eraser"
            aria-pressed={activeTool === 'eraser'}
            onclick={() => selectTool('eraser')}
          >
            <span class="tool-glyph" aria-hidden="true">ER</span>
            <span class="tool-name">Eraser</span>
            <span class="tool-shortcut" aria-hidden="true">E</span>
          </button>

          <button
            class="tool-button"
            class:active={activeTool === 'heal'}
            aria-label="Spot remove"
            aria-pressed={activeTool === 'heal'}
            onclick={() => selectTool('heal')}
          >
            <span class="tool-glyph" aria-hidden="true">HL</span>
            <span class="tool-name">Spot remove</span>
            <span class="tool-shortcut" aria-hidden="true">H</span>
          </button>

          <button
            class="tool-button"
            class:active={activeTool === 'select'}
            aria-label="Select"
            aria-pressed={activeTool === 'select'}
            onclick={() => selectTool('select')}
          >
            <span class="tool-glyph" aria-hidden="true">SL</span>
            <span class="tool-name">Select</span>
            <span class="tool-shortcut" aria-hidden="true">S</span>
          </button>
        </div>

        <div class="control-group compact">
          <label for="brush-size">Brush size: {brushSize}px</label>
          <input
            id="brush-size"
            type="range"
            min="2"
            max="80"
            bind:value={brushSize}
            disabled={!hasImage}
          />
        </div>

        <div class="control-group compact">
          <label for="brush-color">Brush color</label>
          <input
            id="brush-color"
            type="color"
            bind:value={brushColor}
            disabled={!hasImage || activeTool !== 'brush'}
          />
        </div>

        <button class="button button-muted" aria-label="Crop selection" onclick={cropSelection} disabled={!selectionRect}>
          Crop selection
        </button>
      </aside>

      <div class="panel canvas-panel">
        <div class="canvas-header">
          <p class="canvas-title">{activeFileName}</p>
          <p class="canvas-meta">{hasImage ? `${workingImageData?.width ?? 0} x ${workingImageData?.height ?? 0} px` : 'No image loaded yet'}</p>
        </div>

        {#if !hasImage}
          <div class="empty-state">
            <h2>Start with a picture</h2>
            <p>
              Use import, then paint to add, erase to remove, spot remove to heal, and select/crop to frame.
            </p>
            <p class="shortcut-line">
              Speed tip: B Brush, E Eraser, H Spot remove, S Select, Cmd/Ctrl+Z Undo.
            </p>
          </div>
        {/if}

        <div class="canvas-stage">
          <canvas
            bind:this={canvasEl}
            aria-label="Image editor canvas"
            onpointerdown={handlePointerDown}
            onpointermove={handlePointerMove}
            onpointerup={handlePointerUp}
            onpointerleave={handlePointerUp}
          ></canvas>
        </div>
      </div>

      <aside class="panel inspector" aria-label="Inspector panel">
        <div class="inspector-tabs" role="tablist" aria-label="Inspector tabs">
          <button
            class="tab-button"
            class:active={inspectorTab === 'properties'}
            role="tab"
            aria-selected={inspectorTab === 'properties'}
            onclick={() => (inspectorTab = 'properties')}
          >
            Properties
          </button>
          <button
            class="tab-button"
            class:active={inspectorTab === 'adjustments'}
            role="tab"
            aria-selected={inspectorTab === 'adjustments'}
            onclick={() => (inspectorTab = 'adjustments')}
          >
            Adjustments
          </button>
          <button
            class="tab-button"
            class:active={inspectorTab === 'layers'}
            role="tab"
            aria-selected={inspectorTab === 'layers'}
            onclick={() => (inspectorTab = 'layers')}
          >
            Layers
          </button>
        </div>

        {#if inspectorTab === 'properties'}
          <div class="inspector-content">
            <div class="stat-grid">
              <article class="stat-card">
                <h3>Document</h3>
                <p>{activeFileName}</p>
              </article>
              <article class="stat-card">
                <h3>Canvas</h3>
                <p>{hasImage ? `${workingImageData?.width ?? 0} x ${workingImageData?.height ?? 0} px` : 'No image'}</p>
              </article>
              <article class="stat-card">
                <h3>Tool</h3>
                <p>{activeTool}</p>
              </article>
              <article class="stat-card">
                <h3>History</h3>
                <p>{undoStack.length} undo / {redoStack.length} redo</p>
              </article>
            </div>
            <p class="hint">Fast flow: Cmd/Ctrl+Z undo, Shift+Cmd/Ctrl+Z redo, Cmd/Ctrl+E export.</p>
          </div>
        {:else if inspectorTab === 'adjustments'}
          <div class="inspector-content">
            <h3>Adjustments</h3>

            <div class="control-group">
              <label for="black-point">Black point: {blackPoint}</label>
              <input id="black-point" type="range" min="0" max="254" bind:value={blackPoint} disabled={!hasImage} />
            </div>

            <div class="control-group">
              <label for="white-point">White point: {whitePoint}</label>
              <input id="white-point" type="range" min="1" max="255" bind:value={whitePoint} disabled={!hasImage} />
            </div>

            <div class="control-group">
              <label for="hue-shift">Hue shift: {hueShift}°</label>
              <input id="hue-shift" type="range" min="-180" max="180" bind:value={hueShift} disabled={!hasImage} />
            </div>

            <div class="control-group">
              <label for="saturation">Saturation: {saturationPercent}%</label>
              <input id="saturation" type="range" min="0" max="200" bind:value={saturationPercent} disabled={!hasImage} />
            </div>

            <div class="control-group">
              <label for="blur-radius">Blur radius: {blurRadius}</label>
              <input id="blur-radius" type="range" min="0" max="8" bind:value={blurRadius} disabled={!hasImage} />
            </div>

            <button class="button" aria-label="Apply adjustments" onclick={applyAdjustments} disabled={!hasImage}>
              Apply adjustments
            </button>
          </div>
        {:else}
          <div class="inspector-content">
            <h3>Layers</h3>
            <ul class="layer-list">
              <li class="layer-item active">
                <span class="layer-dot"></span>
                <span>Base raster layer</span>
                <span class="layer-tag">Live</span>
              </li>
              {#if backgroundStatus}
                <li class="layer-item">
                  <span class="layer-dot"></span>
                  <span>AI background result</span>
                  <span class="layer-tag">Latest</span>
                </li>
              {/if}
            </ul>
            <p class="hint">
              Layer panel scaffold is ready for deeper multi-layer workflows while MVP edits a single raster surface.
            </p>
          </div>
        {/if}
      </aside>
    </section>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    font-family: 'Manrope', 'Space Grotesk', 'Avenir Next', 'Segoe UI', sans-serif;
    color: #141f36;
    background:
      radial-gradient(circle at 12% 14%, rgba(165, 214, 255, 0.65) 0, rgba(165, 214, 255, 0) 38%),
      radial-gradient(circle at 84% 8%, rgba(188, 234, 255, 0.55) 0, rgba(188, 234, 255, 0) 34%),
      linear-gradient(150deg, #edf4ff 0%, #f7fbff 52%, #ffffff 100%);
  }

  .page {
    min-height: 100vh;
    padding: 1rem;
  }

  .workspace {
    width: min(1520px, 100%);
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .command-bar {
    background: rgba(255, 255, 255, 0.9);
    border: 1px solid #cedcf4;
    border-radius: 1rem;
    padding: 0.95rem;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.95rem;
    backdrop-filter: blur(10px);
    box-shadow: 0 20px 48px -40px rgba(25, 66, 130, 0.4);
  }

  .brand {
    min-width: 240px;
  }

  .brand h1 {
    margin: 0.25rem 0 0;
    font-size: clamp(1.15rem, 2.3vw, 1.7rem);
    line-height: 1.15;
  }

  .subtitle {
    margin: 0.32rem 0 0;
    color: #4d5f7f;
    font-size: 0.84rem;
  }

  .command-groups {
    display: grid;
    gap: 0.55rem;
    justify-items: end;
  }

  .command-group {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
    gap: 0.45rem;
  }

  .status {
    margin: 0;
    font-size: 0.81rem;
    color: #0f4f7c;
    font-weight: 600;
  }

  .status.error {
    color: #9f123c;
  }

  .editor-shell {
    display: grid;
    gap: 0.9rem;
    grid-template-columns: 1fr;
    grid-template-areas:
      'tools'
      'canvas'
      'inspector';
  }

  .badge {
    display: inline-block;
    margin: 0;
    padding: 0.2rem 0.65rem;
    border-radius: 999px;
    background: #d5edff;
    color: #0a4a76;
    font-size: 0.75rem;
    font-weight: 800;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .panel {
    background: rgba(255, 255, 255, 0.94);
    border: 1px solid #d6e1f3;
    border-radius: 1rem;
    padding: 0.9rem;
    display: grid;
    gap: 0.6rem;
    box-shadow: 0 20px 55px -45px rgba(13, 63, 122, 0.35);
  }

  .tool-rail {
    grid-area: tools;
    align-content: flex-start;
  }

  .panel-title-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.75rem;
  }

  .panel-title-row h2 {
    margin: 0;
    font-size: 0.96rem;
  }

  .panel-title-row p {
    margin: 0;
    color: #667899;
    font-size: 0.73rem;
    font-weight: 700;
    letter-spacing: 0.03em;
  }

  .tool-stack {
    display: grid;
    gap: 0.42rem;
  }

  .tool-button {
    display: grid;
    grid-template-columns: 2.2rem 1fr auto;
    align-items: center;
    gap: 0.48rem;
    border: 1px solid #c9d6eb;
    border-radius: 0.78rem;
    background: #f8fbff;
    color: #1d2e4c;
    font-weight: 700;
    padding: 0.46rem 0.52rem;
    cursor: pointer;
    transition: border-color 120ms ease, transform 120ms ease, background-color 120ms ease;
  }

  .tool-button:hover {
    transform: translateY(-1px);
    border-color: #95b4de;
  }

  .tool-button.active {
    background: linear-gradient(120deg, #daf0ff 0%, #d0e6ff 100%);
    border-color: #6ea9e6;
    color: #13386d;
  }

  .tool-glyph {
    display: grid;
    place-items: center;
    border-radius: 0.56rem;
    background: #dde9fa;
    font-size: 0.71rem;
    font-weight: 800;
    min-height: 2rem;
    letter-spacing: 0.03em;
  }

  .tool-name {
    text-align: left;
    font-size: 0.83rem;
  }

  .tool-shortcut {
    color: #6b7f9f;
    font-size: 0.68rem;
    font-weight: 800;
  }

  .control-group {
    display: grid;
    gap: 0.34rem;
  }

  .control-group.compact {
    margin-top: 0.22rem;
  }

  .canvas-panel {
    grid-area: canvas;
    min-height: 520px;
    align-content: flex-start;
  }

  .canvas-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.7rem;
    flex-wrap: wrap;
  }

  .canvas-title {
    margin: 0;
    color: #1b2b4b;
    font-weight: 700;
    font-size: 0.95rem;
  }

  .canvas-meta {
    margin: 0;
    color: #617391;
    font-size: 0.78rem;
    font-weight: 600;
  }

  .canvas-stage {
    border-radius: 0.9rem;
    overflow: hidden;
  }

  .inspector {
    grid-area: inspector;
    align-content: flex-start;
  }

  .inspector-tabs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.36rem;
  }

  .tab-button {
    border: 1px solid #c9d6eb;
    border-radius: 0.65rem;
    background: #f7faff;
    color: #42577a;
    font-size: 0.76rem;
    font-weight: 700;
    padding: 0.45rem 0.5rem;
    cursor: pointer;
    transition: border-color 120ms ease, color 120ms ease, background-color 120ms ease;
  }

  .tab-button.active {
    background: #113861;
    border-color: #113861;
    color: #ffffff;
  }

  .inspector-content {
    display: grid;
    gap: 0.58rem;
  }

  .inspector-content h3 {
    margin: 0;
    font-size: 0.92rem;
  }

  .stat-grid {
    display: grid;
    gap: 0.45rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .stat-card {
    border: 1px solid #d3deef;
    border-radius: 0.72rem;
    background: #f8fbff;
    padding: 0.55rem 0.62rem;
  }

  .stat-card h3 {
    margin: 0;
    font-size: 0.71rem;
    color: #60708d;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .stat-card p {
    margin: 0.24rem 0 0;
    font-size: 0.79rem;
    color: #1a2945;
    font-weight: 700;
    line-height: 1.35;
  }

  .layer-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.36rem;
  }

  .layer-item {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 0.45rem;
    border: 1px solid #d0dcef;
    border-radius: 0.66rem;
    background: #f7faff;
    padding: 0.45rem 0.55rem;
    color: #1f2f4d;
    font-size: 0.8rem;
    font-weight: 700;
  }

  .layer-item.active {
    border-color: #75a9df;
    background: #e7f2ff;
  }

  .layer-dot {
    width: 0.53rem;
    height: 0.53rem;
    border-radius: 999px;
    background: #1f8ee0;
  }

  .layer-tag {
    color: #5a6f92;
    font-size: 0.69rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .hint {
    margin: 0;
    color: #4f5f7e;
    font-size: 0.78rem;
    line-height: 1.45;
  }

  label {
    font-size: 0.78rem;
    color: #2a3f60;
    font-weight: 700;
  }

  input {
    border: 1px solid #c8d7f0;
    border-radius: 0.62rem;
    padding: 0.53rem 0.64rem;
    font-size: 0.9rem;
    background: #f9fbff;
    color: #172744;
  }

  input:focus {
    outline: 2px solid #49cfff;
    border-color: #1f8edb;
  }

  input[type='file'] {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  input[type='range'] {
    width: 100%;
    padding: 0;
  }

  .button {
    border: 1px solid transparent;
    border-radius: 0.68rem;
    color: #ffffff;
    font-weight: 800;
    font-size: 0.82rem;
    padding: 0.54rem 0.78rem;
    cursor: pointer;
    transition: transform 120ms ease, opacity 120ms ease, border-color 120ms ease;
  }

  .button:hover {
    transform: translateY(-1px);
  }

  .button:disabled {
    opacity: 0.52;
    cursor: not-allowed;
    transform: none;
  }

  .command-primary {
    background: linear-gradient(130deg, #148fd8 0%, #0f6db6 100%);
  }

  .button-muted {
    border-color: #bfd0ea;
    background: #e9f2ff;
    color: #123e68;
  }

  .button-muted:hover {
    border-color: #8caed9;
  }

  .empty-state {
    border: 1px dashed #bfd1ee;
    border-radius: 0.9rem;
    padding: 1rem;
    background: #f7fbff;
    color: #425272;
  }

  .empty-state h2 {
    margin: 0;
    font-size: 1.03rem;
  }

  .empty-state p {
    margin: 0.35rem 0 0;
    line-height: 1.48;
  }

  .shortcut-line {
    font-size: 0.78rem;
    color: #55678a;
    font-weight: 700;
  }

  canvas {
    width: 100%;
    height: auto;
    border-radius: 0.9rem;
    background:
      linear-gradient(45deg, #e7edf8 25%, transparent 25%),
      linear-gradient(-45deg, #e7edf8 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #e7edf8 75%),
      linear-gradient(-45deg, transparent 75%, #e7edf8 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0;
    border: 1px solid #cedcf5;
    touch-action: none;
  }

  .tool-button:focus-visible,
  .tab-button:focus-visible,
  .button:focus-visible {
    outline: 2px solid #4fd6ff;
    outline-offset: 2px;
  }

  @media (max-width: 1100px) {
    .command-bar {
      flex-wrap: wrap;
    }

    .command-groups {
      width: 100%;
      justify-items: start;
    }

    .command-group {
      justify-content: flex-start;
    }
  }

  @media (min-width: 960px) {
    .editor-shell {
      grid-template-columns: 256px minmax(0, 1fr) 336px;
      grid-template-areas: 'tools canvas inspector';
      align-items: start;
    }

    .tool-rail,
    .inspector,
    .canvas-panel {
      min-height: 640px;
    }
  }

  @media (min-width: 1320px) {
    .editor-shell {
      grid-template-columns: 280px minmax(0, 1fr) 360px;
    }
  }
</style>
