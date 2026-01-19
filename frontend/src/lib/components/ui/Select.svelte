<script lang="ts" generics="T extends string | number">
	interface Option {
		label: string;
		value: T;
		disabled?: boolean;
	}

	interface Props {
		value?: T;
		options: Option[];
		placeholder?: string;
		className?: string;
		disabled?: boolean;
		label?: string;
		id?: string;
		onchange?: (value: T) => void;
	}

	let {
		value = $bindable(undefined as unknown as T),
		options,
		placeholder = 'Select...',
		className = '',
		disabled = false,
		label = '',
		id = '',
		onchange
	}: Props = $props();

	function handleChange(event: Event) {
		const target = event.currentTarget as HTMLSelectElement;
		const newValue = (typeof value === 'number' ? Number(target.value) : target.value) as T;
		value = newValue;
		onchange?.(newValue);
	}
</script>

{#if label}
	<label class="flex flex-col gap-1 text-sm text-ctp-subtext0">
		<span class="text-xs uppercase tracking-wide text-ctp-overlay1">{label}</span>
		<select
			{id}
			class="rounded-lg border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text transition-colors focus:border-ctp-blue focus:outline-none focus:ring-1 focus:ring-ctp-blue/40 disabled:cursor-not-allowed disabled:opacity-50 {className}"
			{value}
			{disabled}
			onchange={handleChange}
		>
			{#if placeholder}
				<option value={undefined} disabled>{placeholder}</option>
			{/if}
			{#each options as option (option.value)}
				<option value={option.value} disabled={option.disabled}>{option.label}</option>
			{/each}
		</select>
	</label>
{:else}
	<select
		{id}
		class="rounded-lg border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text transition-colors focus:border-ctp-blue focus:outline-none focus:ring-1 focus:ring-ctp-blue/40 disabled:cursor-not-allowed disabled:opacity-50 {className}"
		{value}
		{disabled}
		onchange={handleChange}
	>
		{#if placeholder}
			<option value={undefined} disabled>{placeholder}</option>
		{/if}
		{#each options as option (option.value)}
			<option value={option.value} disabled={option.disabled}>{option.label}</option>
		{/each}
	</select>
{/if}
