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

	const permissionClasses: Record<RequiredPermission, string> = {
		default: 'outline-surface0/75',
		admin:
			'outline-dashed bg-red/5 outline-1 outline-red focus-visible:ring-2 focus-visible:ring-red/60',
		owner:
			'outline-dashed bg-mauve/5 outline-1 outline-mauve focus-visible:ring-2 focus-visible:ring-mauve/60'
	};

	const baseClasses =
		'w-full text-left outline-0 cursor-pointer py-2 rounded-md items-center inline-flex transition-colors';
	const activeClasses = 'bg-base outline-1 outline-surface0 text-lavender';
	const inactiveClasses = 'hover:bg-surface0/50 hover:outline-1';
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -->
<a
	href={resolvedHref}
	{onclick}
	data-sveltekit-preload-data="hover"
	class="{baseClasses} {active ? activeClasses : inactiveClasses} {permissionClasses[
		permission
	]} {collapsed ? 'justify-center' : 'px-3'} {className}"
>
	{#if icon}
		<span class="w-6 h-6 inline-flex items-center justify-center">
			{@render icon()}
		</span>
	{/if}
	{#if children}
		<span class={collapsed ? 'hidden' : 'ml-2'}>
			{@render children()}
		</span>
	{/if}
</a>
