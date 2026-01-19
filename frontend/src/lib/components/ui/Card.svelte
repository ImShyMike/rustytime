<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'bordered' | 'highlight';

	interface Props {
		variant?: Variant;
		className?: string;
		header?: Snippet;
		footer?: Snippet;
		children?: Snippet;
	}

	let { variant = 'default', className = '', header, footer, children }: Props = $props();

	const variantClasses: Record<Variant, string> = {
		default: 'bg-ctp-base border-ctp-surface0/40',
		bordered: 'bg-ctp-surface0/40 border-ctp-surface1',
		highlight: 'bg-ctp-base border-ctp-blue/50'
	};
</script>

<div class="rounded-xl border shadow {variantClasses[variant]} {className}">
	{#if header}
		<div class="border-b border-ctp-surface0 p-4">
			{@render header()}
		</div>
	{/if}

	<div class="p-4">
		{@render children?.()}
	</div>

	{#if footer}
		<div class="border-t border-ctp-surface0 p-4">
			{@render footer()}
		</div>
	{/if}
</div>
