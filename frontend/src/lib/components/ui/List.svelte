<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		className?: string;
		divided?: boolean;
		children?: Snippet;
	}

	let { className = '', divided = true, children }: Props = $props();
</script>

<div class="rounded-lg border border-surface0 bg-crust overflow-hidden {className}" role="list">
	<div class={divided ? 'divide-y divide-surface0' : ''}>
		{@render children?.()}
	</div>
</div>
