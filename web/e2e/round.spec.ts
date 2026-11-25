import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test';

/** A fresh browser (own cookies) whose player has started as a guest. */
async function guest(context: BrowserContext, nickname: string): Promise<Page> {
  const page = await context.newPage();
  await page.goto('/');
  await page.getByLabel('Nickname').fill(nickname);
  await page.getByRole('button', { name: 'Play' }).click();
  await expect(page.getByRole('heading', { name: 'Rooms' })).toBeVisible();
  return page;
}

const browsers = (browser: Browser) =>
  Promise.all([browser.newContext({ locale: 'en-US' }), browser.newContext({ locale: 'en-US' })]);

test('two guests play a round', async ({ browser }) => {
  const contexts = await browsers(browser);
  try {
    const suffix = String(Date.now() % 100_000);
    const [ana, bob] = [
      await guest(contexts[0], `Ana${suffix}`),
      await guest(contexts[1], `Bob${suffix}`),
    ];

    // Bob joins the room Ana opened, found by the code her page shows.
    await ana.getByRole('button', { name: 'Open a room' }).click();
    const code = ((await ana.getByText(/^Room \w{6}$/).textContent()) ?? '').slice(-6);
    await bob.getByRole('button', { name: 'Refresh' }).click();
    await bob
      .getByRole('listitem')
      .filter({ hasText: `Room ${code}` })
      .getByRole('button', { name: 'Join' })
      .click();
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

    // Keys pick a robot and move it; some direction moves it (refused moves
    // don't count), and that one move may even solve the round.
    // Keys typed into a form field are text, not moves: leave the bid box.
    await bob.getByRole('heading', { name: 'Ricochet Robots' }).click();
    await bob.keyboard.press('2');
    await expect(bob.getByRole('button', { name: 'Blue robot' }).first()).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    const moved = bob.getByText(/^Moves: 1/);
    const refused = bob.getByText('That robot can’t move that way.');
    for (const key of ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight']) {
      await bob.keyboard.press(key);
      await expect(moved.or(refused)).toBeVisible();
      if (await moved.isVisible()) break;
    }
    await expect(ana.getByText(/^Moves: 1/)).toBeVisible();

    // Give up, unless that move solved it (then the button is gone, or the
    // server refuses the late give-up); either way both see the same end.
    await bob
      .getByRole('button', { name: 'Give up' })
      .click({ timeout: 2_000 })
      .catch(() => undefined);
    for (const page of [ana, bob]) {
      await expect(
        page.getByText(`Solved by Bob${suffix}`).or(page.getByText('Nobody solved it.')),
      ).toBeVisible();
    }
  } finally {
    await Promise.all(contexts.map((c) => c.close()));
  }
});
