<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Column {
		key: string;
		label: string;
		className?: string;
		align?: 'left' | 'center' | 'right';
	}

	interface Props {
		columns: Column[];
		className?: string;
		tableClassName?: string;
		children?: Snippet;
	}

	let { columns, className = '', tableClassName = '', children }: Props = $props();

	function getAlignClass(align: string | undefined) {
		switch (align) {
			case 'center':
				return 'text-center';
			case 'right':
				return 'text-right';
			default:
				return 'text-left';
		}
	}
</script>

<div class="rounded-lg border border-surface0 bg-crust {className}">
	<div class="overflow-x-auto">
		<table class="min-w-full w-full {tableClassName}">
			<thead class="border-b border-surface0 bg-base">
				<tr>
					{#each columns as column, i (column.key)}
						<th
							class="px-6 py-3 text-xs font-medium uppercase text-subtext0 {getAlignClass(
								column.align
							)} {i === 0 ? 'pl-6' : ''} {column.className ?? ''}"
						>
							{column.label}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{@render children?.()}
			</tbody>
		</table>
	</div>
</div>
