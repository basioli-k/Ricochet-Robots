import { defineConfig, devices } from '@playwright/test';

/**
 * End-to-end tests against the real server: the Rust server on its default
 * port and the Vite dev server in front of it (which proxies /api).
 */
const ci = Boolean(process.env.CI);

export default defineConfig({
  testDir: 'e2e',
  // A round waits out the 60 s bidding window.
  timeout: 150_000,
  retries: ci ? 1 : 0,
  workers: 1,
  reporter: ci ? 'github' : 'list',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: [
    {
      command: 'cargo run --locked -p server',
      cwd: '..',
      url: 'http://127.0.0.1:8080/health',
      timeout: 600_000,
      reuseExistingServer: !ci,
      env: {
        RR_BIND: '127.0.0.1:8080',
        RR_DATABASE_URL: 'sqlite://target/e2e.db?mode=rwc',
        RR_PUBLIC_BASE_URL: 'http://localhost:5173',
      },
    },
    {
      command: 'npm run dev -- --port 5173 --strictPort',
      url: 'http://localhost:5173',
      reuseExistingServer: !ci,
    },
  ],
});
