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
		default: 'bg-mantle border-surface0/40',
		bordered: 'bg-base/40 border-surface1',
		highlight: 'bg-mantle border-blue/50'
	};
</script>

<div class="rounded-xl border shadow {variantClasses[variant]} {className}">
	{#if header}
		<div class="border-b border-surface0 p-4">
			{@render header()}
		</div>
	{/if}

	<div class="p-4">
		{@render children?.()}
	</div>

	{#if footer}
		<div class="border-t border-surface0 p-4">
			{@render footer()}
		</div>
	{/if}
</div>
