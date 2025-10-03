<script lang="ts">
	import Container from './Container.svelte';

	export let title: string = '';
	export let value: string | number | null = null;
	export let className = '';
	export let titleClass = 'text-lg font-semibold text-ctp-text';
	export let valueClass = 'text-3xl font-bold text-ctp-text';
</script>

<Container className={`flex flex-col gap-2 ${className}`.trim()}>
	<slot name="title">
		{#if title}
			<h3 class={titleClass}>{title}</h3>
		{/if}
	</slot>

	<slot>
		{#if value !== null && value !== undefined}
			<p class={valueClass}>{value}</p>
		{/if}
	</slot>
</Container>
