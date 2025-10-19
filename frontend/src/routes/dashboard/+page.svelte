<script lang="ts">
	import { createPieChartOptions, createBarChartOptions } from '$lib/utils/charts';
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { apexcharts } from '$lib/stores/apexcharts';
	import type ApexCharts from 'apexcharts';
	import type { ApexOptions } from 'apexcharts';
	import type { PageData } from './$types';
	import { Container, PageHeading, SectionTitle, StatCard, UserTag } from '$lib';
	import { formatRelativeTime } from '$lib/utils/time';
	import RelativeTime from '$lib/components/ui/RelativeTime.svelte';

	type ApexChartsConstructor = new (element: Element | string, options: ApexOptions) => ApexCharts;

	const isApexChartsConstructor = (value: unknown): value is ApexChartsConstructor =>
		typeof value === 'function';
	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	// Get dashboard data from server-side load
	const dashboardData = data.dashboardData;

	let lastUpdatedAt = $state(new Date());

	let projectsChart: ApexCharts | null = null;
	let languagesChart: ApexCharts | null = null;
	let editorsChart: ApexCharts | null = null;
	let osChart: ApexCharts | null = null;

	let config: string = $state('');
	let copied: boolean = $state(false);

	// Initialize charts when data is available
	$effect(() => {
		if (dashboardData && browser && $apexcharts) {
			initializeCharts();
		}
	});

	$effect(() => {
		if (dashboardData && dashboardData.api_key) {
			config = `[settings]
api_url = "https://api-rustytime.shymike.dev/api/v1"
api_key = ${dashboardData.api_key}`;
		} else {
			config = '';
		}
	});

	async function initializeCharts() {
		if (!dashboardData) return;

		try {
			const apexchartsValue = $apexcharts;
			if (!isApexChartsConstructor(apexchartsValue)) {
				return;
			}

			const ApexCharts = apexchartsValue;

			const theme: 'light' | 'dark' = (() => {
				try {
					const saved = localStorage.getItem('theme');
					if (saved === 'dark' || saved === 'light') return saved;
				} catch (error) {
					console.warn('Unable to read theme preference from localStorage', error);
				}

				const prefersDark =
					window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
				if (prefersDark) return 'dark';

				return document?.documentElement?.classList?.contains('mocha') ? 'dark' : 'light';
			})();

			// Projects chart
			if (dashboardData.projects.length > 0) {
				const projectsElement = document.getElementById('projects-chart');
				if (projectsElement) {
					if (projectsChart) {
						projectsChart.destroy();
					}
					const options = createBarChartOptions(
						dashboardData.projects.slice(0, 8),
						[],
						true,
						theme
					);
					projectsChart = new ApexCharts(projectsElement, options);
					projectsChart.render();
				}
			}

			// Languages chart
			if (dashboardData.languages.length > 0) {
				const languagesElement = document.getElementById('languages-chart');
				if (languagesElement) {
					if (languagesChart) {
						languagesChart.destroy();
					}
					const options = createPieChartOptions(dashboardData.languages.slice(0, 8), [], theme);
					languagesChart = new ApexCharts(languagesElement, options);
					languagesChart.render();
				}
			}

			// Editors chart
			if (dashboardData.editors.length > 0) {
				const editorsElement = document.getElementById('editors-chart');
				if (editorsElement) {
					if (editorsChart) {
						editorsChart.destroy();
					}
					const options = createPieChartOptions(dashboardData.editors.slice(0, 8), [], theme);
					editorsChart = new ApexCharts(editorsElement, options);
					editorsChart.render();
				}
			}

			// Operating Systems chart
			if (dashboardData.operating_systems.length > 0) {
				const osElement = document.getElementById('os-chart');
				if (osElement) {
					if (osChart) {
						osChart.destroy();
					}
					const options = createPieChartOptions(
						dashboardData.operating_systems.slice(0, 8),
						[],
						theme
					);
					osChart = new ApexCharts(osElement, options);
					osChart.render();
				}
			}
		} catch (error) {
			console.error('Failed to initialize ApexCharts:', error);
		}
	}

	function copySetup() {
		if (!config) return;
		navigator.clipboard.writeText(config).then(() => {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		});
	}

	onMount(() => {
		return () => {
			if (projectsChart) projectsChart.destroy();
			if (languagesChart) languagesChart.destroy();
			if (editorsChart) editorsChart.destroy();
			if (osChart) osChart.destroy();
		};
	});
</script>

<svelte:head>
	<title>Dashboard - rustytime</title>
</svelte:head>

{#if dashboardData}
	<div class="bg-ctp-mantle">
		<div class="max-w-6xl mx-auto py-4 md:py-12 px-3">
			<PageHeading title="Dashboard" />

			<!-- User Information -->
			<Container className="pb-1 mb-4">
				<div class="flex items-center gap-4 mb-4">
					{#if dashboardData.avatar_url}
						<img
							src={dashboardData.avatar_url}
							alt="Avatar"
							width="80"
							height="80"
							class="rounded-full border-2 border-ctp-green-500"
						/>
					{/if}
					<div class="flex flex-col">
						<div class="flex items-center gap-2">
							<UserTag admin_level={dashboardData.admin_level} />
							<p class="font-bold text-lg text-ctp-text">{dashboardData.username}</p>
						</div>
						<p class="text-ctp-subtext1">User ID: {dashboardData.user_id}</p>
						<p class="text-ctp-subtext0">
							Joined {formatRelativeTime(new Date(dashboardData.created_at))}
						</p>
					</div>
				</div>
			</Container>

			<!-- Top Stats -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<StatCard
					title="Total Time"
					value={dashboardData.human_readable_total}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Project"
					value={dashboardData.projects.at(0)?.name || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Top Language"
					value={dashboardData.languages.at(0)?.name || 'None'}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
				<StatCard
					title="Total Heartbeats"
					value={dashboardData.total_heartbeats.toLocaleString()}
					valueClass="text-xl font-semibold text-ctp-text"
				/>
			</div>

			<!-- Dashboard Statistics -->
			{#if dashboardData.projects.length || dashboardData.languages.length || dashboardData.editors.length || dashboardData.operating_systems.length}
				<Container className="mb-4">
					<SectionTitle level="h2" className="mb-3 text-lavender">Dashboard Statistics</SectionTitle
					>
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
						<!-- Top Projects (Horizontal Bar Chart) -->
						<div>
							<SectionTitle>Top Projects</SectionTitle>
							{#if dashboardData.projects.length > 0}
								<div id="projects-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No project data available</p>
							{/if}
						</div>

						<!-- Top Languages (Pie Chart) -->
						<div>
							<SectionTitle size="sm" className="mb-4">Top Languages</SectionTitle>
							{#if dashboardData.languages.length > 0}
								<div id="languages-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No language data available</p>
							{/if}
						</div>

						<!-- Top Editors (Pie Chart) -->
						<div>
							<SectionTitle size="sm" className="mb-4">Top Editors</SectionTitle>
							{#if dashboardData.editors.length > 0}
								<div id="editors-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No editor data available</p>
							{/if}
						</div>

						<!-- Top Operating Systems (Pie Chart) -->
						<div>
							<SectionTitle size="sm" className="mb-4">Top Operating Systems</SectionTitle>
							{#if dashboardData.operating_systems.length > 0}
								<div id="os-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No operating system data available</p>
							{/if}
						</div>
					</div>
				</Container>
			{:else}
				<Container
					className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center mb-4"
				>
					<p class="text-lg font-semibold text-ctp-text">No data to display :(</p>
					<p class="text-ctp-subtext0">Complete the setup bellow to start tracking your time!</p>
				</Container>
			{/if}

			<!-- Setup stuff -->
			<Container>
				<SectionTitle level="h2" className="mb-3">Setup</SectionTitle>
				<div class="space-y-4">
					<div>
						<label for="api-setup" class="block text-sm font-medium text-ctp-text mb-2"
							>Copy this into your <code class="bg-ctp-surface1 p-1">~/.wakatime.cfg</code> file:</label
						>
						<div class="relative w-full">
							<textarea
								id="api-setup"
								readonly
								rows="3"
								class="resize-none text-text block w-full pr-14 px-2 py-2 border border-ctp-surface1 rounded-md bg-ctp-surface0/70 text-sm font-mono"
								>{config}</textarea
							>
							<button
								onclick={() => copySetup()}
								aria-label="Copy setup to clipboard"
								class={`absolute top-2 right-2 cursor-pointer h-8 px-2 text-ctp-base text-sm rounded transition-transform duration-200 transform flex items-center gap-2 hover:scale-105 active:scale-100 ` +
									(copied
										? 'bg-ctp-green-600 hover:bg-ctp-green-700'
										: 'bg-ctp-blue/70 hover:bg-ctp-blue')}
							>
								{#if copied}
									<LucideCopyCheck class="w-4 h-4 inline" />
								{:else}
									<LucideCopy class="w-4 h-4 inline" />
								{/if}
							</button>
						</div>
					</div>
				</div>
			</Container>
		</div>
	</div>
	{#if lastUpdatedAt}
		<div
			class="text-center text-ctp-subtext0/85 hover:text-ctp-subtext1 text-sm mb-4"
			title={lastUpdatedAt.toLocaleString()}
		>
			Last updated <RelativeTime datetime={lastUpdatedAt} />
		</div>
	{/if}
{/if}
