<script lang="ts">
	import { browser } from '$app/environment';
	import { invalidate, goto } from '$app/navigation';
	import { theme } from '$lib/stores/theme';
	import type { Theme } from '$lib/stores/theme';
	import type { PageData } from './$types';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import {
		Container,
		PageScaffold,
		SectionTitle,
		StatCard,
		UserTag,
		ToggleGroup,
		EmptyState
	} from '$lib';
	import RelativeTime from '$lib/components/ui/RelativeTime.svelte';
	import { safeGraphData, safeText } from '$lib/utils/text';
	import Avatar from '$lib/components/ui/Avatar.svelte';
	import BarChart from '$lib/charts/BarChart.svelte';
	import PieChart from '$lib/charts/PieChart.svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let dashboardData = $derived(data);
	let lastUpdatedAt = $state(new Date());
	let selectedRange = $derived(data?.range || 'month');
	let loadedRange = $state('today');

	const rangeOptions = [
		{ value: 'day', label: 'Today' },
		{ value: 'week', label: 'This Week' },
		{ value: 'month', label: 'This Month' },
		{ value: 'all', label: 'All Time' }
	];

	const refreshDashboardData = async () => {
		await invalidate('app:dashboard');
	};

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

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	let activeTheme = $derived(browser ? ($theme as Theme) : 'dark');
	let topProjects = $derived(safeGraphData(dashboardData?.projects?.slice(0, 8) ?? []));
	let topLanguages = $derived(safeGraphData(dashboardData?.languages?.slice(0, 8) ?? []));
	let topEditors = $derived(safeGraphData(dashboardData?.editors?.slice(0, 8) ?? []));
	let topOperatingSystems = $derived(
		safeGraphData(dashboardData?.operating_systems?.slice(0, 8) ?? [])
	);
</script>

<svelte:head>
	<title>Dashboard - rustytime</title>
</svelte:head>

{#if dashboardData}
	<PageScaffold title="Dashboard" {lastUpdatedAt}>
		<!-- User Information -->
		<Container className="pb-1 mb-4">
			<div class="flex items-center gap-4 mb-4">
				{#if dashboardData.avatar_url}
					<Avatar url={dashboardData.avatar_url} size={80} />
				{/if}
				<div class="flex flex-col">
					<div class="flex items-center gap-2">
						<UserTag admin_level={dashboardData.admin_level} />
						<p class="font-bold text-lg text-ctp-text">{dashboardData.username}</p>
					</div>
					<p class="text-ctp-subtext1">User ID: {dashboardData.user_id}</p>
					<p class="text-ctp-subtext0" title={new Date(dashboardData.created_at).toLocaleString()}>
						Joined <RelativeTime datetime={new Date(dashboardData.created_at)} />
					</p>
				</div>
			</div>
		</Container>

		<!-- Time Range Filter -->
		<Container className="mb-4">
			<ToggleGroup options={rangeOptions} selected={selectedRange} onchange={handleRangeChange} />
		</Container>

		{#if dashboardData.projects.length || dashboardData.languages.length || dashboardData.editors.length || dashboardData.operating_systems.length}
			<!-- Top Stats -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<StatCard
					title="Total Time"
					value={dashboardData?.human_readable_total || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Project"
					value={safeText(dashboardData?.projects?.[0]?.name) || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Language"
					value={safeText(dashboardData?.languages?.[0]?.name) || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Total Heartbeats"
					value={dashboardData?.total_heartbeats
						? dashboardData.total_heartbeats.toLocaleString()
						: '0'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
			</div>

			<!-- Dashboard Statistics -->
			<Container className="mb-4">
				<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
					<!-- Top Projects (Horizontal Bar Chart) -->
					<div>
						<SectionTitle>Top Projects</SectionTitle>
						{#if topProjects.length > 0}
							<BarChart data={topProjects} theme={activeTheme} horizontal class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No project data available</p>
						{/if}
					</div>

					<!-- Top Languages (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Languages</SectionTitle>
						{#if topLanguages.length > 0}
							<PieChart data={topLanguages} theme={activeTheme} class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No language data available</p>
						{/if}
					</div>

					<!-- Top Editors (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Editors</SectionTitle>
						{#if topEditors.length > 0}
							<PieChart data={topEditors} theme={activeTheme} class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No editor data available</p>
						{/if}
					</div>

					<!-- Top Operating Systems (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Operating Systems</SectionTitle>
						{#if topOperatingSystems.length > 0}
							<PieChart data={topOperatingSystems} theme={activeTheme} class="h-87.5" />
						{:else}
							<p class="text-ctp-subtext0">No operating system data available</p>
						{/if}
					</div>
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
{/if}
