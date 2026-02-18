<script lang="ts">
	import type { Snippet } from 'svelte';
	import CopyButton from './CopyButton.svelte';

	interface Props {
		code?: string;
		showCopy?: boolean;
		className?: string;
		children?: Snippet;
	}

	let { code = '', showCopy = true, className = '', children }: Props = $props();
</script>

<div class="relative w-full">
	<pre
		class="text-text resize-none block w-full pr-14 px-3 py-3 border border-surface1 rounded-md bg-base/40 text-sm font-mono whitespace-pre-wrap wrap-break-word {className}">{#if children}{@render children()}{:else}{code}{/if}</pre>
	{#if showCopy && code}
		<CopyButton text={code} className="absolute top-2 right-2" />
	{/if}
</div>
