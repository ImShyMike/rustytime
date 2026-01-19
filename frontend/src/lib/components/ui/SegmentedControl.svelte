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

<div class="inline-flex rounded-md bg-ctp-surface0/40 p-1 border border-ctp-surface1 {className}">
	{#each options as option (option.value)}
		<button
			type="button"
			class="cursor-pointer px-3 py-1 text-sm rounded transition-colors {selected === option.value
				? 'bg-ctp-blue/80 text-ctp-base'
				: 'text-ctp-text hover:bg-ctp-surface0/60'} {option.disabled
				? 'cursor-not-allowed opacity-50'
				: ''}"
			disabled={option.disabled}
			onclick={() => handleSelect(option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>
