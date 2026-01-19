<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'primary' | 'warning';

	interface Props {
		variant?: Variant;
		collapsed?: boolean;
		className?: string;
		onclick?: () => void;
		children?: Snippet;
		icon?: Snippet;
	}

	let {
		variant = 'default',
		collapsed = false,
		className = '',
		onclick,
		children,
		icon
	}: Props = $props();

	const baseClasses =
		'w-full py-2 rounded-md cursor-pointer items-center inline-flex transition-colors';

	const variantClasses: Record<Variant, string> = {
		default: 'hover:bg-ctp-surface1/50',
		primary: 'bg-ctp-mauve/50 outline outline-ctp-mauve hover:bg-ctp-mauve/65',
		warning: 'bg-ctp-yellow text-ctp-base hover:bg-ctp-yellow/80'
	};
</script>

<button
	type="button"
	class="{baseClasses} {variantClasses[variant]} {collapsed
		? 'justify-center'
		: 'px-3'} {className}"
	{onclick}
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
</button>
