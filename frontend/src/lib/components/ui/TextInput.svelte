<script lang="ts">
	type InputType = 'text' | 'password' | 'email' | 'url' | 'number';

	interface Props {
		value?: string;
		type?: InputType;
		placeholder?: string;
		label?: string;
		id?: string;
		disabled?: boolean;
		required?: boolean;
		className?: string;
		oninput?: (value: string) => void;
		onchange?: (value: string) => void;
	}

	let {
		value = $bindable(''),
		type = 'text',
		placeholder = '',
		label = '',
		id = '',
		disabled = false,
		required = false,
		className = '',
		oninput,
		onchange
	}: Props = $props();

	function handleInput(event: Event) {
		const target = event.currentTarget as HTMLInputElement;
		value = target.value;
		oninput?.(value);
	}

	function handleChange(event: Event) {
		const target = event.currentTarget as HTMLInputElement;
		value = target.value;
		onchange?.(value);
	}
</script>

{#if label}
	<label class="flex flex-col gap-1 text-sm text-ctp-subtext0">
		<span class="text-xs uppercase tracking-wide text-ctp-overlay1"
			>{label}{#if required}<span class="text-ctp-red">*</span>{/if}</span
		>
		<input
			{id}
			{type}
			{placeholder}
			{value}
			{disabled}
			{required}
			class="w-full rounded-lg border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text transition-colors placeholder:text-ctp-overlay1 focus:border-ctp-blue focus:outline-none focus:ring-1 focus:ring-ctp-blue/40 disabled:cursor-not-allowed disabled:opacity-50 {className}"
			oninput={handleInput}
			onchange={handleChange}
		/>
	</label>
{:else}
	<input
		{id}
		{type}
		{placeholder}
		{value}
		{disabled}
		{required}
		class="w-full rounded-lg border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text transition-colors placeholder:text-ctp-overlay1 focus:border-ctp-blue focus:outline-none focus:ring-1 focus:ring-ctp-blue/40 disabled:cursor-not-allowed disabled:opacity-50 {className}"
		oninput={handleInput}
		onchange={handleChange}
	/>
{/if}
