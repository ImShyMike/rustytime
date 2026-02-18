<script lang="ts">
	import { invalidate } from '$app/navigation';
	import { Container, PageScaffold, RelativeTime, Tabs, DataTable } from '$lib';
	import type { PageData } from './$types';
	import type { Leaderboard } from '$lib/types/leaderboard';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { formatDuration } from '$lib/utils/time';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

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

<PageScaffold title="Leaderboard (UTC)" {lastUpdatedAt}>
	<Container>
		<div class="flex justify-between items-end">
			<Tabs {tabs} bind:selected={selectedTab} className="mb-4" />

			<p
				class="text-xs text-overlay1 pb-4"
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
					<td colspan="3" class="p-8 text-center text-subtext0"
						>No data available for this period</td
					>
				</tr>
			{:else}
				{#each currentData.entries as entry (entry.rank)}
					<tr class="border-b last:border-0 border-surface0 hover:bg-base/20">
						<td class="w-12 pl-6 pr-0 py-4">
							<div
								class="flex h-8 w-8 items-center justify-center rounded-full font-bold {entry.rank ===
								1
									? 'bg-yellow text-base'
									: entry.rank === 2
										? 'bg-surface2 text-text'
										: entry.rank === 3
											? 'bg-peach text-base'
											: 'text-subtext0'}"
							>
								{entry.rank}
							</div>
						</td>
						<td class="w-56 py-4">
							<div class="flex items-center gap-3">
								<img
									src={entry.avatar_url}
									alt={entry.user_name}
									class="h-8 w-8 rounded-full border border-surface0"
								/>
								<a
									class="font-medium overflow-hidden text-ellipsis whitespace-nowrap {entry.user_id ===
									page.data.auth?.user?.id
										? 'text-blue'
										: 'text-text'}"
									href={resolve(`/@[slug]`, { slug: entry.user_name })}
									>{entry.user_name || 'Unknown'}</a
								>
							</div>
						</td>
						<td
							class="px-6 py-4 text-right font-mono text-lg text-text"
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
