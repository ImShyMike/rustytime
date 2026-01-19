<script lang="ts">
	import { invalidate } from '$app/navigation';
	import { Container, PageScaffold, RelativeTime, Tabs, DataTable } from '$lib';
	import type { PageData } from './$types';
	import type { Leaderboard } from '$lib/types/leaderboard';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { formatDuration } from '$lib/utils/time';
	import { auth } from '$lib/stores/auth';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let leaderboardData = $derived(data);
	let lastUpdatedAt = $state(new Date());

	const refreshLeaderboard = async () => {
		await invalidate('app:leaderboard');
	};

	setupVisibilityRefresh({
		refresh: refreshLeaderboard,
		onError: (error) => {
			console.error('Failed to refresh leaderboard:', error);
		}
	});

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	let selectedTab = $state<'daily' | 'weekly' | 'all_time'>('daily');

	const tabs = [
		{ id: 'daily' as const, label: 'Daily' },
		{ id: 'weekly' as const, label: 'Weekly' },
		{ id: 'all_time' as const, label: 'All Time' }
	];

	let currentData = $derived<Leaderboard>(leaderboardData[selectedTab] || []);

	const columns = [
		{ key: 'rank', label: 'Rank', className: 'w-24' },
		{ key: 'user', label: 'User', className: 'w-56' },
		{ key: 'time', label: 'Time Coded', align: 'right' as const }
	];
</script>

<svelte:head>
	<title>Leaderboard - rustytime</title>
</svelte:head>

<PageScaffold title="Leaderboard" {lastUpdatedAt}>
	<Container>
		<div class="flex justify-between items-end">
			<Tabs {tabs} bind:selected={selectedTab} className="mb-4" />

			<p
				class="text-xs text-ctp-overlay1 pb-4"
				title={new Date(currentData.generated_at).toLocaleString()}
			>
				<span class="hidden md:inline">Updated&nbsp;</span><RelativeTime
					datetime={new Date(currentData.generated_at)}
				/>
			</p>
		</div>

		<DataTable {columns} tableClassName="table-fixed">
			{#if currentData.entries.length === 0}
				<tr>
					<td colspan="3" class="p-8 text-center text-ctp-subtext0"
						>No data available for this period</td
					>
				</tr>
			{:else}
				{#each currentData.entries as entry (entry.rank)}
					<tr class="border-b last:border-0 border-ctp-surface0 hover:bg-ctp-surface0/20">
						<td class="w-12 pl-6 pr-0 py-4">
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full font-bold {entry.rank ===
								1
									? 'bg-ctp-yellow text-ctp-base'
									: entry.rank === 2
										? 'bg-ctp-overlay0 text-ctp-text'
										: entry.rank === 3
											? 'bg-ctp-peach text-ctp-base'
											: 'text-ctp-subtext0'}"
							>
								{entry.rank}
							</div>
						</td>
						<td class="w-56 py-4">
							<div class="flex items-center gap-3">
								<img
									src={entry.avatar_url}
									alt={entry.user_name}
									class="h-8 w-8 rounded-full border border-ctp-surface0"
								/>
								<a
									class="font-medium overflow-hidden text-ellipsis whitespace-nowrap {entry.user_id ===
									$auth.user?.id
										? 'text-ctp-blue'
										: 'text-ctp-text'}"
									href={entry.user_name ? `https://github.com/${entry.user_name}` : undefined}
									target="_blank"
									data-umami-event="github-profile-link"
									data-umami-event-name={entry.user_name}
									rel="noopener noreferrer external">{entry.user_name || 'Unknown'}</a
								>
							</div>
						</td>
						<td
							class="px-6 py-4 text-right font-mono text-lg text-ctp-text"
							title={formatDuration(entry.total_seconds)}
						>
							{formatDuration(entry.total_seconds, false)}
						</td>
					</tr>
				{/each}
			{/if}
		</DataTable>
	</Container>
</PageScaffold>
