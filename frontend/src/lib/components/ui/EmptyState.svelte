<script lang="ts">
	import type { Snippet } from 'svelte';
	import Container from './Container.svelte';

	interface Props {
		title: string;
		description?: string;
		className?: string;
		children?: Snippet;
		icon?: Snippet;
	}

	let { title, description = '', className = '', children, icon }: Props = $props();
</script>

<Container
	className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center {className}"
>
	{#if icon}
		{@render icon()}
	{/if}
	<div class="space-y-1">
		<p class="text-lg font-semibold text-ctp-text">{title}</p>
		{#if description}
			<p class="text-ctp-subtext0">{description}</p>
		{/if}
	</div>
	{#if children}
		<div class="mt-2">
			{@render children()}
		</div>
	{/if}
</Container>
