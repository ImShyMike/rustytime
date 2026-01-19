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
		default: 'bg-ctp-surface0 text-ctp-text',
		success: 'bg-ctp-green/20 text-ctp-green',
		warning: 'bg-ctp-yellow/20 text-ctp-yellow',
		error: 'bg-ctp-red/20 text-ctp-red',
		info: 'bg-ctp-blue/20 text-ctp-blue'
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
