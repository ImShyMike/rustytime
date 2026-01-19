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
		ensureDateBarChart,
		isApexChartsConstructor
	} from '$lib/charts/apexClient';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import {
		Container,
		PageScaffold,
		SectionTitle,
		StatCard,
		UserTag,
		DataTable,
		EmptyState,
		Button,
		Pagination
	} from '$lib';
	import { auth } from '$lib/stores/auth';
	import { impersonateUser, changeAdminLevel } from '$lib/api/admin';
	import { createApi } from '$lib/api/api';
	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let adminData = $derived(data);
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
		if (data) {
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

	const currentOffset = $derived(adminData.offset);
	const limit = $derived(adminData.limit);
	const total = $derived(adminData.total_users);

	function goToPage(offset: number) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		goto(`/admin?offset=${offset}&limit=${limit}`);
	}

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
			<EmptyState title="No recent activity data available" className="mb-4" />
		{/if}

		<!-- User List -->
		{#if adminData.all_users.length > 0}
			{@const showApiKey = !!adminData.all_users[0].api_key}
			{@const columns = [
				{ key: 'id', label: 'Id' },
				{ key: 'user', label: 'User' },
				{ key: 'type', label: 'Type' },
				{ key: 'created', label: 'Created (UTC)' },
				...(showApiKey ? [{ key: 'api_key', label: 'API Key' }] : []),
				{ key: 'actions', label: 'Actions' }
			]}
			<Container>
				<SectionTitle className="mb-4">Users</SectionTitle>
				<DataTable {columns} tableClassName="min-w-lg">
					{#each adminData.all_users as user (user.id)}
						<tr class="border-b border-ctp-surface0 last:border-0 hover:bg-ctp-surface0/50">
							<td class="pl-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">{user.id}</td>
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
										data-umami-event="github-profile-link"
										data-umami-event-name={user.name}
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
							{#if showApiKey}
								<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1 font-mono">
									{#if user.api_key}
										<Button
											variant="ghost"
											size="sm"
											onClick={() => navigator.clipboard.writeText(user.api_key!)}
											className="font-mono px-2 py-1"
										>
											{user.api_key.substring(0, 16)}...
										</Button>
									{/if}
								</td>
							{/if}
							<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
								{#if $auth.user?.admin_level !== undefined && user.admin_level >= $auth.user.admin_level && (!$auth.impersonation || user.id !== $auth.impersonation.admin_id)}
									<span class="text-xs uppercase tracking-wide text-ctp-subtext1/80">Nothing</span>
								{:else}
									<div class="flex items-center gap-2">
										<Button size="sm" onClick={() => impersonateUser(api, user.id)}>
											{#if $auth.impersonation && user.id === $auth.impersonation.admin_id}
												Go back
											{:else}
												Impersonate
											{/if}
										</Button>

										{#if $auth.user?.admin_level === undefined || ($auth.user.admin_level ?? 0) > (user.admin_level ?? 0) + 1}
											<Button
												size="sm"
												variant="confirm"
												onClick={() => {
													void promoteUser(user.id, user.admin_level);
												}}
											>
												Promote
											</Button>
										{:else if (user.admin_level ?? 0) > 0 && ($auth.user?.admin_level === undefined || ($auth.user.admin_level ?? 0) > (user.admin_level ?? 0))}
											<Button
												variant="danger"
												size="sm"
												onClick={() => {
													void demoteUser(user.id, user.admin_level);
												}}
											>
												Demote
											</Button>
										{/if}
									</div>
								{/if}
							</td>
						</tr>
					{/each}
				</DataTable>

				<Pagination
					offset={currentOffset}
					{limit}
					{total}
					className="mt-4"
					onchange={(newOffset) => goToPage(newOffset)}
				/>
			</Container>
		{:else}
			<EmptyState
				title="No users found"
				description="How are you even seeing this"
				className="mb-4"
			/>
		{/if}
	</PageScaffold>
{/if}
