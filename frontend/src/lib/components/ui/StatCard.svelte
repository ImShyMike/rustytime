<script lang="ts">
	import Container from './Container.svelte';

	export let title: string = '';
	export let value: string | number | null = null;
	export let subvalue: string | number | null = null;
	export let className = '';
	export let titleClass = 'text-xs font-semibold uppercase tracking-wide text-overlay0';
	export let valueClass = 'text-3xl font-bold text-text mt-auto';
	export let subvalueClass = 'text-xs font-semibold uppercase tracking-wide text-overlay0';
</script>

<Container className={`flex flex-col ${className}`.trim()}>
	<slot name="title">
		{#if title}
			<p class={titleClass}>{title}</p>
		{/if}
	</slot>

	<slot>
		{#if value !== null && value !== undefined}
			<p class={valueClass}>{value}</p>
		{/if}
	</slot>

	<slot name="subvalue">
		{#if subvalue !== null && subvalue !== undefined}
			<p class={subvalueClass}>{subvalue}</p>
		{/if}
	</slot>
</Container>
