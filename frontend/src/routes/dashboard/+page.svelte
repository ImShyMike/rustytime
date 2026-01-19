<script lang="ts">
	import { tick } from 'svelte';
	import { browser } from '$app/environment';
	import { invalidate, goto } from '$app/navigation';
	import { apexcharts } from '$lib/stores/apexcharts';
	import { theme } from '$lib/stores/theme';
	import type { Theme } from '$lib/stores/theme';
	import type ApexCharts from 'apexcharts';
	import type { PageData } from './$types';
	import {
		destroyChart,
		ensureBarChart,
		ensurePieChart,
		isApexChartsConstructor
	} from '$lib/charts/apexClient';
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
	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let dashboardData = $derived(data);
	let lastUpdatedAt = $state(new Date());
	let selectedRange = $derived(data?.range || 'month');

	const rangeOptions = [
		{ value: 'day', label: 'Last 24 Hours' },
		{ value: 'week', label: 'Last 7 Days' },
		{ value: 'month', label: 'Last 30 Days' },
		{ value: 'all', label: 'All Time' }
	];

	let projectsChart: ApexCharts | null = null;
	let languagesChart: ApexCharts | null = null;
	let editorsChart: ApexCharts | null = null;
	let osChart: ApexCharts | null = null;

	const refreshDashboardData = async () => {
		await invalidate('app:dashboard');
	};

	const handleRangeChange = async (newRange: string) => {
		selectedRange = newRange;
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		await goto(`/dashboard?range=${newRange}`, { keepFocus: true });
	};

	function destroyCharts() {
		projectsChart = destroyChart(projectsChart);
		languagesChart = destroyChart(languagesChart);
		editorsChart = destroyChart(editorsChart);
		osChart = destroyChart(osChart);
	}

	setupVisibilityRefresh({
		refresh: refreshDashboardData,
		onError: (error) => {
			console.error('Failed to refresh dashboard data:', error);
		},
		onDispose: destroyCharts
	});

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	// Initialize charts when data is available
	$effect(() => {
		if (!browser) {
			return;
		}

		const activeTheme = $theme as Theme;

		if (!dashboardData) {
			destroyCharts();
			return;
		}

		if ($apexcharts) {
			void initializeCharts(activeTheme);
		}
	});

	async function initializeCharts(theme: Theme) {
		if (!browser) {
			return;
		}

		if (!dashboardData) {
			destroyCharts();
			return;
		}

		await tick();

		try {
			const apexchartsValue = $apexcharts;
			if (!isApexChartsConstructor(apexchartsValue)) {
				return;
			}

			const ApexCharts = apexchartsValue;

			const topProjects = safeGraphData(dashboardData.projects.slice(0, 8));
			const topLanguages = safeGraphData(dashboardData.languages.slice(0, 8));
			const topEditors = safeGraphData(dashboardData.editors.slice(0, 8));
			const topOperatingSystems = safeGraphData(dashboardData.operating_systems.slice(0, 8));

			projectsChart = await ensureBarChart({
				ChartCtor: ApexCharts,
				chart: projectsChart,
				elementId: 'projects-chart',
				data: topProjects,
				theme,
				horizontal: true
			});
			languagesChart = await ensurePieChart({
				ChartCtor: ApexCharts,
				chart: languagesChart,
				elementId: 'languages-chart',
				data: topLanguages,
				theme
			});
			editorsChart = await ensurePieChart({
				ChartCtor: ApexCharts,
				chart: editorsChart,
				elementId: 'editors-chart',
				data: topEditors,
				theme
			});
			osChart = await ensurePieChart({
				ChartCtor: ApexCharts,
				chart: osChart,
				elementId: 'os-chart',
				data: topOperatingSystems,
				theme
			});
		} catch (error) {
			console.error('Failed to initialize ApexCharts:', error);
		}
	}
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
		{#if dashboardData.projects.length || dashboardData.languages.length || dashboardData.editors.length || dashboardData.operating_systems.length}
			<Container className="mb-4">
				<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
					<!-- Top Projects (Horizontal Bar Chart) -->
					<div>
						<SectionTitle>Top Projects</SectionTitle>
						{#if dashboardData.projects.length > 0}
							<div id="projects-chart" class="h-87.5"></div>
						{:else}
							<p class="text-ctp-subtext0">No project data available</p>
						{/if}
					</div>

					<!-- Top Languages (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Languages</SectionTitle>
						{#if dashboardData.languages.length > 0}
							<div id="languages-chart" class="h-87.5"></div>
						{:else}
							<p class="text-ctp-subtext0">No language data available</p>
						{/if}
					</div>

					<!-- Top Editors (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Editors</SectionTitle>
						{#if dashboardData.editors.length > 0}
							<div id="editors-chart" class="h-87.5"></div>
						{:else}
							<p class="text-ctp-subtext0">No editor data available</p>
						{/if}
					</div>

					<!-- Top Operating Systems (Pie Chart) -->
					<div>
						<SectionTitle size="sm" className="mb-4">Top Operating Systems</SectionTitle>
						{#if dashboardData.operating_systems.length > 0}
							<div id="os-chart" class="h-87.5"></div>
						{:else}
							<p class="text-ctp-subtext0">No operating system data available</p>
						{/if}
					</div>
				</div>
			</Container>
		{:else}
			<EmptyState
				title="No data to display :("
				description="Complete the setup bellow to start tracking your time!"
				className="mb-4"
			/>
		{/if}
	</PageScaffold>
{/if}
