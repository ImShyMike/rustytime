// Type declarations for unplugin-icons
declare module '~icons/*' {
	import type { SvelteComponent } from 'svelte';
	const component: typeof SvelteComponent;
	export default component;
}
