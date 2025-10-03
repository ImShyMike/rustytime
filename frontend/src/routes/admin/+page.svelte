<script lang="ts">
	import { browser } from '$app/environment';
	import { createDateBarChartOptions } from '$lib/utils/charts';
	import { apexcharts } from '$lib/stores/apexcharts';
	import type ApexCharts from 'apexcharts';
	import type { ApexOptions } from 'apexcharts';
	import type { PageData } from './$types';
	import { Container, KeyValueList, PageHeading, SectionTitle, StatCard, UserTag } from '$lib';

	type ApexChartsConstructor = new (element: Element | string, options: ApexOptions) => ApexCharts;

	const isApexChartsConstructor = (value: unknown): value is ApexChartsConstructor =>
		typeof value === 'function';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	// Get admin data from server-side load
	const adminData = data.adminData;

	let activityChart: ApexCharts | null = null;

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
			<PageHeading title="Admin Dashboard" />

			<!-- System Statistics -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
				<StatCard
					title="Total Users"
					value={adminData.stats.total_users}
					valueClass="text-3xl font-bold text-ctp-blue-600"
				/>
				<StatCard
					title="Total Heartbeats"
					value={adminData.stats.total_heartbeats.toLocaleString()}
					valueClass="text-3xl font-bold text-ctp-green-600"
				/>
				<StatCard
					title="Last 24h"
					value={adminData.stats.heartbeats_last_24h.toLocaleString()}
					valueClass="text-3xl font-bold text-ctp-mauve-600"
				/>
				<StatCard
					title="Requests/sec"
					value={adminData.stats.requests_per_second}
					valueClass="text-3xl font-bold text-ctp-peach-600"
				/>
			</div>

			<!-- Top Lists -->
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
				<!-- Top Languages -->
				<Container>
					<SectionTitle>Top Languages</SectionTitle>
					<KeyValueList
						items={adminData.stats.top_languages.slice(0, 10).map((lang) => ({
							id: lang.language,
							label: lang.language,
							value: lang.count.toLocaleString()
						}))}
					>
						<p slot="empty" class="text-ctp-subtext0">No language data available</p>
					</KeyValueList>
				</Container>

				<!-- Top Projects -->
				<Container>
					<SectionTitle>Top Projects</SectionTitle>
					<KeyValueList
						items={adminData.stats.top_projects.slice(0, 10).map((project) => ({
							id: project.project,
							label: project.project,
							value: project.count.toLocaleString(),
							labelClass: 'truncate'
						}))}
					>
						<p slot="empty" class="text-ctp-subtext0">No project data available</p>
					</KeyValueList>
				</Container>
			</div>

			<!-- Daily Activity Chart -->
			<Container className="mb-4">
				<SectionTitle className="mb-4">Daily Activity (Past Week)</SectionTitle>
				{#if adminData.stats.daily_activity.length > 0}
					<div id="activity-chart" class="w-full h-64"></div>
				{:else}
					<p class="text-ctp-subtext0">No activity data available</p>
				{/if}
			</Container>

			<!-- User List -->
			<Container>
				<SectionTitle className="mb-4">All Users</SectionTitle>
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
			</Container>
		</div>
	</div>
{/if}
