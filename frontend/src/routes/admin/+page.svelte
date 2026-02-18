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
		DataTable,
		EmptyState,
		Button,
		Pagination
	} from '$lib';
	import { page } from '$app/state';
	import { impersonateUser, changeAdminLevel } from '$lib/api/admin';
	import { createApi } from '$lib/api/api';
	import DateBarChart from '$lib/charts/DateBarChart.svelte';
	import { resolve } from '$app/paths';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let adminData = $derived(data);
	let lastUpdatedAt = $state(new Date());

	const serverAuth = $derived(page.data.auth);
	const currentUser = $derived(serverAuth?.user ?? null);
	const impersonation = $derived(serverAuth?.impersonation ?? null);

	const api = createApi(fetch);

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

	setupVisibilityRefresh({
		refresh: refreshAdminData,
		onError: (error) => {
			console.error('Failed to refresh admin data:', error);
		}
	});

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	const currentOffset = $derived(adminData.offset);
	const limit = $derived(adminData.limit);
	const total = $derived(adminData.total_users);

	function goToPage(offset: number) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		goto(`/admin?offset=${offset}&limit=${limit}`);
	}

	let activeTheme = $derived(browser ? ($theme as Theme) : 'dark');
</script>

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
					value={`~${adminData.total_heartbeats.toLocaleString()}`}
					valueClass="text-3xl font-bold text-ctp-green-600"
				/>
			</div>
		</div>

		<!-- Daily Activity Chart -->
		{#if adminData.daily_activity.length > 0}
			<Container className="mb-4">
				<SectionTitle className="mb-4">Daily Activity (Past Week)</SectionTitle>

				<DateBarChart
					data={adminData.daily_activity}
					seriesName="Heartbeats"
					horizontal={false}
					theme={activeTheme}
					class="w-full h-64"
				/>
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
						<tr class="border-b border-surface0 last:border-0 hover:bg-base/50">
							<td class="pl-6 py-4 whitespace-nowrap text-sm text-subtext1">{user.id}</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="flex items-center">
									{#if user.avatar_url}
										<img src={user.avatar_url} alt="Avatar" class="h-8 w-8 rounded-full mr-3" />
									{/if}
									<a
										class="text-sm font-medium {user.id === currentUser?.id
											? 'text-blue'
											: 'text-text'}"
										href={resolve(`/@[slug]`, { slug: user.name })}>{user.name || 'Unknown'}</a
									>
								</div>
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<UserTag admin_level={user.admin_level} />
							</td>
							<td class="px-6 py-4 whitespace-nowrap text-sm text-subtext1"
								>{new Date(user.created_at).toLocaleString('en-US', { timeZone: 'UTC' })}</td
							>
							{#if showApiKey}
								<td class="px-6 py-4 whitespace-nowrap text-sm text-subtext1 font-mono">
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
							<td class="px-6 py-4 whitespace-nowrap text-sm text-subtext1">
								{#if currentUser?.admin_level !== undefined && user.admin_level >= currentUser.admin_level && (!impersonation || user.id !== impersonation.admin_id)}
									<span class="text-xs uppercase tracking-wide text-subtext1/80">Nothing</span>
								{:else}
									<div class="flex items-center gap-2">
										<Button size="sm" onClick={() => impersonateUser(api, user.id)}>
											{#if impersonation && user.id === impersonation.admin_id}
												Go back
											{:else}
												Impersonate
											{/if}
										</Button>

										{#if currentUser?.admin_level === undefined || (currentUser.admin_level ?? 0) > (user.admin_level ?? 0) + 1}
											<Button
												size="sm"
												variant="confirm"
												onClick={() => {
													void promoteUser(user.id, user.admin_level);
												}}
											>
												Promote
											</Button>
										{:else if (user.admin_level ?? 0) > 0 && (currentUser?.admin_level === undefined || (currentUser.admin_level ?? 0) > (user.admin_level ?? 0))}
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
