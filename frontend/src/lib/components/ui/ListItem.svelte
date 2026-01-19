<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		className?: string;
		hoverable?: boolean;
		children?: Snippet;
		leading?: Snippet;
		trailing?: Snippet;
	}

	let { className = '', hoverable = true, children, leading, trailing }: Props = $props();
</script>

<div
	class="flex items-center gap-3 px-4 py-3 {hoverable
		? 'hover:bg-ctp-surface0/20'
		: ''} {className}"
	role="listitem"
>
	{#if leading}
		<div class="shrink-0">
			{@render leading()}
		</div>
	{/if}

	<div class="min-w-0 flex-1">
		{@render children?.()}
	</div>

	{#if trailing}
		<div class="shrink-0">
			{@render trailing()}
		</div>
	{/if}
</div>
