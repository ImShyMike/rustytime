<script lang="ts">
	import { browser } from '$app/environment';
	import { createDateBarChartOptions } from '$lib/utils/charts';
	import { apexcharts } from '$lib/stores/apexcharts';
	import type ApexCharts from 'apexcharts';
	import type { ApexOptions } from 'apexcharts';
	import type { PageData } from './$types';
	import { Container, KeyValueList, PageHeading, SectionTitle, StatCard, UserTag } from '$lib';
	import { auth } from '$lib/stores/auth';
	import { impersonateUser } from '$lib/utils/admin';
	import RelativeTime from '$lib/components/ui/RelativeTime.svelte';

	type ApexChartsConstructor = new (element: Element | string, options: ApexOptions) => ApexCharts;

	const isApexChartsConstructor = (value: unknown): value is ApexChartsConstructor =>
		typeof value === 'function';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	// Get admin data from server-side load
	const adminData = data.adminData;

	let lastUpdatedAt = $state(new Date());

	let activityChart: ApexCharts | null = null;

	$effect(() => {
		if (adminData && browser && $apexcharts) {
			initializeCharts();
		}
	});

	async function initializeCharts() {
		if (!adminData) return;

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
						'Heartbeats',
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

<svelte:head>
	<title>Admin - rustytime</title>
</svelte:head>

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
				{#if adminData.stats.top_projects.length !== 0}
					<Container>
						<SectionTitle className="mb-2">Top Languages</SectionTitle>
						<KeyValueList
							items={adminData.stats.top_languages.slice(0, 10).map((lang) => ({
								id: lang.language,
								label: lang.language,
								value: lang.count.toLocaleString()
							}))}
						></KeyValueList>
					</Container>
				{:else}
					<Container
						className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center"
					>
						<p class="text-lg font-semibold text-ctp-text">No language data available</p>
					</Container>
				{/if}

				<!-- Top Projects -->
				{#if adminData.stats.top_projects.length !== 0}
					<Container>
						<SectionTitle className="mb-2">Top Projects</SectionTitle>
						<KeyValueList
							items={adminData.stats.top_projects.slice(0, 10).map((project) => ({
								id: project.project,
								label: project.project,
								value: project.count.toLocaleString(),
								labelClass: 'truncate'
							}))}
						></KeyValueList>
					</Container>
				{:else}
					<Container
						className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center"
					>
						<p class="text-lg font-semibold text-ctp-text">No project data available</p>
					</Container>
				{/if}
			</div>

			<!-- Daily Activity Chart -->
			{#if adminData.stats.daily_activity.length > 0}
				<Container className="mb-4">
					<SectionTitle className="mb-4">Daily Activity (Past Week)</SectionTitle>

					<div id="activity-chart" class="w-full h-64"></div>
				</Container>
			{:else}
				<Container
					className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center mb-4"
				>
					<p class="text-lg font-semibold text-ctp-text">No activity data available</p>
				</Container>
			{/if}

			<!-- User List -->
			{#if adminData.stats.all_users.length > 0}
				<Container>
					<SectionTitle className="mb-4">All Users</SectionTitle>
					<div class="overflow-x-auto">
						<table class="min-w-full divide-y divide-gray-200">
							<thead class="bg-ctp-base">
								<tr>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Id</th
									>
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
									{#if adminData.stats.all_users[0].api_key}
										<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
											>API Key</th
										>
									{/if}
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Actions</th
									>
								</tr>
							</thead>
							<tbody class="bg-ctp-mantle divide-y divide-ctp-surface1">
								{#each [...adminData.stats.all_users].sort((a, b) => {
									const adminDiff = (b.admin_level ?? 0) - (a.admin_level ?? 0);
									if (adminDiff !== 0) return adminDiff;
									return a.id - b.id;
								}) as user (user.id)}
									<tr>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">{user.id}</td>
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
											<UserTag admin_level={user.admin_level} />
										</td>
										{#if user.api_key}
											<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1 font-mono">
												<button
													onclick={() => navigator.clipboard.writeText(user.api_key!)}
													class="cursor-pointer hover:bg-ctp-base px-2 py-1 rounded"
													title="Click to copy full API key"
												>
													{user.api_key.substring(0, 16)}...
												</button>
											</td>
										{/if}
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{#if $auth.user?.admin_level !== undefined && user.admin_level >= $auth.user.admin_level && (!$auth.impersonation || user.id !== $auth.impersonation.admin_id)}
												<span class="text-xs uppercase tracking-wide text-ctp-subtext1/80"
													>Nothing</span
												>
											{:else}
												<button
													onclick={() => impersonateUser(user.id)}
													class="cursor-pointer inline-flex items-center justify-center rounded bg-ctp-lavender px-3 py-1 text-xs font-semibold text-ctp-base transition hover:bg-ctp-blue"
												>
													{#if $auth.impersonation && user.id === $auth.impersonation.admin_id}
														Go back
													{:else}
														Impersonate
													{/if}
												</button>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</Container>
			{:else}
				<Container
					className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center mb-4"
				>
					<p class="text-lg font-semibold text-ctp-text">No users found</p>
					<p class="text-ctp-subtext0">How are you even seeing this</p>
				</Container>
			{/if}
		</div>
	</div>
	{#if lastUpdatedAt}
		<div class="text-center text-ctp-subtext0/85 hover:text-ctp-subtext1 text-sm mb-4" title={lastUpdatedAt.toLocaleString()}>
			Last updated <RelativeTime datetime={lastUpdatedAt} />
		</div>
	{/if}
{/if}
