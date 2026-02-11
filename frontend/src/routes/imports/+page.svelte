<script lang="ts">
	import { invalidate, goto } from '$app/navigation';
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle, DataTable, Pagination, EmptyState } from '$lib';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import { formatDuration } from '$lib/utils/time';
	import { page } from '$app/state';
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

	const columns = [
		{ key: 'id', label: 'ID' },
		{ key: 'user', label: 'User' },
		{ key: 'status', label: 'Status' },
		{ key: 'imported', label: 'Imported' },
		{ key: 'processed', label: 'Processed' },
		{ key: 'requests', label: 'Requests' },
		{ key: 'duration', label: 'Duration' },
		{ key: 'start', label: 'Start (UTC)' },
		{ key: 'created', label: 'Created (UTC)' }
	];

	function goToPage(offset: number) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		goto(`/admin/imports?offset=${offset}&limit=${limit}`);
	}
</script>

{#if importsData}
	<PageScaffold title="Import Jobs" {lastUpdatedAt}>
		<Container>
			<div class="flex items-center justify-between mb-4">
				<SectionTitle>All Import Jobs ({total})</SectionTitle>
			</div>

			{#if importsData.imports.length > 0}
				<DataTable {columns} tableClassName="min-w-lg">
					{#each importsData.imports as job (job.id)}
						{@const StatusIcon = getStatusIcon(job.status)}
						<tr class="border-b border-ctp-surface0 last:border-0 hover:bg-ctp-surface0/50">
							<td class="pl-6 py-4 whitespace-nowrap text-sm text-ctp-subtext1">{job.id}</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="flex items-center">
									{#if job.user_avatar_url}
										<img src={job.user_avatar_url} alt="Avatar" class="h-8 w-8 rounded-full mr-3" />
									{/if}
									<a
										class="text-sm font-medium {job.user_id === page.data.auth?.user?.id
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
				</DataTable>

				<Pagination
					offset={currentOffset}
					{limit}
					{total}
					className="mt-4"
					onchange={(newOffset) => goToPage(newOffset)}
				/>
			{:else}
				<EmptyState
					title="No import jobs found"
					description="No users have started any import jobs yet."
				/>
			{/if}
		</Container>
	</PageScaffold>
{/if}
