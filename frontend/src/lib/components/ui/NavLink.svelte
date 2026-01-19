<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import type { Snippet } from 'svelte';

	type RequiredPermission = 'default' | 'admin' | 'owner';

	interface Props {
		href: Pathname;
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

	const permissionClasses: Record<RequiredPermission, string> = {
		default: '',
		admin: 'outline-dashed bg-ctp-red/5 outline-1 outline-ctp-red',
		owner: 'outline-dashed bg-ctp-mauve/5 outline-1 outline-ctp-mauve'
	};

	const baseClasses =
		'w-full text-left cursor-pointer py-2 rounded-md items-center inline-flex transition-colors';
	const activeClasses = 'bg-ctp-surface0/70 text-ctp-lavender';
	const inactiveClasses = 'hover:bg-ctp-surface1/50';
</script>

<a
	href={resolve(href)}
	{onclick}
	data-sveltekit-preload-data="hover"
	class="{baseClasses} {active ? activeClasses : inactiveClasses} {permissionClasses[permission]} {collapsed ? 'justify-center' : 'px-3'} {className}"
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
