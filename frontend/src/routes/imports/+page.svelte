<script lang="ts">
	import { invalidate, goto } from '$app/navigation';
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle, Button } from '$lib';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { formatDuration } from '$lib/utils/time';
	import { auth } from '$lib/stores/auth';
	import LucideChevronLeft from '~icons/lucide/chevron-left';
	import LucideChevronRight from '~icons/lucide/chevron-right';
	import LucideLoader2 from '~icons/lucide/loader-2';
	import LucideCheck from '~icons/lucide/check';
	import LucideX from '~icons/lucide/x';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let importsData = $derived(data);
	let lastUpdatedAt = $state(new Date());

	const refreshImportsData = async () => {
		await invalidate('app:admin-imports');
	};

	setupVisibilityRefresh({
		refresh: refreshImportsData,
		onError: (error) => {
			console.error('Failed to refresh imports data:', error);
		}
	});

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	function formatDate(value: string) {
		const date = new Date(value);
		return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
	}

	function formatStartDate(value: string | null) {
		if (!value) return 'N/A';
		const date = new Date(value);
		return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
	}

	function getStatusIcon(status: string) {
		switch (status) {
			case 'completed':
				return LucideCheck;
			case 'failed':
				return LucideX;
			default:
				return LucideLoader2;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'completed':
				return 'text-green';
			case 'failed':
				return 'text-red';
			default:
				return 'text-yellow';
		}
	}

	const currentOffset = $derived(importsData.offset);
	const limit = $derived(importsData.limit);
	const total = $derived(importsData.total);
	const hasPrevious = $derived(currentOffset > 0);
	const hasNext = $derived(currentOffset + limit < total);
	const currentPage = $derived(Math.floor(currentOffset / limit) + 1);
	const totalPages = $derived(Math.ceil(total / limit));

	function goToPage(offset: number) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		goto(`/admin/imports?offset=${offset}&limit=${limit}`);
	}

	function previousPage() {
		goToPage(Math.max(0, currentOffset - limit));
	}

	function nextPage() {
		goToPage(currentOffset + limit);
	}
</script>

<svelte:head>
	<title>Imports - Admin - rustytime</title>
</svelte:head>

{#if importsData}
	<PageScaffold title="Import Jobs" {lastUpdatedAt}>
		<Container>
			<div class="flex items-center justify-between mb-4">
				<SectionTitle>All Import Jobs ({total})</SectionTitle>
			</div>

			{#if importsData.imports.length > 0}
				<div class="rounded-lg border border-surface0 bg-mantle">
					<div class="overflow-x-auto">
						<table class="min-w-lg w-full">
							<thead class="border-b border-surface0 bg-surface0">
								<tr>
									<th class="pl-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>ID</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>User</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Status</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Imported</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Processed</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Requests</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Duration</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Start (UTC)</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-ctp-subtext0 uppercase"
										>Created (UTC)</th
									>
								</tr>
							</thead>
							<tbody>
								{#each importsData.imports as job (job.id)}
									{@const StatusIcon = getStatusIcon(job.status)}
									<tr class="border-b border-surface0 last:border-0 hover:bg-surface0/50">
										<td class="pl-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">{job.id}</td>
										<td class="px-6 py-4 whitespace-nowrap">
											<div class="flex items-center">
												{#if job.user_avatar_url}
													<img
														src={job.user_avatar_url}
														alt="Avatar"
														class="h-8 w-8 rounded-full mr-3"
													/>
												{/if}
												<a
													class="text-sm font-medium {job.user_id === $auth.user?.id
														? 'text-blue'
														: 'text-text'}"
													href={job.user_name ? `https://github.com/${job.user_name}` : undefined}
													target="_blank"
													data-umami-event="github-profile-link"
													data-umami-event-name={job.user_name}
													rel="noopener noreferrer external">{job.user_name || 'Unknown'}</a
												>
											</div>
										</td>
										<td class="px-6 py-4 whitespace-nowrap">
											<div class="flex items-center gap-2">
												<StatusIcon
													class={`w-4 h-4 ${getStatusColor(job.status)} ${job.status === 'running' ? 'animate-spin' : ''}`}
												/>
												<span class={`text-sm font-medium capitalize ${getStatusColor(job.status)}`}
													>{job.status}</span
												>
											</div>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{job.imported_count?.toLocaleString() ?? '-'}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{job.processed_count?.toLocaleString() ?? '-'}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{job.request_count?.toLocaleString() ?? '-'}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{job.time_taken ? formatDuration(job.time_taken) : '-'}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">
											{formatStartDate(job.start_date)}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1"
											>{formatDate(job.created_at)}</td
										>
									</tr>
									{#if job.error_message}
										<tr class="bg-red/5">
											<td colspan="9" class="px-6 py-2">
												<div class="text-sm text-red">
													<span class="font-medium">Error:</span>
													{job.error_message}
												</div>
											</td>
										</tr>
									{/if}
								{/each}
							</tbody>
						</table>
					</div>
				</div>

				<!-- Pagination -->
				{#if totalPages > 1}
					<div class="flex items-center justify-between mt-4">
						<p class="text-sm text-subtext0">
							Showing {currentOffset + 1} - {Math.min(currentOffset + limit, total)} of {total} imports
						</p>
						<div class="flex items-center gap-2">
							<Button onClick={previousPage} disabled={!hasPrevious} className="p-2">
								<LucideChevronLeft class="w-4 h-4" />
							</Button>
							<span class="text-sm text-subtext0">
								Page {currentPage} of {totalPages}
							</span>
							<Button onClick={nextPage} disabled={!hasNext} className="p-2">
								<LucideChevronRight class="w-4 h-4" />
							</Button>
						</div>
					</div>
				{/if}
			{:else}
				<div
					class="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center"
				>
					<p class="text-lg font-semibold text-ctp-text">No import jobs found</p>
					<p class="text-ctp-subtext0">No users have started any import jobs yet.</p>
				</div>
			{/if}
		</Container>
	</PageScaffold>
{/if}
