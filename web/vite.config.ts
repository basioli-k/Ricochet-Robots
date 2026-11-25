/// <reference types="vitest/config" />
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  // In development the Rust server (RR_BIND, 127.0.0.1:8080 by default)
  // answers the API; in production it serves this app itself.
  server: {
    proxy: {
      '/api': { target: 'http://127.0.0.1:8080', ws: true },
    },
  },
  test: {
    include: ['src/**/*.test.ts'],
  },
});
