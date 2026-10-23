import { expect, test } from '@playwright/test';

test.describe('Documentation Site', () => {
  test('home page loads', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/Gala/);
  });

  test('navigation works', async ({ page }) => {
    await page.goto('/');
    await page.click('text=Language Spec');
    await expect(page).toHaveURL(/.*language-spec/);
  });

  test('search works', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="search-button"]');
    await page.fill('[data-testid="search-input"]', 'type system');
    await expect(page.locator('[data-testid="search-results"]')).toBeVisible();
  });

  test('theme toggle works', async ({ page }) => {
    await page.goto('/');
    const themeButton = page.locator('[data-testid="theme-toggle"]');
    await themeButton.click();
    await expect(page.locator('html')).toHaveClass(/dark/);
    await themeButton.click();
    await expect(page.locator('html')).not.toHaveClass(/dark/);
  });

  test('code blocks render correctly', async ({ page }) => {
    await page.goto('/docs/language-spec/type-system');
    await expect(page.locator('pre code')).toBeVisible();
  });
});
