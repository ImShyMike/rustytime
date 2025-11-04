<script lang="ts">
	import { invalidate } from '$app/navigation';
	import { Container, PageScaffold, RelativeTime } from '$lib';
	import type { PageData } from './$types';
	import type { Leaderboard } from '$lib/types/leaderboard';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { formatDuration } from '$lib/utils/time';
	import { auth } from '$lib/stores/auth';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let leaderboardData = $state(data);
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
		leaderboardData = data;
		lastUpdatedAt = new Date();
	});

	let selectedTab = $state<'daily' | 'weekly' | 'all_time'>('daily');

	const tabs = [
		{ id: 'daily' as const, label: 'Daily' },
		{ id: 'weekly' as const, label: 'Weekly' },
		{ id: 'all_time' as const, label: 'All Time' }
	];

	let currentData = $derived<Leaderboard>(leaderboardData[selectedTab] || []);
</script>

<svelte:head>
	<title>Leaderboard - rustytime</title>
</svelte:head>

<PageScaffold title="Leaderboard" {lastUpdatedAt}>
	<Container>
		<div class="flex justify-between items-end">
			<div class="mb-4 flex gap-2 border-b border-base">
				{#each tabs as tab (tab.id)}
					<button
						class="cursor-pointer px-4 py-2 font-medium transition-colors {selectedTab === tab.id
							? 'border-b-2 border-blue text-blue'
							: 'text-subtext0 hover:text-text'}"
						onclick={() => (selectedTab = tab.id)}
					>
						{tab.label}
					</button>
				{/each}
			</div>

			<p
				class="text-xs text-ctp-overlay1 pb-4"
				title={new Date(currentData.generated_at).toLocaleString()}
			>
				<span class="hidden md:inline">Updated&nbsp;</span><RelativeTime
					datetime={new Date(currentData.generated_at)}
				/>
			</p>
		</div>

		<div class="overflow-hidden rounded-lg border border-surface0 bg-mantle">
			{#if currentData.entries.length === 0}
				<div class="p-8 text-center text-subtext0">No data available for this period</div>
			{:else}
				<table class="w-full table-fixed">
					<thead class="border-b border-surface0 bg-surface0">
						<tr>
							<th class="w-24 pl-6 py-3 text-left text-xs font-medium uppercase text-subtext0"
								>Rank</th
							>
							<th class="w-56 pl-0 pr-6 py-3 text-left text-xs font-medium uppercase text-subtext0"
								>User</th
							>
							<th class="px-6 py-3 text-right text-xs font-medium uppercase text-subtext0"
								>Time Coded</th
							>
						</tr>
					</thead>
					<tbody>
						{#each currentData.entries as entry (entry.rank)}
							<tr class="border-b last:border-0 border-surface0 hover:bg-surface0/20">
								<td class="w-12 pl-6 pr-0 py-4">
									<div
										class="flex h-8 w-8 items-center justify-center rounded-full font-bold {entry.rank ===
										1
											? 'bg-yellow text-base'
											: entry.rank === 2
												? 'bg-overlay0 text-text'
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
											class="font-medium text-text overflow-hidden text-ellipsis whitespace-nowrap {entry.user_id === $auth.user?.id ? 'text-yellow' : ''}"
											href={entry.user_name ? `https://github.com/${entry.user_name}` : undefined}
											target="_blank"
											rel="noopener noreferrer">{entry.user_name || 'Unknown'}</a
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
					</tbody>
				</table>
			{/if}
		</div>
	</Container>
</PageScaffold>
