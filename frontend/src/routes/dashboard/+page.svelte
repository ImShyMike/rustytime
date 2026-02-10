<script lang="ts">
	import { browser } from '$app/environment';
	import { invalidate, goto } from '$app/navigation';
	import { theme } from '$lib/stores/theme';
	import type { Theme } from '$lib/stores/theme';
	import type { PageData } from './$types';
	import type { DashboardResponse } from '$lib/types/dashboard';
	import { createDeferredData } from '$lib/utils/deferred-data.svelte';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { Container, PageScaffold, SectionTitle, StatCard, ToggleGroup, EmptyState } from '$lib';
	import { safeGraphData, safeText } from '$lib/utils/text';
	import BarChart from '$lib/charts/BarChart.svelte';
	import PieChart from '$lib/charts/PieChart.svelte';
	import DashboardSkeleton from './DashboardSkeleton.svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let lastUpdatedAt = $state(new Date());
	let selectedRange = $derived(data?.range || 'month');
	let loadedRange = $state('today');

	const deferred = createDeferredData(() => data.dashboard);

	$effect(() => {
		if (deferred.data) {
			lastUpdatedAt = new Date();
		}
	});

	const refreshDashboardData = async () => {
		await invalidate('app:dashboard');
	};

	const rangeOptions = [
		{ value: 'day', label: 'Today' },
		{ value: 'week', label: 'This Week' },
		{ value: 'month', label: 'This Month' },
		{ value: 'all', label: 'All Time' }
	];

	const handleRangeChange = async (newRange: string) => {
		selectedRange = newRange;
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		await goto(`/dashboard?range=${newRange}`, { keepFocus: true });
		loadedRange =
			rangeOptions.find((opt) => opt.value === newRange)?.label.toLowerCase() || 'today';
	};

	setupVisibilityRefresh({
		refresh: refreshDashboardData,
		onError: (error) => {
			console.error('Failed to refresh dashboard data:', error);
		}
	});

	let activeTheme = $derived(browser ? ($theme as Theme) : 'dark');

	const getDerivedData = (d: DashboardResponse) => ({
		topProjects: safeGraphData(d?.projects?.slice(0, 8).reverse() ?? []),
		topLanguages: safeGraphData(d?.languages?.slice(0, 8) ?? []),
		topEditors: safeGraphData(d?.editors?.slice(0, 8) ?? []),
		topOperatingSystems: safeGraphData(d?.operating_systems?.slice(0, 8) ?? [])
	});
</script>

{#if deferred.showSkeleton}
	<DashboardSkeleton />
{:else if deferred.data}
	{@const { topProjects, topLanguages, topEditors, topOperatingSystems } = getDerivedData(
		deferred.data
	)}
	<PageScaffold title="Dashboard" {lastUpdatedAt}>
		<!-- Time Range Filter -->
		<Container className="mb-4">
			<ToggleGroup options={rangeOptions} selected={selectedRange} onchange={handleRangeChange} />
		</Container>

		{#if deferred.data.projects.length || deferred.data.languages.length || deferred.data.editors.length || deferred.data.operating_systems.length}
			<!-- Top Stats -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<StatCard
					title="Total Time"
					value={deferred.data?.human_readable_total || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Project"
					value={safeText(deferred.data?.projects?.[0]?.name) || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Language"
					value={safeText(deferred.data?.languages?.[0]?.name) || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Total Heartbeats"
					value={deferred.data?.total_heartbeats
						? deferred.data.total_heartbeats.toLocaleString()
						: '0'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
			</div>

			<!-- Dashboard Statistics -->
			<Container className="mb-4">
				<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
					<!-- Projects (Horizontal Bar Chart) -->
					<div>
						<SectionTitle size="sm">Projects</SectionTitle>
						{#if topProjects.length > 0}
							<BarChart data={topProjects} theme={activeTheme} horizontal class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No project data available</p>
						{/if}
					</div>

					<!-- Languages (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Languages</SectionTitle>
						{#if topLanguages.length > 0}
							<PieChart data={topLanguages} theme={activeTheme} class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No language data available</p>
						{/if}
					</div>

					{#if deferred.data.editors.length > 1 && deferred.data.operating_systems.length > 1}
						<!-- Editors (Pie Chart) -->
						<div>
							<SectionTitle size="sm" className="mb-4">Editors</SectionTitle>
							{#if topEditors.length > 0}
								<PieChart data={topEditors} theme={activeTheme} class="h-87.5" />
							{:else}
								<p class="text-ctp-subtext0">No editor data available</p>
							{/if}
						</div>

						<!-- Operating Systems (Pie Chart) -->
						<div>
							<SectionTitle size="sm" className="mb-4">Operating Systems</SectionTitle>
							{#if topOperatingSystems.length > 0}
								<PieChart data={topOperatingSystems} theme={activeTheme} class="h-87.5" />
							{:else}
								<p class="text-ctp-subtext0">No operating system data available</p>
							{/if}
						</div>
					{/if}
				</div>
			</Container>
		{:else if loadedRange === 'all'}
			<EmptyState
				title="No data to display :("
				description="Complete the setup in the settings page and start coding to see your stats here!"
				className="mb-4"
			/>
		{:else}
			<EmptyState
				title={`No time tracked ${loadedRange}`}
				description="Get coding or change the time range to see your stats."
				className="mb-4"
			/>
		{/if}
	</PageScaffold>
{:else if deferred.loadError}
	<PageScaffold title="Dashboard" showLastUpdated={false}>
		<EmptyState
			title="Failed to load dashboard"
			description="Something went wrong loading your dashboard data. Please try again."
			className="mb-4"
		/>
	</PageScaffold>
{/if}
