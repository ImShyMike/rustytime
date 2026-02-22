<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import type { Snippet } from 'svelte';

	type RequiredPermission = 'default' | 'admin' | 'owner';

	interface Props {
		href: string | Pathname;
		active?: boolean;
		collapsed?: boolean;
		permission?: RequiredPermission;
		className?: string;
		onclick?: () => void;
		children?: Snippet;
		icon?: Snippet;
	}

	let {
		href,
		active = false,
		collapsed = false,
		permission = 'default',
		className = '',
		onclick,
		children,
		icon
	}: Props = $props();

	const resolvedHref = $derived(typeof href === 'string' ? href : resolve(...href));

	const permissionClasses = $derived.by(() => {
		switch (permission) {
			case 'admin':
				return `outline-dashed outline-1 outline-red focus-visible:ring-2 focus-visible:ring-red/60 ${active ? '' : 'bg-red/5'}`;
			case 'owner':
				return `outline-dashed outline-1 outline-mauve focus-visible:ring-2 focus-visible:ring-mauve/60 ${active ? '' : 'bg-mauve/5'}`;
			default:
				return active ? 'outline-1 outline-surface0' : 'outline-surface0/75';
		}
	});

	const baseClasses =
		'w-full text-left outline-0 cursor-pointer py-2 rounded-md items-center inline-flex transition-none!';
	const activeClasses = 'bg-base font-bold';
	const inactiveClasses = 'hover:bg-surface0/50 hover:outline-1';
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -->
<a
	href={resolvedHref}
	{onclick}
	data-sveltekit-preload-data="hover"
	class="{baseClasses} {active ? activeClasses : inactiveClasses} {permissionClasses} {collapsed
		? 'justify-center'
		: 'px-3'} {className}"
>
	{#if icon}
		<span
			class="w-6 h-6 inline-flex items-center justify-center transition-none! **:transition-none!"
		>
			{@render icon()}
		</span>
	{/if}
	{#if children}
		<span class={collapsed ? 'hidden' : 'ml-2'}>
			{@render children()}
		</span>
	{/if}
</a>
