import { expect, test } from '@playwright/test';

test.describe('Homepage', () => {
  test('loads successfully', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/Gala/);
  });

  test('has navigation', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('nav')).toBeVisible();
  });

  test('theme toggle works', async ({ page }) => {
    await page.goto('/');
    const themeToggle = page.locator('[aria-label="Toggle theme"]');
    await expect(themeToggle).toBeVisible();
    await themeToggle.click();
  });
});

test.describe('Documentation Pages', () => {
  test('language spec page loads', async ({ page }) => {
    await page.goto('/docs/language-spec');
    await expect(page.locator('h1')).toContainText('Language Specification');
  });

  test('guides page loads', async ({ page }) => {
    await page.goto('/docs/guides');
    await expect(page.locator('h1')).toContainText('Guides');
  });
});

test.describe('Search', () => {
  test('search dialog opens', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+K');
    await expect(page.locator('[role="dialog"]')).toBeVisible();
  });
});
