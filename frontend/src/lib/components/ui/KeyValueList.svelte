<script lang="ts">
	type ListItem = {
		id?: string | number;
		label: string;
		value: string | number;
		labelClass?: string;
		valueClass?: string;
	};

	export let items: ListItem[] = [];
	export let className = '';
	export let rowClass = 'flex items-center gap-2 justify-between';
	export let labelClass = 'text-text flex-1 min-w-0';
	export let valueClass = 'text-subtext1 font-mono text-right';
</script>

{#if items.length > 0}
	<div class={`space-y ${className}`.trim()}>
		{#each items as item, index (item.id ?? `${item.label}-${index}`)}
			<div class={`${rowClass}`.trim()}>
				<span class={`${labelClass} ${item.labelClass ?? ''}`.trim()}>{item.label}</span>
				<span class={`${valueClass} ${item.valueClass ?? ''}`.trim()}>{item.value}</span>
			</div>
		{/each}
	</div>
{:else}
	<div class={className}>
		<slot name="empty">
			<p class="text-subtext0">No data available</p>
		</slot>
	</div>
{/if}
