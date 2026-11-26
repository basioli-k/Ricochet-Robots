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

/** Ana plays on a phone (touch, narrow), Bob on a desktop. */
const browsers = (browser: Browser) =>
  Promise.all([
    browser.newContext({
      locale: 'en-US',
      viewport: { width: 390, height: 844 },
      hasTouch: true,
      isMobile: true,
    }),
    browser.newContext({ locale: 'en-US' }),
  ]);

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
    await ana.getByLabel('Moves').fill('9');
    await ana.getByRole('button', { name: 'Bid', exact: true }).click();
    await expect(bob.locator('.bids')).toContainText(`Ana${suffix}: 9`);

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
    // server refuses the late give-up). Then it's Ana's turn, on her phone.
    // (If Bob's one move solved the round, which is rare, this is skipped;
    // hiding the docked controller fails the test otherwise.)
    await bob
      .getByRole('button', { name: 'Give up' })
      .click({ timeout: 2_000 })
      .catch(() => undefined);
    const bobSolved = ana.getByText(`Solved by Bob${suffix}`);
    const anaTurn = ana.getByText('Your turn');
    await expect(bobSolved.or(anaTurn)).toBeVisible();
    if (await anaTurn.isVisible()) {
      // The controller docks to the bottom of the phone screen.
      const dock = ana.locator('.controller.docked');
      await expect(dock).toBeInViewport();
      const green = dock.getByRole('button', { name: 'Green robot' });
      await green.tap();
      await expect(green).toHaveAttribute('aria-pressed', 'true');
      const anaMoved = dock.getByText(/^1 \/ 9$/);
      const anaRefused = ana.getByText('That robot can’t move that way.');
      const anaSolved = ana.getByText(`Solved by Ana${suffix}`);
      for (const dir of ['Up', 'Right', 'Down', 'Left']) {
        await dock.getByRole('button', { name: dir, exact: true }).tap();
        await expect(anaMoved.or(anaRefused).or(anaSolved)).toBeVisible();
        if (!(await anaRefused.isVisible())) break;
      }
      await dock
        .getByRole('button', { name: 'Give up' })
        .tap({ timeout: 2_000 })
        .catch(() => undefined);
    }
    for (const page of [ana, bob]) {
      await expect(
        page.getByText(/^Solved by (Ana|Bob)/).or(page.getByText('Nobody solved it.')),
      ).toBeVisible();
    }
  } finally {
    await Promise.all(contexts.map((c) => c.close()));
  }
});
