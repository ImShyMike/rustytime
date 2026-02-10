<script lang="ts">
	import { Container, PageScaffold, SectionTitle } from '$lib';
	import Skeleton from '$lib/components/ui/Skeleton.svelte';
</script>

<PageScaffold title="Profile" lastUpdatedAt={null} showLastUpdated={false}>
	<svelte:fragment slot="heading">
		<div class="flex items-center gap-4 mb-6">
			<Skeleton className="h-16 w-16 rounded-full" />
			<div class="flex flex-col gap-2">
				<div class="flex items-center gap-2">
					<Skeleton className="w-36 h-7" />
					<Skeleton className="w-16 h-6 rounded-full" />
				</div>
				<Skeleton className="w-20 h-4" />
			</div>
		</div>
	</svelte:fragment>

	<!-- Time Stats Skeleton -->
	<div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-4">
		{#each Array(3) as _}
			<Container className="flex flex-col">
				<Skeleton className="w-24 h-2 mb-2" />
				<Skeleton className="w-32 h-7" />
			</Container>
		{/each}
	</div>

	<!-- Projects Section Skeleton -->
	<SectionTitle className="mb-3">
		<Skeleton className="w-48 h-5 inline-block" />
	</SectionTitle>
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
		{#each Array(6) as _}
			<Container className="flex flex-col gap-2">
				<Skeleton className="w-32 h-5" />
				<Skeleton className="w-20 h-7" />
				<Skeleton className="w-40 h-4" />
			</Container>
		{/each}
	</div>
</PageScaffold>
