<script lang="ts">
	import PageHeading from './PageHeading.svelte';
	import Footer from '../Footer.svelte';

	export let title: string;
	export let lastUpdatedAt: Date | null = null;
	export let showLastUpdated = true;
	export let wrapperClass = '';
	export let contentClass = '';

	const hasHeadingSlot = Boolean($$slots.heading);
	const hasFooterSlot = Boolean($$slots.footer);
</script>

<div class={`bg-ctp-mantle h-full flex flex-col ${wrapperClass}`.trim()}>
	<div class="grow">
		<div class={`max-w-6xl mx-auto py-4 2xl:py-12 px-3 ${contentClass}`.trim()}>
			{#if hasHeadingSlot}
				<slot name="heading" />
			{:else}
				<PageHeading {title} />
			{/if}

			<slot />
		</div>
	</div>

	{#if showLastUpdated}
		{#if hasFooterSlot}
			<slot name="footer" {lastUpdatedAt} />
		{:else}
			<Footer {lastUpdatedAt} />
		{/if}
	{/if}
</div>
