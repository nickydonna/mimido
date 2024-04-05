import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import path from 'node:path';

export default defineConfig({
	plugins: [
		sveltekit(),
		SvelteKitPWA({
			strategies: 'generateSW'
		})
	],
	resolve: {
		alias: {
			"@zoneinfo": path.resolve(__dirname, 'src/lib/server/calendar/zoneinfo')
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
