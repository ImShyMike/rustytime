<script lang="ts">
	import { browser } from '$app/environment';
	import { createDateBarChartOptions } from '$lib/utils/charts';
	import { apexcharts } from '$lib/stores/apexcharts';
	import type { PageData } from './$types';
	import UserTag from '$lib/components/UserTag.svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	// Get admin data from server-side load
	const adminData = data.adminData;

	let activityChart: any = null;

	$effect(() => {
		if (adminData && browser && $apexcharts) {
			initializeCharts();
		}
	});

	async function initializeCharts() {
		if (!adminData) {
			return;
		}

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

			if (adminData.stats.daily_activity.length > 0) {
				const activityElement = document.getElementById('activity-chart');
				if (activityElement && adminData) {
					if (activityChart) {
						activityChart.destroy();
					}
					const options = createDateBarChartOptions(
						adminData.stats.daily_activity,
						[],
						false,
						theme
					);
					activityChart = new ApexCharts(activityElement, options);
					activityChart.render();
				}
			}
		} catch (error) {
			console.error('Failed to initialize ApexCharts:', error);
		}
	}
</script>

{#if adminData}
	<div class="bg-ctp-mantle">
		<div class="max-w-6xl mx-auto py-4 md:py-12 px-3">
			<h1
				class="text-3xl font-bold text-ctp-mauve mb-6 flex items-center gap-2 justify-end text-right md:justify-start md:text-left pr-4 md:pr-0"
			>
				Admin Dashboard
			</h1>

			<!-- System Statistics -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-ctp-text mb-2">Total Users</h3>
					<p class="text-3xl font-bold text-ctp-blue-600">{adminData.stats.total_users}</p>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-ctp-text mb-2">Total Heartbeats</h3>
					<p class="text-3xl font-bold text-ctp-green-600">
						{adminData.stats.total_heartbeats.toLocaleString()}
					</p>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-ctp-text mb-2">Last 24h</h3>
					<p class="text-3xl font-bold text-ctp-mauve-600">
						{adminData.stats.heartbeats_last_24h.toLocaleString()}
					</p>
				</div>
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-ctp-text mb-2">Requests/sec</h3>
					<p class="text-3xl font-bold text-ctp-peach-600">
						{adminData.stats.requests_per_second}
					</p>
				</div>
			</div>

			<!-- Top Lists -->
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
				<!-- Top Languages -->
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-xl font-semibold text-ctp-text mb-4">Top Languages</h3>
					{#if adminData.stats.top_languages.length > 0}
						<div class="space-y-2">
							{#each adminData.stats.top_languages.slice(0, 10) as lang (lang.language)}
								<div class="flex justify-between items-center">
									<span class="text-ctp-text">{lang.language}</span>
									<span class="text-ctp-subtext1 font-mono">{lang.count}</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-ctp-subtext0">No language data available</p>
					{/if}
				</div>

				<!-- Top Projects -->
				<div class="bg-ctp-base rounded-xl shadow p-6">
					<h3 class="text-xl font-semibold text-ctp-text mb-4">Top Projects</h3>
					{#if adminData.stats.top_projects.length > 0}
						<div class="space-y-2">
							{#each adminData.stats.top_projects.slice(0, 10) as project (project.project)}
								<div class="flex justify-between items-center">
									<span class="text-ctp-text truncate">{project.project}</span>
									<span class="text-ctp-subtext1 font-mono">{project.count}</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-ctp-subtext0">No project data available</p>
					{/if}
				</div>
			</div>

			<!-- Daily Activity Chart -->
			<div class="bg-ctp-base rounded-xl shadow p-6 mb-4">
				<h3 class="text-xl font-semibold text-ctp-text mb-4">Daily Activity (Last Week)</h3>
				{#if adminData.stats.daily_activity.length > 0}
					<div id="activity-chart" class="w-full h-64"></div>
				{:else}
					<p class="text-ctp-subtext0">No activity data available</p>
				{/if}
			</div>

			<!-- User Management -->
			<div class="bg-ctp-base rounded-xl shadow p-6">
				<h3 class="text-xl font-semibold text-ctp-text mb-4">All Users</h3>
				{#if adminData.stats.all_users.length > 0}
					<div class="overflow-x-auto">
						<table class="min-w-full divide-y divide-gray-200">
							<thead class="bg-ctp-base">
								<tr>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>User</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>GitHub ID</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Created (UTC)</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Type</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>API Key</th
									>
								</tr>
							</thead>
							<tbody class="bg-ctp-mantle divide-y divide-ctp-surface1">
								{#each adminData.stats.all_users as user (user.github_id)}
									<tr>
										<td class="px-6 py-4 whitespace-nowrap">
											<div class="flex items-center">
												{#if user.avatar_url}
													<img
														src={user.avatar_url}
														alt="Avatar"
														class="h-8 w-8 rounded-full mr-3"
													/>
												{/if}
												<span class="text-sm font-medium text-ctp-text"
													>{user.name || 'Unknown'}</span
												>
											</div>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1"
											>{user.github_id}</td
										>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1"
											>{new Date(user.created_at).toLocaleString('en-US', { timeZone: 'UTC' })}</td
										>
										<td class="px-6 py-4 whitespace-nowrap">
											<UserTag is_admin={user.is_admin} />
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1 font-mono">
											<button
												onclick={() => navigator.clipboard.writeText(user.api_key)}
												class="cursor-pointer hover:bg-ctp-base px-2 py-1 rounded"
												title="Click to copy full API key"
											>
												{user.api_key.substring(0, 16)}...
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-ctp-subtext0">No users found</p>
				{/if}
			</div>
		</div>
	</div>
{/if}
