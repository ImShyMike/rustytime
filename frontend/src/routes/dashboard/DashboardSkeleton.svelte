<script lang="ts">
	import { Container, PageScaffold, SectionTitle, ToggleGroup } from '$lib';
	import Skeleton from '$lib/components/ui/Skeleton.svelte';
	import { page } from '$app/state';

	const rangeOptions = [
		{ value: 'day', label: 'Today', disabled: true },
		{ value: 'week', label: 'This Week', disabled: true },
		{ value: 'month', label: 'This Month', disabled: true },
		{ value: 'year', label: 'All Time', disabled: true }
	];

	const range = page.url.searchParams.get('range') || 'day';
</script>

<PageScaffold title="Dashboard" lastUpdatedAt={null} showLastUpdated={false}>
	<!-- Time Range Filter Skeleton -->
	<Container className="mb-4">
		<ToggleGroup options={rangeOptions} selected={range} onchange={() => {}} />
	</Container>

	<!-- Stat Cards Skeleton -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
		{#each Array(4) as _}
			<Container className="flex flex-col">
				<Skeleton className="w-24 h-2 mb-2" />
				<Skeleton className="w-32 h-7" />
			</Container>
		{/each}
	</div>

	<!-- Charts Skeleton -->
	<Container className="mb-4">
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
			{#each Array(4) as _}
				<div>
					<SectionTitle size="sm" className="mb-4">
						<Skeleton className="w-24 h-5 inline-block" />
					</SectionTitle>
					<Skeleton className="h-87.5 w-full" />
				</div>
			{/each}
		</div>
	</Container>
</PageScaffold>
