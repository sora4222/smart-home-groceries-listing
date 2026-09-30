import { defineConfig } from 'vitest/config'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss(), viteReact()],
  resolve: {
    alias: { '#': new URL('./src', import.meta.url).pathname },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test-setup.ts'],
    globals: false,
    // Unit tests only. `e2e/` is Playwright's — it needs a browser and a
    // running stack, and vitest would otherwise collect those specs too.
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
  },
})
