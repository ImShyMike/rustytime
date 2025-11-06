<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve -- External GitHub profile links */
	import { tick } from 'svelte';
	import { browser } from '$app/environment';
	import { invalidate } from '$app/navigation';
	import { apexcharts } from '$lib/stores/apexcharts';
	import { theme } from '$lib/stores/theme';
	import type { Theme } from '$lib/stores/theme';
	import type ApexCharts from 'apexcharts';
	import type { PageData } from './$types';
	import {
		destroyChart,
		ensureDateBarChart,
		isApexChartsConstructor
	} from '$lib/charts/apexClient';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { Container, PageScaffold, SectionTitle, StatCard, UserTag } from '$lib';
	import { auth } from '$lib/stores/auth';
	import { impersonateUser, changeAdminLevel } from '$lib/api/admin';
	import { createApi } from '$lib/api/api';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let adminData = $state(data);
	let lastUpdatedAt = $state(new Date());

	const api = createApi(fetch);

	let activityChart: ApexCharts | null = null;

	const refreshAdminData = async () => {
		await invalidate('app:admin');
	};

	const promoteUser = async (userId: number, currentLevel: number | null | undefined) => {
		const nextLevel = (currentLevel ?? 0) + 1;
		await changeAdminLevel(api, userId, nextLevel);
		await refreshAdminData();
	};

	const demoteUser = async (userId: number, currentLevel: number | null | undefined) => {
		const nextLevel = Math.max(0, (currentLevel ?? 0) - 1);
		if (nextLevel === (currentLevel ?? 0)) {
			return;
		}

		await changeAdminLevel(api, userId, nextLevel);
		await refreshAdminData();
	};

	function destroyCharts() {
		activityChart = destroyChart(activityChart);
	}

	setupVisibilityRefresh({
		refresh: refreshAdminData,
		onError: (error) => {
			console.error('Failed to refresh admin data:', error);
		},
		onDispose: destroyCharts
	});

	$effect(() => {
		const payload = data;
		adminData = payload;
		if (payload) {
			lastUpdatedAt = new Date();
		}
	});

	$effect(() => {
		if (!browser) {
			return;
		}

		const activeTheme = $theme as Theme;

		if (!adminData) {
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

		if (!adminData) {
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

			activityChart = await ensureDateBarChart({
				ChartCtor: ApexCharts,
				chart: activityChart,
				elementId: 'activity-chart',
				data: adminData.daily_activity,
				theme,
				seriesName: 'Heartbeats',
				horizontal: false
			});
		} catch (error) {
			console.error('Failed to initialize ApexCharts:', error);
		}
	}
</script>

<svelte:head>
	<title>Admin - rustytime</title>
</svelte:head>

{#if adminData}
	<PageScaffold title="Admin Dashboard" {lastUpdatedAt}>
		<!-- System Statistics -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-4">
			<StatCard
				title="Total Users"
				value={adminData.total_users}
				valueClass="text-3xl font-bold text-ctp-blue-600"
			/>
			<StatCard
				title="Requests/sec"
				value={adminData.requests_per_second}
				valueClass="text-3xl font-bold text-ctp-peach-600"
			/>
			<StatCard
				title="Last hour"
				value={adminData.heartbeats_last_hour.toLocaleString()}
				valueClass="text-3xl font-bold text-ctp-lavender-600"
			/>
			<StatCard
				title="Last 24h"
				value={adminData.heartbeats_last_24h.toLocaleString()}
				valueClass="text-3xl font-bold text-ctp-mauve-600"
			/>
			<div class="md:col-span-2 lg:col-span-1">
				<StatCard
					title="Total Heartbeats"
					value={adminData.total_heartbeats.toLocaleString()}
					valueClass="text-3xl font-bold text-ctp-green-600"
				/>
			</div>
		</div>

		<!-- Daily Activity Chart -->
		{#if adminData.daily_activity.length > 0}
			<Container className="mb-4">
				<SectionTitle className="mb-4">Daily Activity (Past Week)</SectionTitle>

				<div id="activity-chart" class="w-full h-64"></div>
			</Container>
		{:else}
			<Container
				className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center mb-4"
			>
				<p class="text-lg font-semibold text-ctp-text">No recent activity data available</p>
			</Container>
		{/if}

		<!-- User List -->
		{#if adminData.all_users.length > 0}
			<Container>
				<SectionTitle className="mb-4">Users</SectionTitle>
				<div class="overflow-hidden rounded-lg border border-surface0 bg-mantle">
					<table class="w-full">
						<thead class="border-b border-surface0 bg-surface0">
							<tr>
								<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
									>Id</th
								>
								<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
									>User</th
								>
								<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
									>Type</th
								>
								<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
									>Created (UTC)</th
								>
								{#if adminData.all_users[0].api_key}
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>API Key</th
									>
								{/if}
								<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
									>Actions</th
								>
							</tr>
						</thead>
						<tbody>
							{#each [...adminData.all_users].sort((a, b) => {
								const adminDiff = (b.admin_level ?? 0) - (a.admin_level ?? 0);
								if (adminDiff !== 0) return adminDiff;
								return a.id - b.id;
							}) as user (user.id)}
								<tr class="border-b border-surface0 last:border-0 hover:bg-surface0/50">
									<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">{user.id}</td>
									<td class="px-6 py-4 whitespace-nowrap">
										<div class="flex items-center">
											{#if user.avatar_url}
												<img src={user.avatar_url} alt="Avatar" class="h-8 w-8 rounded-full mr-3" />
											{/if}
											<a
												class="text-sm font-medium {user.id === $auth.user?.id
													? 'text-blue'
													: 'text-text'}"
												href={user.name ? `https://github.com/${user.name}` : undefined}
												target="_blank"
												rel="noopener noreferrer external">{user.name || 'Unknown'}</a
											>
										</div>
									</td>
									<td class="px-6 py-4 whitespace-nowrap">
										<UserTag admin_level={user.admin_level} />
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1"
										>{new Date(user.created_at).toLocaleString('en-US', { timeZone: 'UTC' })}</td
									>
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
											<div class="flex items-center gap-2">
												<button
													onclick={() => impersonateUser(api, user.id)}
													class="cursor-pointer inline-flex items-center justify-center rounded bg-ctp-lavender px-3 py-1 text-xs font-semibold text-ctp-base transition hover:bg-ctp-blue"
												>
													{#if $auth.impersonation && user.id === $auth.impersonation.admin_id}
														Go back
													{:else}
														Impersonate
													{/if}
												</button>

												{#if $auth.user?.admin_level === undefined || ($auth.user.admin_level ?? 0) > (user.admin_level ?? 0) + 1}
													<button
														class="cursor-pointer inline-flex items-center justify-center rounded bg-ctp-green px-3 py-1 text-xs font-semibold text-ctp-base transition hover:bg-ctp-teal"
														onclick={() => {
															void promoteUser(user.id, user.admin_level);
														}}
													>
														Promote
													</button>
												{:else if (user.admin_level ?? 0) > 0 && ($auth.user?.admin_level === undefined || ($auth.user.admin_level ?? 0) > (user.admin_level ?? 0))}
													<button
														class="cursor-pointer inline-flex items-center justify-center rounded bg-ctp-red px-3 py-1 text-xs font-semibold text-ctp-base transition hover:bg-ctp-maroon"
														onclick={() => {
															void demoteUser(user.id, user.admin_level);
														}}
													>
														Demote
													</button>
												{/if}
											</div>
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
	</PageScaffold>
{/if}
