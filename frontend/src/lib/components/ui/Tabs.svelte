<script lang="ts" generics="T extends string">
	interface Tab {
		id: T;
		label: string;
		disabled?: boolean;
	}

	interface Props {
		tabs: Tab[];
		selected?: T;
		className?: string;
		onchange?: (id: T) => void;
	}

	let { tabs, selected = $bindable(tabs[0]?.id as T), className = '', onchange }: Props = $props();

	function handleSelect(id: T) {
		selected = id;
		onchange?.(id);
	}
</script>

<div class="flex gap-2 border-b border-ctp-base {className}">
	{#each tabs as tab (tab.id)}
		<button
			type="button"
			class="cursor-pointer px-4 py-2 font-medium transition-colors {selected === tab.id
				? 'border-b-2 border-ctp-blue text-ctp-blue'
				: 'text-ctp-subtext0 hover:text-ctp-text'} {tab.disabled
				? 'cursor-not-allowed opacity-50'
				: ''}"
			disabled={tab.disabled}
			onclick={() => handleSelect(tab.id)}
		>
			{tab.label}
		</button>
	{/each}
</div>
