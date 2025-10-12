import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import Icons from 'unplugin-icons/vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, resolve('../'), '');

	return {
		plugins: [
			tailwindcss(),
			sveltekit(),
			SvelteKitPWA({
				strategies: 'generateSW',
				workbox: {
					globPatterns: [
					'client/**/*.{js,css,ico,png,svg,webp,woff,woff2,webmanifest}'
					]
				},
				manifest: {
					name: 'rustytime',
					short_name: 'rustytime',
					description: '🕒 blazingly fast time tracking for developers',
					theme_color: '#b4befe',
					icons: [
					{
						src: 'pwa-192x192.png',
						sizes: '192x192',
						type: 'image/png'
					},
					{
						src: 'pwa-512x512.png',
						sizes: '512x512',
						type: 'image/png'
					}
					]
				},
				devOptions: { enabled: true }
			}),
			Icons({
				autoInstall: true,
				compiler: 'svelte'
			})
		],
		define: {
			'process.env.PUBLIC_BACKEND_API_URL': JSON.stringify(env.PUBLIC_BACKEND_API_URL)
		}
	};
});
