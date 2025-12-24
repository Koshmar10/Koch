import { test, expect } from '@playwright/test';

test('hello world!', async ({ page }) => {
	await page.goto('http://localhost:3000'); // Replace with your application's URL
	expect(await page.title()).toBe('Expected Title'); // Replace with the expected title
});