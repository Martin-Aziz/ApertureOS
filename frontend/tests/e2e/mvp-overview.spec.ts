import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { expect, test } from '@playwright/test';

const thisFile = fileURLToPath(import.meta.url);
const thisDir = path.dirname(thisFile);
const overviewFileUrl = pathToFileURL(
  path.resolve(thisDir, '../../../pixelforge_mvp_overview.html')
).href;

const overviewTabs = [
  { tabLabel: 'Summary', panelId: 'summary' },
  { tabLabel: 'Features', panelId: 'features' },
  { tabLabel: 'Tech Stack', panelId: 'stack' },
  { tabLabel: 'Architecture', panelId: 'arch' },
  { tabLabel: 'Delivery', panelId: 'timeline' },
  { tabLabel: 'Risks', panelId: 'risks' }
];

test.describe('MVP overview page', () => {
  test('switches tabs and activates the matching panel', async ({ page }) => {
    await page.goto(overviewFileUrl);

    for (const { tabLabel, panelId } of overviewTabs) {
      await page.getByRole('button', { name: tabLabel, exact: true }).click();
      await expect(page.locator(`#${panelId}`)).toHaveClass(/active/);
      await expect(page.locator('.panel.active')).toHaveCount(1);
      await expect(page.locator('.tab.active')).toHaveCount(1);
    }
  });
});
