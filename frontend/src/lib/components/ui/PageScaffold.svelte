<script lang="ts">
	import PageHeading from './PageHeading.svelte';
	import RelativeTime from './RelativeTime.svelte';

	export let title: string;
	export let lastUpdatedAt: Date | null = null;
	export let showLastUpdated = true;
	export let wrapperClass = '';
	export let contentClass = '';

	type $$Props = {
		title: string;
		lastUpdatedAt?: Date | null;
		showLastUpdated?: boolean;
		wrapperClass?: string;
		contentClass?: string;
	};

	type $$Slots = {
		default: Record<string, never>;
		heading?: Record<string, never>;
		footer?: { lastUpdatedAt: Date | null };
	};

	const hasHeadingSlot = Boolean($$slots.heading);
	const hasFooterSlot = Boolean($$slots.footer);
</script>

<div class={`bg-ctp-mantle min-h-[99vh] flex flex-col ${wrapperClass}`.trim()}>
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
		{:else if lastUpdatedAt}
			<div
				class="text-center text-ctp-subtext0/70 hover:text-ctp-subtext1 text-sm"
				title={lastUpdatedAt.toLocaleString()}
			>
				Last updated <RelativeTime datetime={lastUpdatedAt} />
			</div>
		{/if}
	{/if}
</div>
