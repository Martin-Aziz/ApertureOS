<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../services/api-client';

  type EditorTool =
    | 'move'
    | 'marq'
    | 'lasso'
    | 'crop'
    | 'brush'
    | 'erase'
    | 'fill'
    | 'clone'
    | 'heal'
    | 'blur'
    | 'dodge'
    | 'pen'
    | 'type'
    | 'shape'
    | 'hand'
    | 'zoom';

  type BlendMode = 'Normal' | 'Multiply' | 'Screen' | 'Overlay';
  type MenuName =
    | 'File'
    | 'Edit'
    | 'Image'
    | 'Layer'
    | 'Select'
    | 'Filter'
    | 'View'
    | 'Window'
    | 'Plugins'
    | 'Help';
  type LeftDockTab = 'navigator' | 'history' | 'presets' | 'macros';
  type RightDockTab =
    | 'color'
    | 'swatches'
    | 'gradients'
    | 'brushes'
    | 'layers'
    | 'channels'
    | 'paths'
    | 'properties'
    | 'adjustments'
    | 'history'
    | 'assets'
    | 'export'
    | 'inspector'
    | 'comments';
  type BottomTrayTab = 'timeline' | 'versions' | 'snapshots' | 'console' | 'batch-jobs' | 'plugin-logs';
  type QuickAction = 'new' | 'open' | 'save' | 'export' | 'undo' | 'redo' | 'compare' | 'share' | 'ai-assist';
  type BatchJobStatus = 'queued' | 'running' | 'done' | 'failed';

  interface Point {
    x: number;
    y: number;
  }

  interface SelectionRect {
    x: number;
    y: number;
    width: number;
    height: number;
  }

  interface Snapshot {
    id: string;
    label: string;
    createdAt: number;
    imageData: ImageData;
  }

  interface BatchJob {
    id: string;
    label: string;
    status: BatchJobStatus;
    detail: string;
    startedAt: number;
  }

  interface DocumentState {
    id: string;
    name: string;
    initialImageData: ImageData | null;
    workingImageData: ImageData | null;
    undoStack: ImageData[];
    redoStack: ImageData[];
    snapshots: Snapshot[];
    savedAt: number | null;
  }

  interface CommandItem {
    id: string;
    label: string;
    keywords: string[];
    run: () => void | Promise<void>;
  }

  const TOP_MENUS: MenuName[] = ['File', 'Edit', 'Image', 'Layer', 'Select', 'Filter', 'View', 'Window', 'Plugins', 'Help'];

  const TOOL_BUTTONS: Array<{ id: EditorTool; label: string; hotkey: string; short: string }> = [
    { id: 'move', label: 'Move', hotkey: 'V', short: 'MV' },
    { id: 'marq', label: 'Marq', hotkey: 'M', short: 'MQ' },
    { id: 'lasso', label: 'Lasso', hotkey: 'L', short: 'LS' },
    { id: 'crop', label: 'Crop', hotkey: 'C', short: 'CP' },
    { id: 'brush', label: 'Brush', hotkey: 'B', short: 'BR' },
    { id: 'erase', label: 'Erase', hotkey: 'E', short: 'ER' },
    { id: 'fill', label: 'Fill', hotkey: 'G', short: 'FL' },
    { id: 'clone', label: 'Clone', hotkey: 'J', short: 'CL' },
    { id: 'heal', label: 'Heal', hotkey: 'H', short: 'HL' },
    { id: 'blur', label: 'Blur', hotkey: 'R', short: 'BL' },
    { id: 'dodge', label: 'Dodge', hotkey: 'O', short: 'DG' },
    { id: 'pen', label: 'Pen', hotkey: 'P', short: 'PN' },
    { id: 'type', label: 'Type', hotkey: 'T', short: 'TY' },
    { id: 'shape', label: 'Shape', hotkey: 'U', short: 'SH' },
    { id: 'hand', label: 'Hand', hotkey: 'K', short: 'HD' },
    { id: 'zoom', label: 'Zoom', hotkey: 'Z', short: 'ZM' }
  ];

  const RIGHT_DOCK_TABS: Array<{ id: RightDockTab; label: string }> = [
    { id: 'color', label: 'Color' },
    { id: 'swatches', label: 'Swatches' },
    { id: 'gradients', label: 'Gradients' },
    { id: 'brushes', label: 'Brushes' },
    { id: 'layers', label: 'Layers' },
    { id: 'channels', label: 'Channels' },
    { id: 'paths', label: 'Paths' },
    { id: 'properties', label: 'Properties' },
    { id: 'adjustments', label: 'Adjustments' },
    { id: 'history', label: 'History' },
    { id: 'assets', label: 'Assets' },
    { id: 'export', label: 'Export Queue' },
    { id: 'inspector', label: 'Inspector' },
    { id: 'comments', label: 'Comments' }
  ];

  const BOTTOM_TRAY_TABS: Array<{ id: BottomTrayTab; label: string }> = [
    { id: 'timeline', label: 'Timeline' },
    { id: 'versions', label: 'Versions' },
    { id: 'snapshots', label: 'Snapshots' },
    { id: 'console', label: 'Console' },
    { id: 'batch-jobs', label: 'Batch Jobs' },
    { id: 'plugin-logs', label: 'Plugin Logs' }
  ];

  const SWATCHES = ['#f97316', '#ef4444', '#eab308', '#22c55e', '#14b8a6', '#0ea5e9', '#6366f1', '#f43f5e'];
  const GRADIENTS = ['Ocean Fade', 'Warm Print', 'Studio Gray', 'Mint Glass'];
  const DEFAULT_WIDTH = 1600;
  const DEFAULT_HEIGHT = 1000;
  const selectionStroke = '#4cc9ff';

  let idCounter = 0;

  function nextId(prefix: string): string {
    idCounter += 1;
    return `${prefix}-${Date.now()}-${idCounter}`;
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
  }

  function index(width: number, x: number, y: number, channel: number): number {
    return (y * width + x) * 4 + channel;
  }

  function readChannel(data: Uint8ClampedArray, offset: number): number {
    return data[offset] ?? 0;
  }

  function cloneImageData(imageData: ImageData): ImageData {
    return new ImageData(Uint8ClampedArray.from(imageData.data), imageData.width, imageData.height);
  }

  function cloneStack(stack: ImageData[]): ImageData[] {
    return stack.map((entry) => cloneImageData(entry));
  }

  function cloneSnapshots(items: Snapshot[]): Snapshot[] {
    return items.map((item) => ({
      ...item,
      imageData: cloneImageData(item.imageData)
    }));
  }

  function makeBlankImage(width: number, height: number): ImageData {
    const safeWidth = Math.max(1, Math.floor(width));
    const safeHeight = Math.max(1, Math.floor(height));
    const buffer = new Uint8ClampedArray(safeWidth * safeHeight * 4);

    for (let offset = 0; offset < buffer.length; offset += 4) {
      buffer[offset] = 248;
      buffer[offset + 1] = 248;
      buffer[offset + 2] = 248;
      buffer[offset + 3] = 255;
    }

    return new ImageData(buffer, safeWidth, safeHeight);
  }

  function createDocument(name: string, width = DEFAULT_WIDTH, height = DEFAULT_HEIGHT): DocumentState {
    const blank = makeBlankImage(width, height);
    return {
      id: nextId('doc'),
      name,
      initialImageData: cloneImageData(blank),
      workingImageData: cloneImageData(blank),
      undoStack: [],
      redoStack: [],
      snapshots: [],
      savedAt: null
    };
  }

  function formatClock(timestamp: number | null): string {
    if (!timestamp) {
      return 'Never';
    }

    return new Date(timestamp).toLocaleTimeString([], {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
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
    const safeHex = normalized.length === 6 ? normalized : '21a6ff';
    return [
      Number.parseInt(safeHex.slice(0, 2), 16),
      Number.parseInt(safeHex.slice(2, 4), 16),
      Number.parseInt(safeHex.slice(4, 6), 16)
    ];
  }

  let canvasEl: HTMLCanvasElement;
  let openInputEl: HTMLInputElement;
  let hydrated = false;

  let documents: DocumentState[] = [createDocument('poster.psd', 1600, 1000), createDocument('hero-banner.png', 1920, 1080)];
  let activeDocumentId = documents[0]?.id ?? '';
  let activeFileName = documents[0]?.name ?? 'Untitled image';

  let initialImageData: ImageData | null = documents[0]?.initialImageData
    ? cloneImageData(documents[0].initialImageData)
    : null;
  let workingImageData: ImageData | null = documents[0]?.workingImageData
    ? cloneImageData(documents[0].workingImageData)
    : null;

  let undoStack: ImageData[] = cloneStack(documents[0]?.undoStack ?? []);
  let redoStack: ImageData[] = cloneStack(documents[0]?.redoStack ?? []);
  let snapshots: Snapshot[] = cloneSnapshots(documents[0]?.snapshots ?? []);
  let lastSavedAt: number | null = documents[0]?.savedAt ?? null;

  let activeTool: EditorTool = 'brush';
  let brushSize = 24;
  let brushColor = '#21a6ff';
  let brushHardness = 72;
  let brushOpacity = 100;
  let brushFlow = 88;
  let blendMode: BlendMode = 'Normal';

  let blackPoint = 0;
  let whitePoint = 255;
  let hueShift = 0;
  let saturationPercent = 100;
  let blurRadius = 0;

  let zoomPercent = 125;
  let canvasPanX = 0;
  let canvasPanY = 0;

  let cursorX = 0;
  let cursorY = 0;
  let selectionRect: SelectionRect | null = null;
  let selectionStart: Point | null = null;

  let isDrawing = false;
  let isPanning = false;
  let panOrigin: Point | null = null;
  let isMovingSelection = false;
  let moveSelectionOrigin: Point | null = null;
  let moveSelectionStartRect: SelectionRect | null = null;
  let penAnchor: Point | null = null;

  let compareEnabled = false;
  let leftDockVisible = true;
  let rightDockVisible = true;
  let leftDockTab: LeftDockTab = 'navigator';
  let rightDockTab: RightDockTab = 'layers';
  let bottomTrayTab: BottomTrayTab = 'console';

  let showRulers = true;
  let showGuides = true;
  let showBleed = false;
  let showSafeArea = true;

  let activeMenu: MenuName | null = null;
  let commandSearch = '';
  let commandCatalog: CommandItem[] = [];
  let activeMenuCommands: CommandItem[] = [];
  let filteredCommands: CommandItem[] = [];

  let isRemovingBackground = false;
  let backgroundError: string | null = null;
  let backgroundStatus: string | null = null;

  let historyLog: string[] = ['Editor started.'];
  let pluginLogs: string[] = ['Plugin host online.', 'OpenCanvas bridge active.'];
  let exportQueue: string[] = [];
  let batchJobs: BatchJob[] = [];
  let comments: string[] = ['Kickoff note: verify hero contrast before final export.'];
  let commentDraft = '';

  const menuCommandIds: Record<MenuName, string[]> = {
    File: ['new-doc', 'open-file', 'save-doc', 'export-doc', 'share-doc'],
    Edit: ['undo-step', 'redo-step', 'crop-selection', 'reset-image'],
    Image: ['ai-assist', 'apply-adjustments', 'make-snapshot'],
    Layer: ['open-layers', 'open-assets', 'open-export-queue'],
    Select: ['tool-marq', 'tool-lasso', 'clear-selection'],
    Filter: ['apply-adjustments', 'macro-soft-focus', 'macro-auto-tone'],
    View: ['toggle-rulers', 'toggle-guides', 'toggle-left-dock', 'toggle-right-dock'],
    Window: ['open-properties', 'open-history', 'open-comments'],
    Plugins: ['open-plugin-logs', 'ai-assist'],
    Help: ['show-shortcuts']
  };

  $: hasImage = Boolean(workingImageData);
  $: zoomScale = zoomPercent / 100;
  $: canvasDimensions = hasImage ? `${workingImageData?.width ?? 0}x${workingImageData?.height ?? 0}` : '--';
  $: memoryLabel = hasImage ? `${((workingImageData?.data.length ?? 0) / (1024 * 1024)).toFixed(1)} MB` : '--';
  $: selectionLabel = selectionRect
    ? `${Math.max(1, Math.round(selectionRect.width))}x${Math.max(1, Math.round(selectionRect.height))}`
    : '--';
  $: activeToolLabel = TOOL_BUTTONS.find((entry) => entry.id === activeTool)?.label ?? activeTool;

  $: {
    const needle = commandSearch.trim().toLowerCase();
    filteredCommands =
      needle.length === 0
        ? []
        : commandCatalog
            .filter((command) => `${command.label} ${command.keywords.join(' ')}`.toLowerCase().includes(needle))
            .slice(0, 8);
  }

  $: {
    activeMenuCommands =
      activeMenu === null
        ? []
        : menuCommandIds[activeMenu]
            .map((id) => commandCatalog.find((command) => command.id === id))
            .filter((command): command is CommandItem => Boolean(command));
  }

  commandCatalog = [
    {
      id: 'new-doc',
      label: 'New Document',
      keywords: ['new', 'file', 'canvas'],
      run: () => runQuickAction('new')
    },
    {
      id: 'open-file',
      label: 'Open Image',
      keywords: ['open', 'import', 'file'],
      run: () => runQuickAction('open')
    },
    {
      id: 'save-doc',
      label: 'Save Snapshot',
      keywords: ['save', 'disk', 'snapshot'],
      run: () => runQuickAction('save')
    },
    {
      id: 'export-doc',
      label: 'Export PNG',
      keywords: ['export', 'png', 'download'],
      run: () => runQuickAction('export')
    },
    {
      id: 'undo-step',
      label: 'Undo',
      keywords: ['undo', 'history'],
      run: () => runQuickAction('undo')
    },
    {
      id: 'redo-step',
      label: 'Redo',
      keywords: ['redo', 'history'],
      run: () => runQuickAction('redo')
    },
    {
      id: 'share-doc',
      label: 'Share Token',
      keywords: ['share', 'clipboard'],
      run: () => runQuickAction('share')
    },
    {
      id: 'ai-assist',
      label: 'AI Assist: Remove Background',
      keywords: ['ai', 'assist', 'remove', 'background'],
      run: () => runQuickAction('ai-assist')
    },
    {
      id: 'make-snapshot',
      label: 'Create Snapshot',
      keywords: ['snapshot', 'version'],
      run: () => createSnapshot('Manual snapshot')
    },
    {
      id: 'toggle-rulers',
      label: 'Toggle Rulers',
      keywords: ['view', 'rulers'],
      run: () => {
        showRulers = !showRulers;
        pushHistory(`Rulers ${showRulers ? 'enabled' : 'disabled'}.`);
      }
    },
    {
      id: 'toggle-guides',
      label: 'Toggle Guides',
      keywords: ['view', 'guides'],
      run: () => {
        showGuides = !showGuides;
        pushHistory(`Guides ${showGuides ? 'enabled' : 'disabled'}.`);
      }
    },
    {
      id: 'toggle-left-dock',
      label: 'Toggle Left Dock',
      keywords: ['window', 'left', 'dock'],
      run: () => {
        leftDockVisible = !leftDockVisible;
      }
    },
    {
      id: 'toggle-right-dock',
      label: 'Toggle Right Dock',
      keywords: ['window', 'right', 'dock'],
      run: () => {
        rightDockVisible = !rightDockVisible;
      }
    },
    {
      id: 'open-layers',
      label: 'Open Layers Dock',
      keywords: ['layers', 'dock'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'layers';
      }
    },
    {
      id: 'open-assets',
      label: 'Open Assets Dock',
      keywords: ['assets', 'dock'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'assets';
      }
    },
    {
      id: 'open-export-queue',
      label: 'Open Export Queue',
      keywords: ['export', 'queue'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'export';
      }
    },
    {
      id: 'open-properties',
      label: 'Open Properties Dock',
      keywords: ['properties', 'inspector'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'properties';
      }
    },
    {
      id: 'open-history',
      label: 'Open History Dock',
      keywords: ['history', 'dock'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'history';
      }
    },
    {
      id: 'open-comments',
      label: 'Open Comments Dock',
      keywords: ['comments', 'dock'],
      run: () => {
        rightDockVisible = true;
        rightDockTab = 'comments';
      }
    },
    {
      id: 'clear-selection',
      label: 'Clear Selection',
      keywords: ['select', 'clear'],
      run: () => {
        selectionRect = null;
        drawEditorSurface();
      }
    },
    {
      id: 'apply-adjustments',
      label: 'Apply Adjustments',
      keywords: ['adjustments', 'levels', 'hue', 'blur'],
      run: () => applyAdjustments()
    },
    {
      id: 'crop-selection',
      label: 'Crop To Selection',
      keywords: ['crop', 'selection'],
      run: () => cropSelection()
    },
    {
      id: 'reset-image',
      label: 'Reset Image',
      keywords: ['reset', 'image'],
      run: () => resetImage()
    },
    {
      id: 'open-plugin-logs',
      label: 'Open Plugin Logs',
      keywords: ['plugins', 'logs'],
      run: () => {
        bottomTrayTab = 'plugin-logs';
      }
    },
    {
      id: 'tool-marq',
      label: 'Switch To Marquee Tool',
      keywords: ['tool', 'marq'],
      run: () => selectTool('marq')
    },
    {
      id: 'tool-lasso',
      label: 'Switch To Lasso Tool',
      keywords: ['tool', 'lasso'],
      run: () => selectTool('lasso')
    },
    {
      id: 'macro-soft-focus',
      label: 'Run Macro: Soft Focus',
      keywords: ['macro', 'soft', 'focus'],
      run: () => runMacro('soft-focus')
    },
    {
      id: 'macro-auto-tone',
      label: 'Run Macro: Auto Tone',
      keywords: ['macro', 'auto', 'tone'],
      run: () => runMacro('auto-tone')
    },
    {
      id: 'show-shortcuts',
      label: 'Show Shortcuts',
      keywords: ['help', 'shortcuts'],
      run: () => showShortcutHelp()
    }
  ];

  onMount(() => {
    hydrated = true;
    drawEditorSurface();

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

      if (usesModifier && key === 's') {
        event.preventDefault();
        saveDocument();
        return;
      }

      if (usesModifier && key === 'o') {
        event.preventDefault();
        openFilePicker();
        return;
      }

      if (usesModifier && key === 'n') {
        event.preventDefault();
        createNewDocument();
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

      if (key === '[') {
        brushSize = clamp(brushSize - 2, 1, 200);
        return;
      }

      if (key === ']') {
        brushSize = clamp(brushSize + 2, 1, 200);
        return;
      }

      if (key === '+' || key === '=') {
        zoomPercent = clamp(zoomPercent + 25, 25, 400);
        return;
      }

      if (key === '-') {
        zoomPercent = clamp(zoomPercent - 25, 25, 400);
        return;
      }

      if (key === '0') {
        zoomPercent = 100;
        return;
      }

      const mapped = TOOL_BUTTONS.find((entry) => entry.hotkey.toLowerCase() === key);
      if (mapped) {
        selectTool(mapped.id);
      }
    };

    const onGlobalPointerDown = (event: PointerEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest('.menu-slot') && !target?.closest('.search-cluster')) {
        activeMenu = null;
      }
    };

    const onBeforeUnload = () => {
      persistActiveDocumentState();
    };

    window.addEventListener('keydown', onShortcut);
    window.addEventListener('pointerdown', onGlobalPointerDown);
    window.addEventListener('beforeunload', onBeforeUnload);

    return () => {
      window.removeEventListener('keydown', onShortcut);
      window.removeEventListener('pointerdown', onGlobalPointerDown);
      window.removeEventListener('beforeunload', onBeforeUnload);
    };
  });

  function pushHistory(message: string): void {
    historyLog = [`${formatClock(Date.now())} ${message}`, ...historyLog].slice(0, 120);
  }

  function pushPluginLog(message: string): void {
    pluginLogs = [`${formatClock(Date.now())} ${message}`, ...pluginLogs].slice(0, 120);
  }

  function startBatchJob(label: string, detail = 'Processing'): string {
    const id = nextId('job');
    const job: BatchJob = {
      id,
      label,
      status: 'running',
      detail,
      startedAt: Date.now()
    };

    batchJobs = [job, ...batchJobs].slice(0, 30);
    return id;
  }

  function completeBatchJob(id: string, status: BatchJobStatus, detail: string): void {
    batchJobs = batchJobs.map((job) => {
      if (job.id !== id) {
        return job;
      }

      return {
        ...job,
        status,
        detail
      };
    });
  }

  function setInfo(message: string): void {
    backgroundError = null;
    backgroundStatus = message;
    pushHistory(message);
  }

  function setError(message: string): void {
    backgroundError = message;
    backgroundStatus = null;
    pushHistory(`Error: ${message}`);
  }

  function persistActiveDocumentState(): void {
    documents = documents.map((document) => {
      if (document.id !== activeDocumentId) {
        return document;
      }

      return {
        ...document,
        name: activeFileName,
        initialImageData: initialImageData ? cloneImageData(initialImageData) : null,
        workingImageData: workingImageData ? cloneImageData(workingImageData) : null,
        undoStack: cloneStack(undoStack),
        redoStack: cloneStack(redoStack),
        snapshots: cloneSnapshots(snapshots),
        savedAt: lastSavedAt
      };
    });
  }

  function loadDocumentState(document: DocumentState): void {
    activeFileName = document.name;
    initialImageData = document.initialImageData ? cloneImageData(document.initialImageData) : null;
    workingImageData = document.workingImageData ? cloneImageData(document.workingImageData) : null;
    undoStack = cloneStack(document.undoStack);
    redoStack = cloneStack(document.redoStack);
    snapshots = cloneSnapshots(document.snapshots);
    lastSavedAt = document.savedAt;
    compareEnabled = false;
    selectionRect = null;
    selectionStart = null;
    drawEditorSurface();
  }

  function setActiveDocument(documentId: string): void {
    if (documentId === activeDocumentId) {
      return;
    }

    persistActiveDocumentState();
    const nextDocument = documents.find((entry) => entry.id === documentId);
    if (!nextDocument) {
      return;
    }

    activeDocumentId = documentId;
    loadDocumentState(nextDocument);
    setInfo(`Switched to ${nextDocument.name}.`);
  }

  function createNewDocument(): void {
    persistActiveDocumentState();

    const nextNumber = documents.length + 1;
    const name = `untitled-${nextNumber}.png`;
    const newDocument = createDocument(name);

    documents = [...documents, newDocument];
    activeDocumentId = newDocument.id;
    loadDocumentState(newDocument);
    setInfo(`Created ${name}.`);
  }

  function renameActiveDocument(name: string): void {
    activeFileName = name;
    documents = documents.map((document) => {
      if (document.id !== activeDocumentId) {
        return document;
      }

      return {
        ...document,
        name
      };
    });
  }

  function drawEditorSurface(): void {
    if (!canvasEl) {
      return;
    }

    const context = canvasEl.getContext('2d');
    if (!context) {
      return;
    }

    const displayImage = compareEnabled && initialImageData ? initialImageData : workingImageData;
    if (!displayImage) {
      context.clearRect(0, 0, canvasEl.width, canvasEl.height);
      return;
    }

    if (canvasEl.width !== displayImage.width || canvasEl.height !== displayImage.height) {
      canvasEl.width = displayImage.width;
      canvasEl.height = displayImage.height;
    }

    context.putImageData(displayImage, 0, 0);

    if (selectionRect && !compareEnabled) {
      context.save();
      context.strokeStyle = selectionStroke;
      context.lineWidth = Math.max(1, Math.round(canvasEl.width / 300));
      context.setLineDash([8, 5]);
      context.strokeRect(selectionRect.x + 0.5, selectionRect.y + 0.5, selectionRect.width, selectionRect.height);
      context.restore();
    }
  }

  function pushUndoSnapshot(): void {
    if (!workingImageData) {
      return;
    }

    undoStack = [...undoStack, cloneImageData(workingImageData)].slice(-80);
    redoStack = [];
  }

  function applyNewWorkingImage(nextImage: ImageData, resetBaseline = false): void {
    workingImageData = cloneImageData(nextImage);

    if (resetBaseline || !initialImageData) {
      initialImageData = cloneImageData(nextImage);
    }

    drawEditorSurface();
  }

  function undo(): void {
    if (!workingImageData || undoStack.length === 0) {
      return;
    }

    const previous = undoStack[undoStack.length - 1];
    if (!previous) {
      return;
    }

    undoStack = undoStack.slice(0, -1);
    redoStack = [...redoStack, cloneImageData(workingImageData)].slice(-80);

    workingImageData = cloneImageData(previous);
    selectionRect = null;
    drawEditorSurface();
  }

  function redo(): void {
    if (!workingImageData || redoStack.length === 0) {
      return;
    }

    const next = redoStack[redoStack.length - 1];
    if (!next) {
      return;
    }

    redoStack = redoStack.slice(0, -1);
    undoStack = [...undoStack, cloneImageData(workingImageData)].slice(-80);

    workingImageData = cloneImageData(next);
    selectionRect = null;
    drawEditorSurface();
  }

  function toCanvasPoint(event: PointerEvent): Point {
    const bounds = canvasEl.getBoundingClientRect();
    const scaleX = canvasEl.width / Math.max(1, bounds.width);
    const scaleY = canvasEl.height / Math.max(1, bounds.height);

    const x = clamp(Math.floor((event.clientX - bounds.left) * scaleX), 0, Math.max(0, canvasEl.width - 1));
    const y = clamp(Math.floor((event.clientY - bounds.top) * scaleY), 0, Math.max(0, canvasEl.height - 1));
    return { x, y };
  }

  function normalizedSelection(start: Point, end: Point): SelectionRect | null {
    const x = Math.min(start.x, end.x);
    const y = Math.min(start.y, end.y);
    const width = Math.abs(start.x - end.x);
    const height = Math.abs(start.y - end.y);

    if (width < 2 || height < 2) {
      return null;
    }

    return { x, y, width, height };
  }

  function pointInRect(point: Point, rect: SelectionRect): boolean {
    return (
      point.x >= rect.x &&
      point.x <= rect.x + rect.width &&
      point.y >= rect.y &&
      point.y <= rect.y + rect.height
    );
  }

  function isSelectionTool(tool: EditorTool): boolean {
    return tool === 'marq' || tool === 'lasso' || tool === 'crop';
  }

  function isPaintTool(tool: EditorTool): boolean {
    return (
      tool === 'brush' ||
      tool === 'erase' ||
      tool === 'clone' ||
      tool === 'heal' ||
      tool === 'blur' ||
      tool === 'dodge'
    );
  }

  function blendByMode(base: number, paint: number, mode: BlendMode): number {
    const b = base / 255;
    const p = paint / 255;

    if (mode === 'Multiply') {
      return Math.round(clamp(b * p, 0, 1) * 255);
    }

    if (mode === 'Screen') {
      return Math.round(clamp(1 - (1 - b) * (1 - p), 0, 1) * 255);
    }

    if (mode === 'Overlay') {
      const next = b < 0.5 ? 2 * b * p : 1 - 2 * (1 - b) * (1 - p);
      return Math.round(clamp(next, 0, 1) * 255);
    }

    return paint;
  }

  function brushStrength(distance: number, radius: number): number {
    if (distance > radius) {
      return 0;
    }

    const hardEdge = radius * (brushHardness / 100);
    if (distance <= hardEdge) {
      return 1;
    }

    const falloff = Math.max(1, radius - hardEdge);
    return clamp(1 - (distance - hardEdge) / falloff, 0, 1);
  }

  function paintAtPointWithTool(point: Point, tool: EditorTool, size: number): void {
    if (!workingImageData) {
      return;
    }

    const data = workingImageData.data;
    const source = Uint8ClampedArray.from(data);
    const width = workingImageData.width;
    const height = workingImageData.height;
    const [paintRed, paintGreen, paintBlue] = hexToRgb(brushColor);
    const radius = Math.max(1, Math.floor(size / 2));

    for (let y = point.y - radius; y <= point.y + radius; y += 1) {
      for (let x = point.x - radius; x <= point.x + radius; x += 1) {
        if (x < 0 || y < 0 || x >= width || y >= height) {
          continue;
        }

        const distance = Math.hypot(x - point.x, y - point.y);
        if (distance > radius) {
          continue;
        }

        const localStrength =
          brushStrength(distance, radius) * clamp((brushOpacity / 100) * (brushFlow / 100), 0, 1);
        if (localStrength <= 0) {
          continue;
        }

        const offset = index(width, x, y, 0);

        if (tool === 'erase') {
          data[offset + 3] = Math.round(readChannel(source, offset + 3) * (1 - localStrength));
          continue;
        }

        if (tool === 'dodge') {
          data[offset] = clamp(Math.round(readChannel(source, offset) + localStrength * 55), 0, 255);
          data[offset + 1] = clamp(Math.round(readChannel(source, offset + 1) + localStrength * 55), 0, 255);
          data[offset + 2] = clamp(Math.round(readChannel(source, offset + 2) + localStrength * 55), 0, 255);
          data[offset + 3] = 255;
          continue;
        }

        if (tool === 'blur') {
          let sampleRed = 0;
          let sampleGreen = 0;
          let sampleBlue = 0;
          let sampleAlpha = 0;
          let samples = 0;

          for (let sampleY = y - 1; sampleY <= y + 1; sampleY += 1) {
            for (let sampleX = x - 1; sampleX <= x + 1; sampleX += 1) {
              if (sampleX < 0 || sampleY < 0 || sampleX >= width || sampleY >= height) {
                continue;
              }

              const sampleOffset = index(width, sampleX, sampleY, 0);
              sampleRed += readChannel(source, sampleOffset);
              sampleGreen += readChannel(source, sampleOffset + 1);
              sampleBlue += readChannel(source, sampleOffset + 2);
              sampleAlpha += readChannel(source, sampleOffset + 3);
              samples += 1;
            }
          }

          if (samples > 0) {
            const avgRed = Math.round(sampleRed / samples);
            const avgGreen = Math.round(sampleGreen / samples);
            const avgBlue = Math.round(sampleBlue / samples);
            const avgAlpha = Math.round(sampleAlpha / samples);

            data[offset] = Math.round(readChannel(source, offset) + (avgRed - readChannel(source, offset)) * localStrength);
            data[offset + 1] = Math.round(
              readChannel(source, offset + 1) + (avgGreen - readChannel(source, offset + 1)) * localStrength
            );
            data[offset + 2] = Math.round(
              readChannel(source, offset + 2) + (avgBlue - readChannel(source, offset + 2)) * localStrength
            );
            data[offset + 3] = Math.round(
              readChannel(source, offset + 3) + (avgAlpha - readChannel(source, offset + 3)) * localStrength
            );
          }

          continue;
        }

        if (tool === 'heal' || tool === 'clone') {
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
              sampleRed += readChannel(source, ringOffset);
              sampleGreen += readChannel(source, ringOffset + 1);
              sampleBlue += readChannel(source, ringOffset + 2);
              sampleAlpha += readChannel(source, ringOffset + 3);
              samples += 1;
            }
          }

          if (samples > 0) {
            const averageRed = Math.round(sampleRed / samples);
            const averageGreen = Math.round(sampleGreen / samples);
            const averageBlue = Math.round(sampleBlue / samples);
            const averageAlpha = Math.round(sampleAlpha / samples);

            data[offset] = Math.round(readChannel(source, offset) + (averageRed - readChannel(source, offset)) * localStrength);
            data[offset + 1] = Math.round(
              readChannel(source, offset + 1) + (averageGreen - readChannel(source, offset + 1)) * localStrength
            );
            data[offset + 2] = Math.round(
              readChannel(source, offset + 2) + (averageBlue - readChannel(source, offset + 2)) * localStrength
            );
            data[offset + 3] = Math.round(
              readChannel(source, offset + 3) + (averageAlpha - readChannel(source, offset + 3)) * localStrength
            );
          }

          continue;
        }

        const targetRed = blendByMode(readChannel(source, offset), paintRed, blendMode);
        const targetGreen = blendByMode(readChannel(source, offset + 1), paintGreen, blendMode);
        const targetBlue = blendByMode(readChannel(source, offset + 2), paintBlue, blendMode);

        data[offset] = Math.round(readChannel(source, offset) + (targetRed - readChannel(source, offset)) * localStrength);
        data[offset + 1] = Math.round(
          readChannel(source, offset + 1) + (targetGreen - readChannel(source, offset + 1)) * localStrength
        );
        data[offset + 2] = Math.round(
          readChannel(source, offset + 2) + (targetBlue - readChannel(source, offset + 2)) * localStrength
        );
        data[offset + 3] = 255;
      }
    }

    drawEditorSurface();
  }

  function paintAtPoint(point: Point): void {
    paintAtPointWithTool(point, activeTool, brushSize);
  }

  function paintLine(start: Point, end: Point): void {
    if (!workingImageData) {
      return;
    }

    const steps = Math.max(Math.abs(end.x - start.x), Math.abs(end.y - start.y));
    if (steps === 0) {
      paintAtPointWithTool(start, 'brush', Math.max(1, Math.floor(brushSize / 2)));
      return;
    }

    for (let step = 0; step <= steps; step += 1) {
      const t = step / steps;
      const point = {
        x: Math.round(start.x + (end.x - start.x) * t),
        y: Math.round(start.y + (end.y - start.y) * t)
      };
      paintAtPointWithTool(point, 'brush', Math.max(1, Math.floor(brushSize / 2)));
    }
  }

  function fillSelectionOrCanvas(): void {
    if (!workingImageData) {
      return;
    }

    const [red, green, blue] = hexToRgb(brushColor);
    const data = workingImageData.data;
    const width = workingImageData.width;
    const height = workingImageData.height;

    const bounds = selectionRect
      ? {
          xStart: clamp(Math.floor(selectionRect.x), 0, width - 1),
          yStart: clamp(Math.floor(selectionRect.y), 0, height - 1),
          xEnd: clamp(Math.ceil(selectionRect.x + selectionRect.width), 0, width),
          yEnd: clamp(Math.ceil(selectionRect.y + selectionRect.height), 0, height)
        }
      : {
          xStart: 0,
          yStart: 0,
          xEnd: width,
          yEnd: height
        };

    for (let y = bounds.yStart; y < bounds.yEnd; y += 1) {
      for (let x = bounds.xStart; x < bounds.xEnd; x += 1) {
        const offset = index(width, x, y, 0);
        data[offset] = red;
        data[offset + 1] = green;
        data[offset + 2] = blue;
        data[offset + 3] = 255;
      }
    }

    drawEditorSurface();
    setInfo(selectionRect ? 'Filled selected area.' : 'Filled full canvas.');
  }

  function addTextAtPoint(point: Point): void {
    if (!canvasEl || !workingImageData) {
      return;
    }

    const text = window.prompt('Type text to place on canvas:', 'OpenCanvas');
    if (!text) {
      return;
    }

    const context = canvasEl.getContext('2d');
    if (!context) {
      return;
    }

    context.putImageData(workingImageData, 0, 0);
    context.save();
    context.font = `${Math.max(16, Math.round(brushSize * 1.3))}px "IBM Plex Sans", "Avenir Next", sans-serif`;
    context.fillStyle = brushColor;
    context.globalAlpha = brushOpacity / 100;
    context.fillText(text, point.x, point.y);
    context.restore();

    workingImageData = context.getImageData(0, 0, canvasEl.width, canvasEl.height);
    drawEditorSurface();
    setInfo('Placed text layer preview on the active raster.');
  }

  function drawShapeAtPoint(point: Point): void {
    if (!canvasEl || !workingImageData) {
      return;
    }

    const context = canvasEl.getContext('2d');
    if (!context) {
      return;
    }

    const width = Math.max(18, Math.round(brushSize * 2.8));
    const height = Math.max(18, Math.round(brushSize * 1.8));

    context.putImageData(workingImageData, 0, 0);
    context.save();
    context.strokeStyle = brushColor;
    context.fillStyle = `${brushColor}22`;
    context.lineWidth = Math.max(1, Math.round(brushSize / 9));
    context.strokeRect(point.x - width / 2, point.y - height / 2, width, height);
    context.fillRect(point.x - width / 2, point.y - height / 2, width, height);
    context.restore();

    workingImageData = context.getImageData(0, 0, canvasEl.width, canvasEl.height);
    drawEditorSurface();
    setInfo('Inserted shape preview on canvas.');
  }

  function handlePenPoint(point: Point): void {
    if (!workingImageData) {
      return;
    }

    if (!penAnchor) {
      penAnchor = point;
      setInfo('Pen anchor placed. Click another point to draw.');
      return;
    }

    pushUndoSnapshot();
    paintLine(penAnchor, point);
    penAnchor = point;
    setInfo('Pen segment drawn.');
  }

  function handlePointerDown(event: PointerEvent): void {
    if (!canvasEl || !workingImageData) {
      return;
    }

    canvasEl.setPointerCapture(event.pointerId);
    const point = toCanvasPoint(event);
    cursorX = point.x;
    cursorY = point.y;

    if (compareEnabled && activeTool !== 'hand' && activeTool !== 'zoom') {
      setInfo('Compare mode is on. Toggle Compare to edit.');
      return;
    }

    if (activeTool === 'hand') {
      isPanning = true;
      panOrigin = {
        x: event.clientX - canvasPanX,
        y: event.clientY - canvasPanY
      };
      return;
    }

    if (activeTool === 'zoom') {
      zoomPercent = clamp(zoomPercent + (event.altKey ? -25 : 25), 25, 400);
      return;
    }

    if (activeTool === 'fill') {
      pushUndoSnapshot();
      fillSelectionOrCanvas();
      return;
    }

    if (activeTool === 'type') {
      pushUndoSnapshot();
      addTextAtPoint(point);
      return;
    }

    if (activeTool === 'shape') {
      pushUndoSnapshot();
      drawShapeAtPoint(point);
      return;
    }

    if (activeTool === 'pen') {
      handlePenPoint(point);
      return;
    }

    if (activeTool === 'move' && selectionRect && pointInRect(point, selectionRect)) {
      isMovingSelection = true;
      moveSelectionOrigin = point;
      moveSelectionStartRect = { ...selectionRect };
      return;
    }

    if (isSelectionTool(activeTool)) {
      selectionStart = point;
      selectionRect = null;
      drawEditorSurface();
      return;
    }

    if (isPaintTool(activeTool)) {
      pushUndoSnapshot();
      isDrawing = true;
      paintAtPoint(point);
    }
  }

  function handlePointerMove(event: PointerEvent): void {
    if (!workingImageData || !canvasEl) {
      return;
    }

    const point = toCanvasPoint(event);
    cursorX = point.x;
    cursorY = point.y;

    if (isPanning && panOrigin) {
      canvasPanX = event.clientX - panOrigin.x;
      canvasPanY = event.clientY - panOrigin.y;
      return;
    }

    if (isMovingSelection && selectionRect && moveSelectionOrigin && moveSelectionStartRect) {
      const deltaX = point.x - moveSelectionOrigin.x;
      const deltaY = point.y - moveSelectionOrigin.y;
      const maxX = Math.max(0, workingImageData.width - moveSelectionStartRect.width);
      const maxY = Math.max(0, workingImageData.height - moveSelectionStartRect.height);

      selectionRect = {
        ...selectionRect,
        x: clamp(moveSelectionStartRect.x + deltaX, 0, maxX),
        y: clamp(moveSelectionStartRect.y + deltaY, 0, maxY)
      };
      drawEditorSurface();
      return;
    }

    if (selectionStart && isSelectionTool(activeTool)) {
      selectionRect = normalizedSelection(selectionStart, point);
      drawEditorSurface();
      return;
    }

    if (isDrawing && isPaintTool(activeTool)) {
      paintAtPoint(point);
    }
  }

  function handlePointerUp(event: PointerEvent): void {
    if (canvasEl && canvasEl.hasPointerCapture(event.pointerId)) {
      canvasEl.releasePointerCapture(event.pointerId);
    }

    if (isMovingSelection) {
      setInfo('Selection frame moved.');
    }

    isDrawing = false;
    isPanning = false;
    isMovingSelection = false;
    panOrigin = null;
    moveSelectionOrigin = null;
    moveSelectionStartRect = null;
    selectionStart = null;
  }

  function selectTool(tool: EditorTool): void {
    activeTool = tool;
    penAnchor = null;

    if (tool !== 'move' && !isSelectionTool(tool)) {
      selectionStart = null;
    }

    pushHistory(`Tool set to ${TOOL_BUTTONS.find((entry) => entry.id === tool)?.label ?? tool}.`);
  }

  function resetControls(): void {
    brushSize = 24;
    brushHardness = 72;
    brushOpacity = 100;
    brushFlow = 88;
    blendMode = 'Normal';

    blackPoint = 0;
    whitePoint = 255;
    hueShift = 0;
    saturationPercent = 100;
    blurRadius = 0;
  }

  function resetImage(): void {
    if (!initialImageData) {
      return;
    }

    workingImageData = cloneImageData(initialImageData);
    undoStack = [];
    redoStack = [];
    selectionRect = null;
    compareEnabled = false;
    resetControls();
    drawEditorSurface();
    setInfo('Image reset to baseline state.');
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
    compareEnabled = false;
    drawEditorSurface();
    setInfo(`Cropped to ${width}x${height}.`);
  }

  function applyAdjustments(): void {
    if (!workingImageData) {
      return;
    }

    const jobId = startBatchJob('Apply adjustments');
    pushUndoSnapshot();

    const leveled = applyLevels(workingImageData.data, blackPoint, whitePoint);
    const adjusted = applyHueSaturation(leveled, hueShift, saturationPercent / 100);
    const blurred = applyBoxBlur(adjusted, workingImageData.width, workingImageData.height, blurRadius);

    workingImageData = new ImageData(Uint8ClampedArray.from(blurred), workingImageData.width, workingImageData.height);
    selectionRect = null;
    drawEditorSurface();
    completeBatchJob(jobId, 'done', 'Adjustments complete');
    setInfo('Adjustments applied to active layer.');
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

  async function loadFileToCanvas(file: File): Promise<void> {
    if (!canvasEl) {
      return;
    }

    const jobId = startBatchJob(`Open ${file.name}`);
    let source: CanvasImageSource;
    let width = 0;
    let height = 0;

    try {
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
        throw new Error('Canvas context unavailable');
      }

      context.drawImage(source, 0, 0, width, height);
      const loaded = context.getImageData(0, 0, width, height);

      initialImageData = cloneImageData(loaded);
      workingImageData = cloneImageData(loaded);
      undoStack = [];
      redoStack = [];
      snapshots = [];
      lastSavedAt = null;
      selectionRect = null;
      compareEnabled = false;
      resetControls();
      renameActiveDocument(file.name);
      drawEditorSurface();

      completeBatchJob(jobId, 'done', `${file.name} loaded`);
      setInfo(`Opened ${file.name}.`);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to open image.';
      completeBatchJob(jobId, 'failed', message);
      setError(message);
    }
  }

  async function importImage(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) {
      return;
    }

    await loadFileToCanvas(file);
    input.value = '';
  }

  function openFilePicker(): void {
    openInputEl?.click();
  }

  function buildWorkingDataUrl(): string | null {
    if (!workingImageData) {
      return null;
    }

    const exportCanvas = document.createElement('canvas');
    exportCanvas.width = workingImageData.width;
    exportCanvas.height = workingImageData.height;
    const context = exportCanvas.getContext('2d');
    if (!context) {
      return null;
    }

    context.putImageData(workingImageData, 0, 0);
    return exportCanvas.toDataURL('image/png');
  }

  function saveDocument(): void {
    if (!workingImageData) {
      return;
    }

    const dataUrl = buildWorkingDataUrl();
    if (!dataUrl) {
      setError('Failed to serialize image for save.');
      return;
    }

    try {
      const payload = {
        name: activeFileName,
        savedAt: Date.now(),
        width: workingImageData.width,
        height: workingImageData.height,
        image: dataUrl
      };

      localStorage.setItem(`opencanvas-document-${activeDocumentId}`, JSON.stringify(payload));
      lastSavedAt = payload.savedAt;
      createSnapshot(`Save ${snapshots.length + 1}`);
      setInfo(`Saved ${activeFileName} to local workspace storage.`);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to save document locally.';
      setError(message);
    }
  }

  function createSnapshot(label: string): void {
    if (!workingImageData) {
      return;
    }

    const entry: Snapshot = {
      id: nextId('snapshot'),
      label,
      createdAt: Date.now(),
      imageData: cloneImageData(workingImageData)
    };

    snapshots = [entry, ...snapshots].slice(0, 24);
    persistActiveDocumentState();
  }

  function restoreSnapshot(snapshotId: string): void {
    const snapshot = snapshots.find((entry) => entry.id === snapshotId);
    if (!snapshot) {
      return;
    }

    pushUndoSnapshot();
    workingImageData = cloneImageData(snapshot.imageData);
    selectionRect = null;
    compareEnabled = false;
    drawEditorSurface();
    setInfo(`Restored snapshot: ${snapshot.label}.`);
  }

  function exportImage(): void {
    if (!workingImageData) {
      return;
    }

    const dataUrl = buildWorkingDataUrl();
    if (!dataUrl) {
      setError('Failed to build export image.');
      return;
    }

    const link = document.createElement('a');
    const stem = activeFileName.replace(/\.[^.]+$/, '') || 'edited-image';
    link.download = `${stem}-edited.png`;
    link.href = dataUrl;
    link.click();

    exportQueue = [`${formatClock(Date.now())} Exported ${link.download}`, ...exportQueue].slice(0, 60);
    setInfo(`Exported ${link.download}.`);
  }

  function compareToggle(): void {
    if (!initialImageData || !workingImageData) {
      return;
    }

    compareEnabled = !compareEnabled;
    drawEditorSurface();
    setInfo(compareEnabled ? 'Compare mode enabled.' : 'Compare mode disabled.');
  }

  async function shareDocument(): Promise<void> {
    if (!workingImageData) {
      return;
    }

    const sharePayload = JSON.stringify({
      name: activeFileName,
      width: workingImageData.width,
      height: workingImageData.height,
      timestamp: Date.now()
    });
    const shareToken = btoa(sharePayload);
    const shareText = `opencanvas://share/${shareToken}`;

    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(shareText);
        setInfo('Share token copied to clipboard.');
      } else {
        setError('Clipboard API is unavailable in this environment.');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to copy share token.';
      setError(message);
    }
  }

  async function removeBackground(): Promise<void> {
    if (!workingImageData || isRemovingBackground) {
      return;
    }

    const encoded = buildWorkingDataUrl()?.split(',')[1];
    if (!encoded) {
      setError('Could not serialize the current image for local AI processing.');
      return;
    }

    const jobId = startBatchJob('AI background removal');
    isRemovingBackground = true;
    backgroundError = null;
    backgroundStatus = null;

    try {
      pushPluginLog('AI Assist invoked for background removal.');
      const result = await apiClient.removeBackground(encoded);
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

      const next = context.getImageData(0, 0, width, height);
      workingImageData = cloneImageData(next);
      initialImageData = cloneImageData(next);
      selectionRect = null;
      compareEnabled = false;
      drawEditorSurface();

      completeBatchJob(jobId, 'done', `Provider ${result.provider}, ${result.processing_ms}ms`);
      backgroundStatus = `AI Assist complete via ${result.provider} in ${result.processing_ms}ms.`;
      pushHistory(backgroundStatus);
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : 'Local AI background removal failed. Ensure the AI service is running.';
      completeBatchJob(jobId, 'failed', message);
      setError(message);
    } finally {
      isRemovingBackground = false;
    }
  }

  function applyBrushPreset(preset: 'technical' | 'airbrush' | 'retouch'): void {
    if (preset === 'technical') {
      brushSize = 12;
      brushHardness = 90;
      brushOpacity = 100;
      brushFlow = 100;
      blendMode = 'Normal';
    } else if (preset === 'airbrush') {
      brushSize = 36;
      brushHardness = 30;
      brushOpacity = 70;
      brushFlow = 60;
      blendMode = 'Screen';
    } else {
      brushSize = 18;
      brushHardness = 55;
      brushOpacity = 88;
      brushFlow = 72;
      blendMode = 'Overlay';
    }

    setInfo(`Loaded ${preset} preset.`);
  }

  function runMacro(macro: 'auto-tone' | 'soft-focus' | 'clear-selection'): void {
    if (macro === 'clear-selection') {
      selectionRect = null;
      drawEditorSurface();
      setInfo('Selection cleared.');
      return;
    }

    if (!workingImageData) {
      return;
    }

    if (macro === 'auto-tone') {
      blackPoint = 8;
      whitePoint = 245;
      hueShift = 0;
      saturationPercent = 108;
      blurRadius = 0;
      applyAdjustments();
      setInfo('Macro applied: Auto Tone.');
      return;
    }

    blackPoint = 4;
    whitePoint = 252;
    hueShift = -2;
    saturationPercent = 102;
    blurRadius = 1;
    applyAdjustments();
    setInfo('Macro applied: Soft Focus.');
  }

  function showShortcutHelp(): void {
    bottomTrayTab = 'console';
    setInfo('Shortcuts: B/E/H/J/R/O tools, M/L/C selections, Cmd/Ctrl+S save, Cmd/Ctrl+E export.');
  }

  function addComment(): void {
    const text = commentDraft.trim();
    if (!text) {
      return;
    }

    comments = [`${formatClock(Date.now())} ${text}`, ...comments].slice(0, 40);
    commentDraft = '';
    setInfo('Comment added to inspector notes.');
  }

  async function runQuickAction(action: QuickAction): Promise<void> {
    if (action === 'new') {
      createNewDocument();
      return;
    }

    if (action === 'open') {
      openFilePicker();
      return;
    }

    if (action === 'save') {
      saveDocument();
      return;
    }

    if (action === 'export') {
      exportImage();
      return;
    }

    if (action === 'undo') {
      undo();
      return;
    }

    if (action === 'redo') {
      redo();
      return;
    }

    if (action === 'compare') {
      compareToggle();
      return;
    }

    if (action === 'share') {
      await shareDocument();
      return;
    }

    await removeBackground();
  }

  async function executeCommand(command: CommandItem): Promise<void> {
    activeMenu = null;
    commandSearch = '';
    await command.run();
  }

  function handleSearchKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      const first = filteredCommands[0];
      if (first) {
        void executeCommand(first);
      }
      return;
    }

    if (event.key === 'Escape') {
      commandSearch = '';
      activeMenu = null;
    }
  }

  function toggleMenu(menu: MenuName): void {
    activeMenu = activeMenu === menu ? null : menu;
  }
</script>

<div class="opencanvas" aria-live="polite" data-hydrated={hydrated ? 'true' : 'false'}>
  <input bind:this={openInputEl} class="hidden-input" type="file" accept="image/*" onchange={importImage} />

  <header class="menu-bar">
    <div class="menu-left">
      <strong class="brand">OpenCanvas</strong>

      <nav class="menu-cluster" aria-label="Main menu">
        {#each TOP_MENUS as menu}
          <div class="menu-slot">
            <button
              class="menu-button"
              class:active={activeMenu === menu}
              aria-haspopup="menu"
              aria-expanded={activeMenu === menu}
              onclick={() => toggleMenu(menu)}
            >
              {menu}
            </button>

            {#if activeMenu === menu}
              <div class="menu-dropdown" role="menu" aria-label={`${menu} menu`}>
                {#each activeMenuCommands as command}
                  <button class="menu-command" role="menuitem" onclick={() => void executeCommand(command)}>
                    {command.label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </nav>
    </div>

    <div class="search-cluster">
      <input
        type="search"
        bind:value={commandSearch}
        placeholder="Search commands"
        aria-label="Search command palette"
        onkeydown={handleSearchKeydown}
      />

      {#if filteredCommands.length > 0}
        <div class="search-results" role="listbox" aria-label="Command search results">
          {#each filteredCommands as command, index}
            <button
              class="search-result"
              role="option"
              aria-selected={index === 0}
              onclick={() => void executeCommand(command)}
            >
              {command.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </header>

  <section class="quick-actions" aria-label="Quick actions">
    <span class="row-label">Quick Actions</span>
    <button class="quick-button" onclick={() => void runQuickAction('new')}>New</button>
    <button class="quick-button" onclick={() => void runQuickAction('open')}>Open</button>
    <button class="quick-button" onclick={() => void runQuickAction('save')} disabled={!hasImage}>Save</button>
    <button class="quick-button" onclick={() => void runQuickAction('export')} disabled={!hasImage}>Export</button>
    <button class="quick-button" onclick={() => void runQuickAction('undo')} disabled={undoStack.length === 0}>Undo</button>
    <button class="quick-button" onclick={() => void runQuickAction('redo')} disabled={redoStack.length === 0}>Redo</button>
    <button class="quick-button" class:active={compareEnabled} onclick={() => void runQuickAction('compare')} disabled={!hasImage}>
      Compare
    </button>
    <button class="quick-button" onclick={() => void runQuickAction('share')} disabled={!hasImage}>Share</button>
    <button class="quick-button ai" onclick={() => void runQuickAction('ai-assist')} disabled={!hasImage || isRemovingBackground}>
      {isRemovingBackground ? 'AI Assist...' : 'AI Assist'}
    </button>
  </section>

  <section class="context-bar" aria-label="Context and tool options">
    <span class="row-label">Context / Tool Options</span>

    <div class="context-pill">
      Tool
      <strong>{activeToolLabel}</strong>
    </div>

    <label>
      Size
      <input type="range" min="1" max="200" bind:value={brushSize} disabled={!hasImage} />
      <strong>{brushSize}</strong>
    </label>

    <label>
      Hardness
      <input type="range" min="0" max="100" bind:value={brushHardness} disabled={!hasImage} />
      <strong>{brushHardness}%</strong>
    </label>

    <label>
      Opacity
      <input type="range" min="1" max="100" bind:value={brushOpacity} disabled={!hasImage} />
      <strong>{brushOpacity}%</strong>
    </label>

    <label>
      Flow
      <input type="range" min="1" max="100" bind:value={brushFlow} disabled={!hasImage} />
      <strong>{brushFlow}%</strong>
    </label>

    <label>
      Blend
      <select bind:value={blendMode} disabled={!hasImage}>
        <option value="Normal">Normal</option>
        <option value="Multiply">Multiply</option>
        <option value="Screen">Screen</option>
        <option value="Overlay">Overlay</option>
      </select>
    </label>

    <button class="mini-toggle" class:active={leftDockVisible} onclick={() => (leftDockVisible = !leftDockVisible)}>
      Left Dock
    </button>
    <button class="mini-toggle" class:active={rightDockVisible} onclick={() => (rightDockVisible = !rightDockVisible)}>
      Right Dock
    </button>
  </section>

  {#if backgroundError}
    <p class="notice error" role="alert">{backgroundError}</p>
  {:else if backgroundStatus}
    <p class="notice" role="status">{backgroundStatus}</p>
  {/if}

  <section
    class="workspace-grid"
    style={`grid-template-columns: ${leftDockVisible ? '86px 230px ' : '86px '}minmax(0, 1fr)${rightDockVisible ? ' 325px' : ''};`}
  >
    <aside class="left-tool-bar" aria-label="Left tool bar">
      {#each TOOL_BUTTONS as tool}
        <button
          class="tool-button"
          class:active={activeTool === tool.id}
          title={`${tool.label} (${tool.hotkey})`}
          aria-label={tool.label}
          onclick={() => selectTool(tool.id)}
        >
          <span class="tool-short">{tool.short}</span>
          <span class="tool-name">{tool.label}</span>
          <span class="tool-hotkey">{tool.hotkey}</span>
        </button>
      {/each}
    </aside>

    {#if leftDockVisible}
      <aside class="left-dock" aria-label="Left dock">
        <div class="dock-tabs compact">
          <button class:active={leftDockTab === 'navigator'} onclick={() => (leftDockTab = 'navigator')}>Navigator</button>
          <button class:active={leftDockTab === 'history'} onclick={() => (leftDockTab = 'history')}>History</button>
          <button class:active={leftDockTab === 'presets'} onclick={() => (leftDockTab = 'presets')}>Presets</button>
          <button class:active={leftDockTab === 'macros'} onclick={() => (leftDockTab = 'macros')}>Macros</button>
        </div>

        {#if leftDockTab === 'navigator'}
          <div class="dock-panel">
            <h3>Navigator</h3>
            <p>{activeFileName}</p>
            <p>Canvas {canvasDimensions}</p>
            <p>Zoom {zoomPercent}%</p>
            <button class="panel-button" onclick={() => (zoomPercent = 100)}>Reset Zoom</button>
            <button class="panel-button" onclick={() => { canvasPanX = 0; canvasPanY = 0; }}>Center View</button>
          </div>
        {:else if leftDockTab === 'history'}
          <div class="dock-panel">
            <h3>History Mini</h3>
            <ul class="simple-list">
              {#each historyLog.slice(0, 10) as line}
                <li>{line}</li>
              {/each}
            </ul>
          </div>
        {:else if leftDockTab === 'presets'}
          <div class="dock-panel">
            <h3>Tool Presets</h3>
            <button class="panel-button" onclick={() => applyBrushPreset('technical')}>Technical Ink</button>
            <button class="panel-button" onclick={() => applyBrushPreset('airbrush')}>Airbrush Cloud</button>
            <button class="panel-button" onclick={() => applyBrushPreset('retouch')}>Retouch Mix</button>
          </div>
        {:else}
          <div class="dock-panel">
            <h3>Actions / Macros</h3>
            <button class="panel-button" onclick={() => runMacro('auto-tone')}>Macro: Auto Tone</button>
            <button class="panel-button" onclick={() => runMacro('soft-focus')}>Macro: Soft Focus</button>
            <button class="panel-button" onclick={() => runMacro('clear-selection')}>Macro: Clear Selection</button>
          </div>
        {/if}
      </aside>
    {/if}

    <section class="main-workspace" aria-label="Main workspace">
      <div class="document-tabs">
        {#each documents as document}
          <button
            class="document-tab"
            class:active={document.id === activeDocumentId}
            onclick={() => setActiveDocument(document.id)}
          >
            {document.name}
          </button>
        {/each}
        <button class="document-tab add" onclick={createNewDocument}>+</button>
      </div>

      <div class="guide-bar">
        <button class="guide-toggle" class:active={showRulers} onclick={() => (showRulers = !showRulers)}>Rulers</button>
        <button class="guide-toggle" class:active={showGuides} onclick={() => (showGuides = !showGuides)}>Guides</button>
        <button class="guide-toggle" class:active={showBleed} onclick={() => (showBleed = !showBleed)}>Bleed</button>
        <button class="guide-toggle" class:active={showSafeArea} onclick={() => (showSafeArea = !showSafeArea)}>Safe Area</button>
        <div class="grow"></div>
        <button class="guide-toggle" onclick={() => createSnapshot('Manual snapshot')} disabled={!hasImage}>Snapshot</button>
      </div>

      <div class="canvas-stage">
        {#if compareEnabled}
          <p class="compare-pill">Compare mode: displaying baseline image</p>
        {/if}

        {#if showRulers}
          <div class="ruler-top" aria-hidden="true"></div>
          <div class="ruler-left" aria-hidden="true"></div>
        {/if}

        <div class="canvas-scroll" class:with-rulers={showRulers}>
          <div class="canvas-transform" style={`transform: translate(${canvasPanX}px, ${canvasPanY}px) scale(${zoomScale});`}>
            <canvas
              bind:this={canvasEl}
              aria-label="Canvas"
              onpointerdown={handlePointerDown}
              onpointermove={handlePointerMove}
              onpointerup={handlePointerUp}
              onpointerleave={handlePointerUp}
            ></canvas>

            {#if hasImage && showGuides}
              <div class="canvas-guide vertical"></div>
              <div class="canvas-guide horizontal"></div>
            {/if}

            {#if hasImage && showBleed}
              <div class="bleed-frame"></div>
            {/if}

            {#if hasImage && showSafeArea}
              <div class="safe-frame"></div>
            {/if}
          </div>

          {#if !hasImage}
            <div class="empty-state">
              <h2>Open or create a document</h2>
              <p>Use New or Open from the quick actions row to start editing.</p>
            </div>
          {/if}
        </div>
      </div>

      <div class="status-strip">
        <p>Zoom {zoomPercent}% | RGB/8 | {canvasDimensions} | GPU</p>
        <p>Cursor {cursorX}/{cursorY} | Selection {selectionLabel} | Memory {memoryLabel}</p>
      </div>
    </section>

    {#if rightDockVisible}
      <aside class="right-dock" aria-label="Right dock">
        <div class="dock-tabs right">
          {#each RIGHT_DOCK_TABS as tab}
            <button class:active={rightDockTab === tab.id} onclick={() => (rightDockTab = tab.id)}>{tab.label}</button>
          {/each}
        </div>

        <div class="dock-panel right-content">
          {#if rightDockTab === 'color'}
            <h3>Color</h3>
            <label>
              Foreground
              <input type="color" bind:value={brushColor} disabled={!hasImage} />
            </label>
            <p>{brushColor.toUpperCase()}</p>
          {:else if rightDockTab === 'swatches'}
            <h3>Swatches</h3>
            <div class="swatch-grid">
              {#each SWATCHES as swatch}
                <button class="swatch" style={`background: ${swatch};`} title={swatch} onclick={() => (brushColor = swatch)}></button>
              {/each}
            </div>
          {:else if rightDockTab === 'gradients'}
            <h3>Gradients</h3>
            <ul class="simple-list">
              {#each GRADIENTS as gradient}
                <li>{gradient}</li>
              {/each}
            </ul>
          {:else if rightDockTab === 'brushes'}
            <h3>Brushes</h3>
            <label>
              Size
              <input type="range" min="1" max="200" bind:value={brushSize} disabled={!hasImage} />
            </label>
            <label>
              Hardness
              <input type="range" min="0" max="100" bind:value={brushHardness} disabled={!hasImage} />
            </label>
            <label>
              Opacity
              <input type="range" min="1" max="100" bind:value={brushOpacity} disabled={!hasImage} />
            </label>
          {:else if rightDockTab === 'layers'}
            <h3>Layers</h3>
            <ul class="layer-list">
              <li class="layer-row active"><span class="dot"></span><span>Base raster layer</span><span>Live</span></li>
              {#if backgroundStatus}
                <li class="layer-row"><span class="dot"></span><span>AI background pass</span><span>Latest</span></li>
              {/if}
              <li class="layer-row"><span class="dot"></span><span>Selection overlay</span><span>UI</span></li>
            </ul>
          {:else if rightDockTab === 'channels'}
            <h3>Channels</h3>
            <ul class="simple-list">
              <li>RGB Composite</li>
              <li>Red</li>
              <li>Green</li>
              <li>Blue</li>
              <li>Alpha</li>
            </ul>
          {:else if rightDockTab === 'paths'}
            <h3>Paths</h3>
            <p>{penAnchor ? `Last anchor at ${penAnchor.x}, ${penAnchor.y}` : 'No active path anchor.'}</p>
          {:else if rightDockTab === 'properties'}
            <h3>Properties</h3>
            <p>Document {activeFileName}</p>
            <p>Canvas {canvasDimensions}</p>
            <p>Tool {activeToolLabel}</p>
            <p>Undo {undoStack.length} / Redo {redoStack.length}</p>
          {:else if rightDockTab === 'adjustments'}
            <h3>Adjustments</h3>
            <label>
              Black point {blackPoint}
              <input type="range" min="0" max="254" bind:value={blackPoint} disabled={!hasImage} />
            </label>
            <label>
              White point {whitePoint}
              <input type="range" min="1" max="255" bind:value={whitePoint} disabled={!hasImage} />
            </label>
            <label>
              Hue shift {hueShift}
              <input type="range" min="-180" max="180" bind:value={hueShift} disabled={!hasImage} />
            </label>
            <label>
              Saturation {saturationPercent}%
              <input type="range" min="0" max="200" bind:value={saturationPercent} disabled={!hasImage} />
            </label>
            <label>
              Blur radius {blurRadius}
              <input type="range" min="0" max="8" bind:value={blurRadius} disabled={!hasImage} />
            </label>
            <button class="panel-button" onclick={applyAdjustments} disabled={!hasImage}>Apply adjustments</button>
          {:else if rightDockTab === 'history'}
            <h3>History</h3>
            <ul class="simple-list dense">
              {#each historyLog.slice(0, 24) as line}
                <li>{line}</li>
              {/each}
            </ul>
          {:else if rightDockTab === 'assets'}
            <h3>Assets</h3>
            <ul class="simple-list">
              <li>Brand logo placeholder</li>
              <li>CTA icon set</li>
              <li>Texture collection</li>
            </ul>
          {:else if rightDockTab === 'export'}
            <h3>Export Queue</h3>
            <button class="panel-button" onclick={exportImage} disabled={!hasImage}>Queue PNG Export</button>
            <ul class="simple-list dense">
              {#if exportQueue.length === 0}
                <li>No exports queued yet.</li>
              {:else}
                {#each exportQueue.slice(0, 20) as entry}
                  <li>{entry}</li>
                {/each}
              {/if}
            </ul>
          {:else if rightDockTab === 'inspector'}
            <h3>Inspector</h3>
            <p>Rulers {showRulers ? 'On' : 'Off'}</p>
            <p>Guides {showGuides ? 'On' : 'Off'}</p>
            <p>Bleed {showBleed ? 'On' : 'Off'}</p>
            <p>Safe area {showSafeArea ? 'On' : 'Off'}</p>
          {:else}
            <h3>Comments</h3>
            <div class="comment-entry">
              <input type="text" bind:value={commentDraft} placeholder="Add comment" />
              <button class="panel-button" onclick={addComment}>Add</button>
            </div>
            <ul class="simple-list dense">
              {#each comments as comment}
                <li>{comment}</li>
              {/each}
            </ul>
          {/if}
        </div>
      </aside>
    {/if}
  </section>

  <footer class="bottom-tray" aria-label="Bottom tray">
    <div class="tray-tabs">
      {#each BOTTOM_TRAY_TABS as tab}
        <button class:active={bottomTrayTab === tab.id} onclick={() => (bottomTrayTab = tab.id)}>{tab.label}</button>
      {/each}
    </div>

    <div class="tray-content">
      {#if bottomTrayTab === 'timeline'}
        <ul class="simple-list dense">
          {#each historyLog.slice(0, 30) as line}
            <li>{line}</li>
          {/each}
        </ul>
      {:else if bottomTrayTab === 'versions'}
        <ul class="simple-list">
          {#each documents as document}
            <li>{document.name} | saved {formatClock(document.savedAt)}</li>
          {/each}
        </ul>
      {:else if bottomTrayTab === 'snapshots'}
        <div class="snapshot-grid">
          {#if snapshots.length === 0}
            <p>No snapshots yet.</p>
          {:else}
            {#each snapshots as snapshot}
              <article class="snapshot-card">
                <p>{snapshot.label}</p>
                <p>{formatClock(snapshot.createdAt)}</p>
                <button class="panel-button" onclick={() => restoreSnapshot(snapshot.id)}>Restore</button>
              </article>
            {/each}
          {/if}
        </div>
      {:else if bottomTrayTab === 'console'}
        <pre class="console-log">{historyLog.slice(0, 40).join('\n')}</pre>
      {:else if bottomTrayTab === 'batch-jobs'}
        <ul class="simple-list dense">
          {#if batchJobs.length === 0}
            <li>No batch jobs yet.</li>
          {:else}
            {#each batchJobs as job}
              <li>{job.label} | {job.status} | {job.detail}</li>
            {/each}
          {/if}
        </ul>
      {:else}
        <pre class="console-log">{pluginLogs.slice(0, 40).join('\n')}</pre>
      {/if}
    </div>
  </footer>
</div>

<style>
  :global(body) {
    margin: 0;
    min-height: 100vh;
    color: #d6deea;
    font-family: 'IBM Plex Sans', 'Avenir Next', 'Segoe UI', sans-serif;
    background:
      radial-gradient(circle at 20% -10%, rgba(71, 122, 186, 0.22), transparent 45%),
      radial-gradient(circle at 90% 0%, rgba(245, 129, 67, 0.18), transparent 38%),
      linear-gradient(180deg, #141a23 0%, #0f141b 100%);
  }

  .opencanvas {
    min-height: 100vh;
    display: grid;
    grid-template-rows: auto auto auto auto 1fr auto;
    background: rgba(14, 19, 27, 0.94);
    border: 1px solid #283242;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
  }

  .menu-bar,
  .quick-actions,
  .context-bar,
  .notice,
  .bottom-tray {
    border-bottom: 1px solid #283242;
  }

  .menu-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.45rem 0.8rem;
    background: #121a25;
    position: relative;
    z-index: 12;
  }

  .menu-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-width: 0;
  }

  .brand {
    color: #f2f5fb;
    letter-spacing: 0.02em;
    font-size: 0.93rem;
    white-space: nowrap;
  }

  .menu-cluster {
    display: flex;
    align-items: center;
    gap: 0.15rem;
    flex-wrap: wrap;
  }

  .menu-slot {
    position: relative;
  }

  .menu-button {
    border: 1px solid transparent;
    border-radius: 0.42rem;
    background: transparent;
    color: #cfd8e5;
    padding: 0.3rem 0.5rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .menu-button:hover,
  .menu-button.active {
    color: #ffffff;
    background: #1c2735;
    border-color: #2a3a4d;
  }

  .menu-dropdown {
    position: absolute;
    top: calc(100% + 0.32rem);
    left: 0;
    min-width: 220px;
    background: #172231;
    border: 1px solid #304257;
    border-radius: 0.5rem;
    padding: 0.25rem;
    display: grid;
    gap: 0.15rem;
    z-index: 30;
    box-shadow: 0 16px 36px -20px rgba(0, 0, 0, 0.7);
  }

  .menu-command {
    text-align: left;
    border: 1px solid transparent;
    border-radius: 0.38rem;
    background: transparent;
    color: #d6e0ec;
    font-size: 0.76rem;
    padding: 0.33rem 0.45rem;
    cursor: pointer;
  }

  .menu-command:hover {
    background: #223145;
    border-color: #365070;
  }

  .search-cluster {
    position: relative;
    min-width: 220px;
    width: min(340px, 42vw);
  }

  .search-cluster input {
    width: 100%;
    background: #0f1823;
    border: 1px solid #2c3d52;
    border-radius: 0.42rem;
    color: #deebff;
    padding: 0.42rem 0.55rem;
    font-size: 0.78rem;
  }

  .search-results {
    position: absolute;
    top: calc(100% + 0.28rem);
    left: 0;
    right: 0;
    background: #152131;
    border: 1px solid #324760;
    border-radius: 0.45rem;
    display: grid;
    padding: 0.2rem;
    z-index: 32;
  }

  .search-result {
    border: 0;
    background: transparent;
    color: #d6e0ec;
    text-align: left;
    border-radius: 0.3rem;
    padding: 0.36rem 0.45rem;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .search-result:hover {
    background: #243248;
  }

  .quick-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: wrap;
    padding: 0.43rem 0.78rem;
    background: #151f2d;
  }

  .row-label {
    font-size: 0.72rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #90a0b8;
    font-weight: 700;
    margin-right: 0.22rem;
  }

  .quick-button {
    border: 1px solid #34465e;
    border-radius: 0.38rem;
    background: #1d2a3a;
    color: #dde7f3;
    font-size: 0.74rem;
    font-weight: 700;
    padding: 0.34rem 0.5rem;
    cursor: pointer;
    transition: transform 120ms ease, background-color 120ms ease, border-color 120ms ease;
  }

  .quick-button:hover {
    transform: translateY(-1px);
    background: #25364c;
    border-color: #456182;
  }

  .quick-button.active {
    background: #29556f;
    border-color: #5da9d3;
  }

  .quick-button.ai {
    background: linear-gradient(120deg, #1f6f8f 0%, #2f8d7d 100%);
    border-color: #5ec4af;
  }

  .quick-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    transform: none;
  }

  .context-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    padding: 0.42rem 0.78rem;
    background: #101925;
  }

  .context-bar label {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.72rem;
    color: #a6b6cb;
    background: #172333;
    border: 1px solid #2a3d54;
    border-radius: 0.4rem;
    padding: 0.26rem 0.42rem;
  }

  .context-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.72rem;
    color: #a6b6cb;
    background: #172333;
    border: 1px solid #2a3d54;
    border-radius: 0.4rem;
    padding: 0.26rem 0.42rem;
  }

  .context-bar label strong {
    color: #eaf0f7;
    font-size: 0.73rem;
  }

  .context-pill strong {
    color: #eaf0f7;
    font-size: 0.73rem;
  }

  .context-bar input[type='range'] {
    width: 86px;
  }

  .context-bar select {
    background: #101a27;
    color: #d9e4f3;
    border: 1px solid #354861;
    border-radius: 0.32rem;
    font-size: 0.72rem;
    padding: 0.16rem 0.22rem;
  }

  .mini-toggle {
    border: 1px solid #355070;
    border-radius: 0.4rem;
    background: #1a2a3e;
    color: #c6d7ec;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.28rem 0.45rem;
    cursor: pointer;
  }

  .mini-toggle.active {
    background: #244464;
  }

  .notice {
    margin: 0;
    padding: 0.33rem 0.78rem;
    background: #172333;
    color: #93d0ff;
    font-size: 0.74rem;
    font-weight: 600;
  }

  .notice.error {
    color: #ff9ea7;
    background: #2a1b24;
  }

  .workspace-grid {
    min-height: 0;
    display: grid;
    align-items: stretch;
    gap: 1px;
    background: #283242;
    flex: 1;
  }

  .left-tool-bar,
  .left-dock,
  .main-workspace,
  .right-dock {
    background: #111923;
    min-height: 0;
  }

  .left-tool-bar {
    display: grid;
    gap: 0.28rem;
    align-content: start;
    padding: 0.45rem 0.35rem;
    overflow-y: auto;
  }

  .tool-button {
    border: 1px solid #2a394d;
    border-radius: 0.44rem;
    background: #192433;
    color: #d8e4f3;
    display: grid;
    gap: 0.04rem;
    justify-items: center;
    padding: 0.32rem 0.2rem;
    cursor: pointer;
  }

  .tool-button:hover {
    border-color: #446182;
  }

  .tool-button.active {
    background: #245070;
    border-color: #5ca8d3;
  }

  .tool-short {
    font-size: 0.58rem;
    font-weight: 800;
    letter-spacing: 0.04em;
  }

  .tool-name {
    font-size: 0.62rem;
    font-weight: 700;
  }

  .tool-hotkey {
    font-size: 0.56rem;
    color: #8fa7c4;
    font-weight: 700;
  }

  .left-dock,
  .right-dock {
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 0;
  }

  .dock-tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0.35rem;
    border-bottom: 1px solid #27374a;
    background: #141f2e;
    flex-wrap: wrap;
  }

  .dock-tabs.compact button {
    font-size: 0.68rem;
  }

  .dock-tabs.right {
    max-height: 132px;
    overflow-y: auto;
  }

  .dock-tabs button {
    border: 1px solid #31465f;
    border-radius: 0.35rem;
    background: #1a2738;
    color: #bfd0e5;
    padding: 0.25rem 0.38rem;
    font-size: 0.67rem;
    font-weight: 700;
    cursor: pointer;
  }

  .dock-tabs button.active {
    background: #255377;
    border-color: #6ab3de;
    color: #ffffff;
  }

  .dock-panel {
    min-height: 0;
    overflow-y: auto;
    display: grid;
    align-content: start;
    gap: 0.48rem;
    padding: 0.55rem;
    font-size: 0.76rem;
  }

  .dock-panel h3 {
    margin: 0;
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #8fa7c6;
  }

  .dock-panel p {
    margin: 0;
    color: #d0dced;
  }

  .panel-button {
    border: 1px solid #38526d;
    border-radius: 0.35rem;
    background: #1f3349;
    color: #dbe8f8;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.32rem 0.44rem;
    cursor: pointer;
  }

  .panel-button:hover {
    border-color: #5c86af;
  }

  .panel-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .main-workspace {
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    min-height: 0;
  }

  .document-tabs {
    display: flex;
    gap: 0.23rem;
    padding: 0.36rem;
    border-bottom: 1px solid #273748;
    background: #121c29;
    overflow-x: auto;
  }

  .document-tab {
    border: 1px solid #324960;
    border-radius: 0.35rem;
    background: #1b2a3b;
    color: #c9d8ea;
    font-size: 0.73rem;
    font-weight: 700;
    padding: 0.32rem 0.5rem;
    white-space: nowrap;
    cursor: pointer;
  }

  .document-tab.active {
    border-color: #69add9;
    background: #254f6f;
    color: #ffffff;
  }

  .document-tab.add {
    min-width: 2rem;
  }

  .guide-bar {
    border-bottom: 1px solid #273748;
    background: #101825;
    display: flex;
    align-items: center;
    gap: 0.28rem;
    padding: 0.35rem;
    flex-wrap: wrap;
  }

  .guide-toggle {
    border: 1px solid #35506d;
    border-radius: 0.35rem;
    background: #1a2b3f;
    color: #c4d5ea;
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.25rem 0.42rem;
    cursor: pointer;
  }

  .guide-toggle.active {
    background: #2a5677;
  }

  .grow {
    flex: 1;
  }

  .canvas-stage {
    position: relative;
    min-height: 0;
    background:
      radial-gradient(circle at 20% 12%, rgba(53, 76, 105, 0.34), transparent 40%),
      linear-gradient(180deg, #0f1723 0%, #0b121b 100%);
    overflow: hidden;
  }

  .compare-pill {
    position: absolute;
    top: 0.45rem;
    left: 0.5rem;
    z-index: 4;
    margin: 0;
    font-size: 0.68rem;
    color: #b0dffd;
    background: rgba(19, 39, 62, 0.8);
    border: 1px solid #3f668d;
    border-radius: 999px;
    padding: 0.2rem 0.45rem;
  }

  .ruler-top,
  .ruler-left {
    position: absolute;
    z-index: 2;
    background:
      repeating-linear-gradient(
        90deg,
        rgba(160, 183, 210, 0.15) 0,
        rgba(160, 183, 210, 0.15) 1px,
        transparent 1px,
        transparent 10px
      ),
      #0f1824;
  }

  .ruler-top {
    height: 24px;
    left: 30px;
    right: 0;
    top: 0;
  }

  .ruler-left {
    width: 24px;
    left: 0;
    top: 24px;
    bottom: 0;
    background:
      repeating-linear-gradient(
        180deg,
        rgba(160, 183, 210, 0.15) 0,
        rgba(160, 183, 210, 0.15) 1px,
        transparent 1px,
        transparent 10px
      ),
      #0f1824;
  }

  .canvas-scroll {
    position: absolute;
    inset: 0;
    overflow: auto;
    padding: 0.95rem;
  }

  .canvas-scroll.with-rulers {
    padding: 2rem 1rem 1rem 2rem;
  }

  .canvas-transform {
    position: relative;
    width: fit-content;
    margin: 0 auto;
    transform-origin: top left;
  }

  canvas {
    display: block;
    max-width: none;
    border: 1px solid #3c4f66;
    background:
      linear-gradient(45deg, #e1e6f0 25%, transparent 25%),
      linear-gradient(-45deg, #e1e6f0 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #e1e6f0 75%),
      linear-gradient(-45deg, transparent 75%, #e1e6f0 75%);
    background-size: 18px 18px;
    background-position: 0 0, 0 9px, 9px -9px, -9px 0;
    touch-action: none;
  }

  .canvas-guide {
    position: absolute;
    pointer-events: none;
    background: rgba(76, 185, 255, 0.5);
  }

  .canvas-guide.vertical {
    top: 0;
    bottom: 0;
    width: 1px;
    left: 50%;
  }

  .canvas-guide.horizontal {
    left: 0;
    right: 0;
    height: 1px;
    top: 50%;
  }

  .bleed-frame,
  .safe-frame {
    position: absolute;
    pointer-events: none;
    inset: 0;
    border: 1px dashed rgba(247, 146, 85, 0.72);
  }

  .safe-frame {
    inset: 9%;
    border-color: rgba(122, 224, 189, 0.75);
  }

  .empty-state {
    margin: 3.4rem auto;
    max-width: 420px;
    text-align: center;
    border: 1px dashed #355174;
    border-radius: 0.6rem;
    background: rgba(19, 30, 44, 0.82);
    padding: 1rem;
    color: #bcd0e8;
  }

  .empty-state h2 {
    margin: 0;
    font-size: 1rem;
  }

  .empty-state p {
    margin: 0.42rem 0 0;
    font-size: 0.82rem;
  }

  .status-strip {
    border-top: 1px solid #29394d;
    background: #121c29;
    padding: 0.35rem 0.52rem;
    display: grid;
    gap: 0.2rem;
  }

  .status-strip p {
    margin: 0;
    font-size: 0.72rem;
    color: #9fb0c6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .right-content label {
    display: grid;
    gap: 0.25rem;
    font-size: 0.72rem;
    color: #bfd0e5;
  }

  .right-content input,
  .comment-entry input {
    background: #0f1926;
    border: 1px solid #30445b;
    border-radius: 0.35rem;
    color: #d9e6f8;
    font-size: 0.74rem;
    padding: 0.3rem 0.4rem;
  }

  .right-content input[type='range'] {
    padding: 0;
  }

  .simple-list {
    margin: 0;
    padding-left: 1rem;
    display: grid;
    gap: 0.3rem;
    color: #c8d7eb;
  }

  .simple-list.dense {
    gap: 0.18rem;
  }

  .swatch-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.3rem;
  }

  .swatch {
    border: 1px solid #31475e;
    border-radius: 0.28rem;
    min-height: 1.8rem;
    cursor: pointer;
  }

  .layer-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.26rem;
  }

  .layer-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 0.36rem;
    border: 1px solid #2e4158;
    border-radius: 0.35rem;
    background: #172435;
    padding: 0.32rem 0.38rem;
    font-size: 0.69rem;
  }

  .layer-row.active {
    border-color: #5ea9d8;
    background: #254d6b;
  }

  .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 999px;
    background: #5bd1ff;
  }

  .comment-entry {
    display: flex;
    gap: 0.3rem;
    align-items: center;
  }

  .comment-entry input {
    flex: 1;
  }

  .bottom-tray {
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 140px;
    background: #101926;
  }

  .tray-tabs {
    display: flex;
    gap: 0.28rem;
    flex-wrap: wrap;
    padding: 0.36rem;
    border-bottom: 1px solid #27394c;
  }

  .tray-tabs button {
    border: 1px solid #314861;
    border-radius: 0.34rem;
    background: #1a2a3c;
    color: #bfd0e7;
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.26rem 0.4rem;
    cursor: pointer;
  }

  .tray-tabs button.active {
    background: #29577a;
    border-color: #70b4df;
  }

  .tray-content {
    min-height: 0;
    overflow: auto;
    padding: 0.5rem;
    font-size: 0.74rem;
  }

  .snapshot-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 0.42rem;
  }

  .snapshot-card {
    border: 1px solid #30465f;
    border-radius: 0.45rem;
    background: #172435;
    padding: 0.45rem;
    display: grid;
    gap: 0.32rem;
  }

  .snapshot-card p {
    margin: 0;
  }

  .console-log {
    margin: 0;
    white-space: pre-wrap;
    font-family: 'IBM Plex Mono', 'SFMono-Regular', Consolas, monospace;
    color: #b8cde6;
    font-size: 0.72rem;
  }

  .hidden-input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
  }

  button:focus-visible,
  input:focus-visible,
  select:focus-visible {
    outline: 2px solid #5dc5ff;
    outline-offset: 2px;
  }

  @media (max-width: 1280px) {
    .workspace-grid {
      grid-template-columns: 76px minmax(0, 1fr) !important;
    }

    .left-dock,
    .right-dock {
      display: none;
    }

    .search-cluster {
      width: min(280px, 44vw);
    }
  }

  @media (max-width: 920px) {
    .menu-bar {
      flex-direction: column;
      align-items: stretch;
      gap: 0.5rem;
    }

    .menu-left {
      flex-wrap: wrap;
      gap: 0.5rem;
    }

    .search-cluster {
      width: 100%;
      min-width: 0;
    }

    .workspace-grid {
      grid-template-columns: 1fr !important;
    }

    .left-tool-bar {
      display: flex;
      overflow-x: auto;
      gap: 0.28rem;
      padding: 0.35rem;
    }

    .tool-button {
      min-width: 72px;
    }

    .canvas-scroll.with-rulers {
      padding: 1.8rem 0.8rem 0.8rem 1.6rem;
    }
  }

  @media (max-width: 640px) {
    .quick-actions,
    .context-bar {
      gap: 0.25rem;
      padding: 0.36rem;
    }

    .row-label {
      width: 100%;
    }

    .context-bar label {
      width: calc(50% - 0.25rem);
      justify-content: space-between;
      flex-wrap: wrap;
    }

    .context-bar input[type='range'] {
      width: 100%;
    }
  }
</style>