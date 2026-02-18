<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'success' | 'warning' | 'error' | 'info';

	interface Props {
		variant?: Variant;
		className?: string;
		children?: Snippet;
		icon?: Snippet;
	}

	let { variant = 'default', className = '', children, icon }: Props = $props();

	const variantClasses: Record<Variant, string> = {
		default: 'bg-base text-text',
		success: 'bg-green/20 text-green',
		warning: 'bg-yellow/20 text-yellow',
		error: 'bg-red/20 text-red',
		info: 'bg-blue/20 text-blue'
	};
</script>

<span
	class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium {variantClasses[
		variant
	]} {className}"
>
	{#if icon}
		{@render icon()}
	{/if}
	{@render children?.()}
</span>
