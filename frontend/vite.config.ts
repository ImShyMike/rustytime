import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import Icons from 'unplugin-icons/vite';
import { resolve } from 'path';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, resolve('../'), '');

	return {
		plugins: [
			tailwindcss(),
			sveltekit(),
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
