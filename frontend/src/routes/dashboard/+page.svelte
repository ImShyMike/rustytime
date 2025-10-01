<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { createDataLoader } from '$lib/utils/dataLoader';
	import { handleAuthEffect } from '$lib/utils/authEffect';
	import type { DashboardResponse } from '$lib/types/dashboard';
	import { createPieChartOptions, createBarChartOptions } from '$lib/utils/charts';
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { apexcharts } from '$lib/stores/apexcharts';

	const {
		data: dashboardData,
		loading,
		error,
		loadData
	} = createDataLoader<DashboardResponse>('/page/dashboard');

	$effect(() => {
		handleAuthEffect({
			isAuthLoading: $auth.isLoading,
			isAuthenticated: $auth.isAuthenticated,
			user: $auth.user,
			data: $dashboardData,
			loading: $loading,
			error: $error,
			loadData,
			requireAdmin: false,
			redirectTo: '/'
		});
	});

	let projectsChart: any = null;
	let languagesChart: any = null;
	let editorsChart: any = null;
	let osChart: any = null;

	let config: string = $state('');
	let copied: boolean = $state(false);

	// Initialize charts when data is available
	$effect(() => {
		if ($dashboardData && browser && $apexcharts) {
			initializeCharts();
		}
	});

	$effect(() => {
		if ($dashboardData && $dashboardData.api_key) {
			config = `[settings]
api_url = https://api-rustytime.shymike.dev/api/v1
api_key = ${$dashboardData.api_key}`;
		} else {
			config = '';
		}
	});

	async function initializeCharts() {
		if (!$dashboardData) return;

		try {
			const ApexCharts = $apexcharts as any;

			const theme: 'light' | 'dark' = (() => {
				try {
					const saved = localStorage.getItem('theme');
					if (saved === 'dark' || saved === 'light') return saved;
				} catch (e) {}

				const prefersDark =
					window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
				if (prefersDark) return 'dark';

				return document?.documentElement?.classList?.contains('mocha') ? 'dark' : 'light';
			})();

			// Projects chart
			if ($dashboardData.projects.length > 0) {
				const projectsElement = document.getElementById('projects-chart');
				if (projectsElement) {
					if (projectsChart) {
						projectsChart.destroy();
					}
					const options = createBarChartOptions(
						$dashboardData.projects.slice(0, 8),
						[],
						true,
						theme
					);
					projectsChart = new ApexCharts(projectsElement, options);
					projectsChart.render();
				}
			}

			// Languages chart
			if ($dashboardData.languages.length > 0) {
				const languagesElement = document.getElementById('languages-chart');
				if (languagesElement) {
					if (languagesChart) {
						languagesChart.destroy();
					}
					const options = createPieChartOptions($dashboardData.languages.slice(0, 8), [], theme);
					languagesChart = new ApexCharts(languagesElement, options);
					languagesChart.render();
				}
			}

			// Editors chart
			if ($dashboardData.editors.length > 0) {
				const editorsElement = document.getElementById('editors-chart');
				if (editorsElement) {
					if (editorsChart) {
						editorsChart.destroy();
					}
					const options = createPieChartOptions($dashboardData.editors.slice(0, 8), [], theme);
					editorsChart = new ApexCharts(editorsElement, options);
					editorsChart.render();
				}
			}

			// Operating Systems chart
			if ($dashboardData.operating_systems.length > 0) {
				const osElement = document.getElementById('os-chart');
				if (osElement) {
					if (osChart) {
						osChart.destroy();
					}
					const options = createPieChartOptions(
						$dashboardData.operating_systems.slice(0, 8),
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

{#if $loading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-ctp-subtext0">Loading dashboard...</span>
	</div>
{:else if $auth.isLoading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-ctp-subtext0">Authenticating...</span>
	</div>
{:else if $error}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-ctp-red-600 mb-4">Error</h1>
			<p class="text-ctp-subtext1 mb-4">{$error}</p>
			<button
				onclick={() => window.location.reload()}
				class="cursor-pointer bg-ctp-blue-600 hover:bg-ctp-blue-700 text-ctp-base px-4 py-2 rounded"
				>Retry</button
			>
		</div>
	</div>
{:else if $auth.isAuthenticated && $auth.user && $dashboardData}
	<div class="min-h-screen bg-ctp-mantle">
		<div class="max-w-4xl mx-auto py-6 md:py-12 px-3">
			<h1 class="text-3xl font-bold text-ctp-mauve mb-6 flex items-center gap-2">Dashboard</h1>

			<!-- User Information -->
			<div class="bg-ctp-base rounded-xl shadow p-4 pb-1 mb-4">
				<div class="flex items-center gap-4 mb-4">
					{#if $dashboardData.avatar_url}
						<img
							src={$dashboardData.avatar_url}
							alt="Avatar"
							width="80"
							height="80"
							class="rounded-full border-2 border-ctp-green-500"
						/>
					{/if}
					<div class="flex flex-col">
						<div class="flex items-center gap-2">
							<span
								class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {$auth.user
									.is_admin
									? 'bg-ctp-red-400 text-ctp-crust'
									: 'bg-ctp-overlay2 text-ctp-crust'} items-center h-6"
							>
								{$auth.user.is_admin ? 'Admin' : 'User'}
							</span>
							<p class="font-bold text-lg text-ctp-text">{$dashboardData.username}</p>
						</div>
						<p class="text-ctp-subtext1">GitHub ID: {$dashboardData.github_id}</p>
						<p class="text-ctp-subtext1">Member since: {$dashboardData.created_at}</p>
					</div>
				</div>
			</div>

			<!-- Top Stats -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<div class="bg-ctp-base rounded-xl shadow p-4">
					<p class="text-sm text-ctp-text">Total Time</p>
					<h3 class="font-semibold text-ctp-text">{$dashboardData.human_readable_total}</h3>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-4">
					<p class="text-sm text-ctp-text">Top Project</p>
					<h3 class="font-semibold text-ctp-text">
						{$dashboardData.projects.at(0)?.name || 'None'}
					</h3>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-4">
					<p class="text-sm text-ctp-text">Top Language</p>
					<h3 class="font-semibold text-ctp-text">
						{$dashboardData.languages.at(0)?.name || 'None'}
					</h3>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-4">
					<p class="text-sm text-ctp-text">Total Heartbeats</p>
					<h3 class="font-semibold text-ctp-text">
						{$dashboardData.total_heartbeats.toLocaleString()}
					</h3>
				</div>
			</div>

			<!-- Dashboard Statistics -->
			<div class="bg-ctp-base rounded-xl shadow p-6 mb-4">
				<h2 class="text-xl font-semibold text-ctp-text mb-3">Dashboard Statistics</h2>
				{#if $dashboardData.projects.length || $dashboardData.languages.length || $dashboardData.editors.length || $dashboardData.operating_systems.length}
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
						<!-- Top Projects (Horizontal Bar Chart) -->
						<div>
							<h3 class="text-lg font-medium text-ctp-text mb-4">Top Projects</h3>
							{#if $dashboardData.projects.length > 0}
								<div id="projects-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No project data available</p>
							{/if}
						</div>

						<!-- Top Languages (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-ctp-text mb-4">Top Languages</h3>
							{#if $dashboardData.languages.length > 0}
								<div id="languages-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No language data available</p>
							{/if}
						</div>

						<!-- Top Editors (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-ctp-text mb-4">Top Editors</h3>
							{#if $dashboardData.editors.length > 0}
								<div id="editors-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No editor data available</p>
							{/if}
						</div>

						<!-- Top Operating Systems (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-ctp-text mb-4">Top Operating Systems</h3>
							{#if $dashboardData.operating_systems.length > 0}
								<div id="os-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-ctp-subtext0">No operating system data available</p>
							{/if}
						</div>
					</div>
				{:else}
					<p class="text-md text-ctp-subtext0">No data to display :(</p>
					<p class="text-ctp-subtext0">Complete the setup bellow to start tracking your time!</p>
				{/if}
			</div>

			<!-- Setup stuff -->
			<div class="bg-ctp-base rounded-xl shadow p-6 mb-4">
				<h2 class="text-xl font-semibold text-ctp-text mb-3">Setup</h2>
				<div class="space-y-4">
					<div>
						<label for="api-setup" class="block text-sm font-medium text-ctp-text mb-2"
							>Copy this into your <code class="bg-ctp-surface1 p-1">~/.wakatime.cfg</code> file:</label
						>
						<div class="relative w-full">
							<textarea
								id="api-setup"
								readonly
								value={config}
								rows="3"
								class="resize-none text-text block w-full pr-14 px-2 py-2 border border-ctp-surface1 rounded-md bg-ctp-surface0/70 text-sm font-mono"
							></textarea>
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
			</div>
		</div>
	</div>
{/if}
