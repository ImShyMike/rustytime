<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		className?: string;
		divided?: boolean;
		children?: Snippet;
	}

	let { className = '', divided = true, children }: Props = $props();
</script>

<div
	class="rounded-lg border border-ctp-surface0 bg-ctp-mantle overflow-hidden {className}"
	role="list"
>
	<div class={divided ? 'divide-y divide-ctp-surface0' : ''}>
		{@render children?.()}
	</div>
</div>
