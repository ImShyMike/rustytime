<script lang="ts">
	import LucideChevronLeft from '~icons/lucide/chevron-left';
	import LucideChevronRight from '~icons/lucide/chevron-right';
	import Button from './Button.svelte';

	interface Props {
		offset: number;
		limit: number;
		total: number;
		className?: string;
		onchange?: (offset: number) => void;
	}

	let { offset, limit, total, className = '', onchange }: Props = $props();

	const hasPrevious = $derived(offset > 0);
	const hasNext = $derived(offset + limit < total);
	const currentPage = $derived(Math.floor(offset / limit) + 1);
	const totalPages = $derived(Math.ceil(total / limit));
	const showingStart = $derived(offset + 1);
	const showingEnd = $derived(Math.min(offset + limit, total));

	function goToPrevious() {
		const newOffset = Math.max(0, offset - limit);
		onchange?.(newOffset);
	}

	function goToNext() {
		const newOffset = offset + limit;
		onchange?.(newOffset);
	}
</script>

{#if totalPages > 1}
	<div class="flex items-center justify-between {className}">
		<p class="text-sm text-ctp-subtext0">
			Showing {showingStart} - {showingEnd} of {total}
		</p>
		<div class="flex items-center gap-2">
			<Button onClick={goToPrevious} disabled={!hasPrevious} className="p-2">
				<LucideChevronLeft class="h-4 w-4" />
			</Button>
			<span class="text-sm text-ctp-subtext0">
				Page {currentPage} of {totalPages}
			</span>
			<Button onClick={goToNext} disabled={!hasNext} className="p-2">
				<LucideChevronRight class="h-4 w-4" />
			</Button>
		</div>
	</div>
{/if}
