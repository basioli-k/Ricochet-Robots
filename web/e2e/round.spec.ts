import { expect, test, type Browser, type Page } from '@playwright/test';

/** A fresh browser (own cookies) whose player has started as a guest. */
async function guest(browser: Browser, nickname: string): Promise<Page> {
  const page = await (await browser.newContext({ locale: 'en-US' })).newPage();
  await page.goto('/');
  await page.getByLabel('Nickname').fill(nickname);
  await page.getByRole('button', { name: 'Play' }).click();
  await expect(page.getByRole('heading', { name: 'Rooms' })).toBeVisible();
  return page;
}

test('two guests play a round', async ({ browser }) => {
  const suffix = String(Date.now() % 100_000);
  const ana = await guest(browser, `Ana${suffix}`);
  const bob = await guest(browser, `Bob${suffix}`);

  await ana.getByRole('button', { name: 'Open a room' }).click();
  await expect(ana.getByRole('button', { name: 'Start the game' })).toBeVisible();
  await bob.getByRole('button', { name: 'Refresh' }).click();
  await bob.getByRole('button', { name: 'Join' }).first().click();
  await expect(ana.locator('.scores')).toContainText(`Bob${suffix}`);

  await ana.getByRole('button', { name: 'Start the game' }).click();
  for (const page of [ana, bob]) {
    await expect(page.getByText('Find a route')).toBeVisible();
    await expect(page.locator('.board .robot')).toHaveCount(4);
  }

  await bob.getByLabel('Moves').fill('5');
  await bob.getByRole('button', { name: 'Bid', exact: true }).click();
  await expect(ana.locator('.bids')).toContainText(`Bob${suffix}: 5`);

  await ana.getByLabel('Chat').fill('good luck');
  await ana.getByRole('button', { name: 'Send' }).click();
  await expect(bob.locator('.chat')).toContainText(`Ana${suffix}: good luck`);

  // The server closes bidding once its 60 s window has passed.
  await expect(bob.getByText('Your turn')).toBeVisible({ timeout: 75_000 });
  await expect(ana.getByText(`Bob${suffix} is showing their route.`)).toBeVisible();

  // Some direction moves the selected robot; refused moves don't count.
  const moved = bob.getByText('Moves: 1 / 5');
  const refused = bob.getByText('That robot can’t move that way.');
  for (const dir of ['Up', 'Down', 'Left', 'Right']) {
    await bob.getByRole('button', { name: dir, exact: true }).click();
    await expect(moved.or(refused)).toBeVisible();
    if (await moved.isVisible()) break;
  }
  await expect(ana.getByText('Moves: 1 / 5')).toBeVisible();

  await bob.getByRole('button', { name: 'Give up' }).click();
  for (const page of [ana, bob]) {
    await expect(page.getByText('Nobody solved it.')).toBeVisible();
  }
});
