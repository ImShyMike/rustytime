<script lang="ts">
	import { Container, PageScaffold } from '$lib';
	import Skeleton from '$lib/components/ui/Skeleton.svelte';
</script>

<PageScaffold title="Projects" lastUpdatedAt={null} showLastUpdated={false}>
	<!-- Stat Cards Skeleton -->
	<div class="grid grid-cols-1 gap-4 mb-4 sm:grid-cols-2">
		{#each Array(2) as _}
			<Container className="flex flex-col">
				<Skeleton className="w-28 h-3 mb-2" />
				<Skeleton className="w-16 h-8 mb-1" />
				<Skeleton className="w-36 h-3" />
			</Container>
		{/each}
	</div>

	<!-- Filter Bar Skeleton -->
	<Container className="flex flex-col mb-4 gap-4 h-23">
		<div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
			<div class="flex flex-col gap-1 flex-1">
				<Skeleton className="w-12 h-3" />
				<Skeleton className="w-full h-10 rounded-lg" />
			</div>
			<div class="flex flex-col sm:flex-row items-stretch sm:items-end gap-4">
				<div class="flex flex-col gap-1 flex-1">
					<Skeleton className="w-16 h-3" />
					<Skeleton className="w-46 h-10 rounded-lg" />
				</div>
				<div class="flex-1">
					<Skeleton className="w-14 h-3 mb-1" />
					<Skeleton className="w-40 h-10 rounded-lg" />
				</div>
			</div>
		</div>
	</Container>

	<!-- Project Cards Skeleton -->
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
		{#each Array(12) as _}
			<Container className="flex flex-col gap-2">
				<Skeleton className="w-32 h-5" />
				<Skeleton className="w-20 h-7" />
				<Skeleton className="w-40 h-4" />
				<div class="flex flex-col gap-1">
					<Skeleton className="w-28 h-4" />
					<Skeleton className="w-36 h-3" />
				</div>
			</Container>
		{/each}
	</div>
</PageScaffold>
