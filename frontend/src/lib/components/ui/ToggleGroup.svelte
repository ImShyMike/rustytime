<script lang="ts" generics="T extends string">
	interface Option {
		value: T;
		label: string;
		disabled?: boolean;
	}

	interface Props {
		options: Option[];
		selected?: T;
		className?: string;
		onchange?: (value: T) => void;
	}

	let {
		options,
		selected = $bindable(options[0]?.value as T),
		className = '',
		onchange
	}: Props = $props();

	function handleSelect(value: T) {
		selected = value;
		onchange?.(value);
	}
</script>

<div class="flex flex-wrap items-stretch gap-3 {className}">
	{#each options as option (option.value)}
		<button
			type="button"
			class="cursor-pointer flex-1 px-3 py-1 rounded text-sm font-medium transition-colors {selected ===
			option.value
				? 'bg-ctp-lavender/90 text-ctp-crust'
				: 'bg-ctp-surface0/70 border border-ctp-surface1 text-ctp-text hover:bg-ctp-surface0'} {option.disabled
				? 'cursor-not-allowed opacity-50'
				: ''}"
			disabled={option.disabled}
			onclick={() => handleSelect(option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>
