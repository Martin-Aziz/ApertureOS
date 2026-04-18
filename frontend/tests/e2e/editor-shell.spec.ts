import { expect, test, type Page } from '@playwright/test';

const sampleImagePayload = {
  name: 'editor-sample.svg',
  mimeType: 'image/svg+xml',
  buffer: Buffer.from(
    '<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80">'
      + '<rect width="120" height="80" fill="#ffd166" />'
      + '<circle cx="60" cy="40" r="24" fill="#118ab2" />'
      + '<rect x="14" y="12" width="22" height="22" fill="#ef476f" />'
      + '</svg>',
    'utf8'
  ),
};

const mockedBackgroundRemovedImageBase64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO7+2SMAAAAASUVORK5CYII=';

async function canvasSignature(page: Page): Promise<number> {
  return page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      return 0;
    }

    const context = canvas.getContext('2d');
    if (!context) {
      return 0;
    }

    const imageData = context.getImageData(0, 0, canvas.width, canvas.height).data;
    return Array.from(imageData).reduce((total, value, index) => total + value * (index + 1), 0);
  });
}

async function canvasSize(page: Page): Promise<{ width: number; height: number }> {
  return page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      return { width: 0, height: 0 };
    }

    return { width: canvas.width, height: canvas.height };
  });
}

test.describe('Editor shell flows', () => {
  test('loads directly into editor without login', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Import, retouch, add, remove, and crop.' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Start with a picture' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Export PNG' })).toBeDisabled();
  });

  test('paints, erases, heals, crops selection, and supports undo/redo', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('main[data-hydrated="true"]')).toBeVisible();
    await page.locator('#import-image').setInputFiles(sampleImagePayload);

    await expect
      .poll(async () => canvasSignature(page))
      .toBeGreaterThan(0);

    const original = await canvasSignature(page);

    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();
    if (!box) {
      throw new Error('Canvas box is unavailable');
    }

    await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.65, box.y + box.height * 0.65);
    await page.mouse.up();

    const painted = await canvasSignature(page);
    expect(painted).not.toEqual(original);

    await page.getByRole('button', { name: 'Undo' }).click();
    const afterUndo = await canvasSignature(page);
    expect(afterUndo).toEqual(original);

    await expect(page.getByRole('button', { name: 'Redo' })).toBeEnabled();
    await page.getByRole('button', { name: 'Redo' }).click();
    const afterRedo = await canvasSignature(page);
    expect(afterRedo).toEqual(painted);

    await page.getByRole('button', { name: 'Undo' }).click();
    const afterRedoUndo = await canvasSignature(page);
    expect(afterRedoUndo).toEqual(original);

    await page.mouse.move(box.x + box.width * 0.35, box.y + box.height * 0.35);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
    await page.mouse.up();

    const repainted = await canvasSignature(page);
    expect(repainted).not.toEqual(original);

    await page.getByRole('button', { name: 'Eraser' }).click();
    await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.6, box.y + box.height * 0.6);
    await page.mouse.up();

    const erased = await canvasSignature(page);
    expect(erased).not.toEqual(repainted);

    await page.getByRole('button', { name: 'Spot remove' }).click();
    await page.mouse.move(box.x + box.width * 0.58, box.y + box.height * 0.42);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.64, box.y + box.height * 0.47);
    await page.mouse.up();

    const healed = await canvasSignature(page);
    expect(healed).not.toEqual(erased);

    const cropButton = page.getByRole('button', { name: 'Crop selection' });
    await expect(cropButton).toBeDisabled();

    const beforeCropSize = await canvasSize(page);
    await page.getByRole('button', { name: 'Select', exact: true }).click();
    await page.mouse.move(box.x + box.width * 0.2, box.y + box.height * 0.2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * 0.8, box.y + box.height * 0.8);
    await page.mouse.up();
    await expect(cropButton).toBeEnabled();

    await cropButton.click();
    const afterCropSize = await canvasSize(page);
    expect(afterCropSize.width).toBeLessThanOrEqual(beforeCropSize.width);
    expect(afterCropSize.height).toBeLessThanOrEqual(beforeCropSize.height);
    expect(afterCropSize.width * afterCropSize.height).toBeGreaterThan(0);
  });

  test('imports an image, applies adjustments, resets, and exports', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('main[data-hydrated="true"]')).toBeVisible();
    await page.locator('#import-image').setInputFiles(sampleImagePayload);

    await expect(page.getByRole('button', { name: 'Export PNG' })).toBeEnabled();
    await expect
      .poll(async () => canvasSignature(page))
      .toBeGreaterThan(0);

    const initialSignature = await canvasSignature(page);

    await page.getByLabel('Black point: 0').fill('20');
    await page.getByLabel('White point: 255').fill('220');
    await page.getByLabel('Hue shift: 0°').fill('120');
    await page.getByLabel('Saturation: 100%').fill('160');
    await page.getByLabel('Blur radius: 0').fill('2');
    await page.getByRole('button', { name: 'Apply adjustments' }).click();

    const adjustedSignature = await canvasSignature(page);

    expect(adjustedSignature).not.toEqual(initialSignature);

    await page.getByRole('button', { name: 'Undo' }).click();
    const afterUndo = await canvasSignature(page);
    expect(afterUndo).toEqual(initialSignature);

    await page.getByRole('button', { name: 'Reset' }).click();

    const resetSignature = await canvasSignature(page);

    expect(resetSignature).toEqual(initialSignature);

    const download = page.waitForEvent('download');
    await page.getByRole('button', { name: 'Export PNG' }).click();
    const artifact = await download;
    expect(artifact.suggestedFilename()).toContain('editor-sample-edited.png');
  });

  test('removes background with local AI and shows success status', async ({ page }) => {
    await page.route('**/api/v1/ai/remove-background', async (route) => {
      if (route.request().method() === 'OPTIONS') {
        await route.fulfill({
          status: 204,
          headers: {
            'access-control-allow-origin': 'http://127.0.0.1:5173',
            'access-control-allow-methods': 'POST, OPTIONS',
            'access-control-allow-headers': 'content-type'
          }
        });
        return;
      }

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        headers: {
          'access-control-allow-origin': 'http://127.0.0.1:5173'
        },
        body: JSON.stringify({
          image_base64: mockedBackgroundRemovedImageBase64,
          provider: 'mock-rembg',
          processing_ms: 12
        })
      });
    });

    await page.goto('/');
    await expect(page.locator('main[data-hydrated="true"]')).toBeVisible();
    await page.locator('#import-image').setInputFiles(sampleImagePayload);

    await expect
      .poll(async () => canvasSignature(page))
      .toBeGreaterThan(0);

    await page.getByRole('button', { name: 'Remove background (local AI)' }).click();
    await expect(page.getByText('Background removed locally (mock-rembg) in 12ms.')).toBeVisible();
  });

  test('shows local AI error message when background removal fails', async ({ page }) => {
    await page.route('**/api/v1/ai/remove-background', async (route) => {
      if (route.request().method() === 'OPTIONS') {
        await route.fulfill({
          status: 204,
          headers: {
            'access-control-allow-origin': 'http://127.0.0.1:5173',
            'access-control-allow-methods': 'POST, OPTIONS',
            'access-control-allow-headers': 'content-type'
          }
        });
        return;
      }

      await route.fulfill({
        status: 503,
        contentType: 'application/json',
        headers: {
          'access-control-allow-origin': 'http://127.0.0.1:5173'
        },
        body: JSON.stringify({
          error: {
            code: 'service_unavailable',
            message: 'REMBG is unavailable in this environment'
          }
        })
      });
    });

    await page.goto('/');
    await expect(page.locator('main[data-hydrated="true"]')).toBeVisible();
    await page.locator('#import-image').setInputFiles(sampleImagePayload);

    await expect
      .poll(async () => canvasSignature(page))
      .toBeGreaterThan(0);

    await page.getByRole('button', { name: 'Remove background (local AI)' }).click();
    await expect(page.getByText('REMBG is unavailable in this environment')).toBeVisible();
  });
});
