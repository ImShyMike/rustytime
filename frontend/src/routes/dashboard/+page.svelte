<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDataLoader } from '$lib/utils/dataLoader';
	import { handleAuthEffect } from '$lib/utils/authEffect';
	import type { DashboardResponse } from '$lib/types/dashboard';
	import { createPieChartOptions, createHorizontalBarChartOptions } from '$lib/utils/chart';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	const {
		data: dashboardData,
		loading,
		error,
		loadData
	} = createDataLoader<DashboardResponse>('/dashboard');

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

	// Initialize charts when data is available
	$effect(() => {
		if ($dashboardData && browser) {
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
			const ApexChartsModule = await import('apexcharts');
			const ApexCharts = ApexChartsModule.default;

			// Projects chart
			if ($dashboardData.projects.length > 0) {
				const projectsElement = document.getElementById('projects-chart');
				if (projectsElement) {
					if (projectsChart) {
						projectsChart.destroy();
					}
					const options = createHorizontalBarChartOptions($dashboardData.projects.slice(0, 8), []);
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
					const options = createPieChartOptions($dashboardData.languages.slice(0, 8), []);
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
					const options = createPieChartOptions($dashboardData.editors.slice(0, 8), []);
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
					const options = createPieChartOptions($dashboardData.operating_systems.slice(0, 8), []);
					osChart = new ApexCharts(osElement, options);
					osChart.render();
				}
			}
		} catch (error) {
			console.error('Failed to load ApexCharts:', error);
		}
	}

	onMount(() => {
		return () => {
			if (projectsChart) projectsChart.destroy();
			if (languagesChart) languagesChart.destroy();
			if (editorsChart) editorsChart.destroy();
			if (osChart) osChart.destroy();
		};
	});

	const handleLogout = () => {
		auth.logout();
		goto(resolve('/'));
	};
</script>

{#if $loading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-gray-500">Loading dashboard...</span>
	</div>
{:else if $auth.isLoading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-gray-500">Authenticating...</span>
	</div>
{:else if $error}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-red-600 mb-4">Error</h1>
			<p class="text-gray-600 mb-4">{$error}</p>
			<button
				onclick={() => window.location.reload()}
				class="cursor-pointer bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
				>Retry</button
			>
		</div>
	</div>
{:else if $auth.isAuthenticated && $auth.user && $dashboardData}
	<div class="min-h-screen bg-gray-50">
		<div class="max-w-4xl mx-auto py-12">
			<h1 class="text-3xl font-bold text-orange-600 mb-6 flex items-center gap-2">
				rustytime dashboard
			</h1>

			<!-- User Information -->
			<div class="bg-white rounded-xl shadow p-4 pb-1 mb-4">
				<div class="flex items-center gap-4 mb-4">
					{#if $dashboardData.avatar_url}
						<img
							src={$dashboardData.avatar_url}
							alt="Avatar"
							width="80"
							height="80"
							class="rounded-full border-2 border-green-500"
						/>
					{/if}
					<div class="flex flex-col">
						<div class="flex items-center gap-2">
							{#if $dashboardData.is_admin}
								<span
									class="inline-flex items-center px-2.5 py-0.75 rounded-full text-xs font-medium bg-red-100 text-red-800"
								>
									Admin
								</span>
							{/if}
							<p class="font-bold text-lg text-gray-900">{$dashboardData.username}</p>
						</div>
						<p class="text-gray-600">GitHub ID: {$dashboardData.github_id}</p>
						<p class="text-gray-600">Member since: {$dashboardData.created_at}</p>
					</div>
				</div>
			</div>

			<!-- Top Stats -->
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-4">
				<div class="bg-white rounded-xl shadow p-4">
					<p class="text-sm text-gray-700">Total Time</p>
					<h3 class="font-semibold text-gray-800">{$dashboardData.human_readable_total}</h3>
				</div>
				<div class="bg-white rounded-xl shadow p-4">
					<p class="text-sm text-gray-700">Top Project</p>
					<h3 class="font-semibold text-gray-800">
						{$dashboardData.projects.at(0)?.name || 'None'}
					</h3>
				</div>
				<div class="bg-white rounded-xl shadow p-4">
					<p class="text-sm text-gray-700">Top Language</p>
					<h3 class="font-semibold text-gray-800">
						{$dashboardData.languages.at(0)?.name || 'None'}
					</h3>
				</div>
				<div class="bg-white rounded-xl shadow p-4">
					<p class="text-sm text-gray-700">Total Heartbeats</p>
					<h3 class="font-semibold text-gray-800">
						{$dashboardData.total_heartbeats.toLocaleString()}
					</h3>
				</div>
			</div>

			<!-- Dashboard Statistics -->
			<div class="bg-white rounded-xl shadow p-8 mb-4">
				<h2 class="text-xl font-semibold text-gray-800 mb-6">Dashboard Statistics</h2>
				{#if $dashboardData.projects.length || $dashboardData.languages.length || $dashboardData.editors.length || $dashboardData.operating_systems.length}
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
						<!-- Top Projects (Horizontal Bar Chart) -->
						<div>
							<h3 class="text-lg font-medium text-gray-700 mb-4">Top Projects</h3>
							{#if $dashboardData.projects.length > 0}
								<div id="projects-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-gray-500">No project data available</p>
							{/if}
						</div>

						<!-- Top Languages (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-gray-700 mb-4">Top Languages</h3>
							{#if $dashboardData.languages.length > 0}
								<div id="languages-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-gray-500">No language data available</p>
							{/if}
						</div>

						<!-- Top Editors (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-gray-700 mb-4">Top Editors</h3>
							{#if $dashboardData.editors.length > 0}
								<div id="editors-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-gray-500">No editor data available</p>
							{/if}
						</div>

						<!-- Top Operating Systems (Pie Chart) -->
						<div>
							<h3 class="text-lg font-medium text-gray-700 mb-4">Top Operating Systems</h3>
							{#if $dashboardData.operating_systems.length > 0}
								<div id="os-chart" class="h-[350px]"></div>
							{:else}
								<p class="text-gray-500">No operating system data available</p>
							{/if}
						</div>
					</div>
				{:else}
					<p class="text-md text-gray-500">No data to display :(</p>
					<p class="text-gray-500">Complete the setup bellow to start tracking your time!</p>
				{/if}
			</div>

			<!-- Setup stuff -->
			<div class="bg-white rounded-xl shadow p-6 mb-4">
				<h2 class="text-xl font-semibold text-gray-800 mb-2">Setup</h2>
				<div class="space-y-4">
					<div>
						<label for="api-setup" class="block text-sm font-medium text-gray-700 mb-2"
							>Copy this into your <code class="bg-gray-200 p-1">~/.wakatime.cfg</code> file:</label
						>
						<div class="flex flex-col items-start gap-2">
							<textarea
								id="api-setup"
								readonly
								value={config}
								rows="3"
								class="resize-none block w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-sm font-mono"
							></textarea>
							<button
								onclick={() => navigator.clipboard.writeText(config)}
								class="cursor-pointer px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
							>
								Copy
							</button>
						</div>
					</div>
				</div>
			</div>

			<div class="flex justify-between mt-4 px-2">
				<a href={resolve('/')} class="bg-gray-600 hover:bg-gray-700 text-white py-2 px-6 rounded"
					>Back</a
				>
				<div class="space-x-4">
					{#if $dashboardData.is_admin}
						<a
							href={resolve('/admin')}
							class="bg-red-600 hover:bg-red-700 text-white py-2 px-6 rounded">Admin</a
						>
					{/if}
					<button
						onclick={handleLogout}
						class="cursor-pointer bg-red-600 hover:bg-red-700 text-white py-2 px-6 rounded"
						>Logout</button
					>
				</div>
			</div>
		</div>
	</div>
{:else}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-red-600 mb-4">Access Denied</h1>
			<p class="text-gray-600 mb-4">You need to be logged in to access the dashboard.</p>
			<a href={resolve('/')} class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
				>Go Home</a
			>
		</div>
	</div>
{/if}
