import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import path from 'node:path';

export default defineConfig({
	plugins: [
		sveltekit(),
		SvelteKitPWA({
			srcDir: './src',
			scope: '/',
			base: '/',
			pwaAssets: {
				config: true
			},
			registerType: 'autoUpdate',
			strategies: 'generateSW',
			devOptions: {
				enabled: true,
				type: 'module',
				navigateFallback: '/'
			},
			injectManifest: {
				globPatterns: ['client/**/*.{js,css,ico,png,svg,webp,woff,woff2}']
			},
			workbox: {
				globPatterns: ['client/**/*.{js,css,ico,png,svg,webp,woff,woff2}']
			},
			includeAssets: ['favicon.ico', 'apple-touch-icon.png', 'mask-icon.svg'],
			manifest: {
				name: 'MimiDo',
				start_url: '/',
				display: 'standalone',
				short_name: 'MimiDo',
				description: 'An app for organizing mimis',
				theme_color: '#9CA3AF',
				background_color: '#9CA3AF',
				icons: [
					{
						src: '/frog.jpg',
						sizes: '192x192',
						type: 'image/jpg'
					}
				]
			}
		})
	],
	resolve: {
		alias: {
			'@zoneinfo': path.resolve(__dirname, 'src/lib/server/calendar/zoneinfo')
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
