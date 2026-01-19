import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import Icons from 'unplugin-icons/vite';

export default defineConfig(() => {
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
			'import.meta.env.VITE_GIT_COMMIT_HASH': JSON.stringify(
				process.env.CF_PAGES_COMMIT_SHA || 'dev'
			)
		}
	};
});
