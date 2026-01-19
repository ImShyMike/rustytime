<script lang="ts">
	import LucideSearch from '~icons/lucide/search';
	import LucideX from '~icons/lucide/x';

	interface Props {
		value?: string;
		placeholder?: string;
		className?: string;
		showClear?: boolean;
		oninput?: (value: string) => void;
		onchange?: (value: string) => void;
	}

	let {
		value = $bindable(''),
		placeholder = 'Search...',
		className = '',
		showClear = true,
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

	function handleClear() {
		value = '';
		oninput?.('');
		onchange?.('');
	}
</script>

<label
	class="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-ctp-surface1 bg-ctp-base/60 px-3 py-2 text-sm text-ctp-text shadow-sm transition-colors focus-within:border-ctp-blue focus-within:bg-ctp-base focus-within:ring-1 focus-within:ring-ctp-blue/40 {className}"
>
	<LucideSearch class="h-4 w-4 shrink-0 text-ctp-overlay1" aria-hidden="true" />
	<input
		type="search"
		class="w-full bg-transparent text-sm text-ctp-text placeholder:text-ctp-overlay1 focus:outline-none"
		{placeholder}
		{value}
		oninput={handleInput}
		onchange={handleChange}
	/>
	{#if showClear && value}
		<button
			type="button"
			onclick={handleClear}
			class="shrink-0 cursor-pointer text-ctp-overlay1 hover:text-ctp-text"
			aria-label="Clear search"
		>
			<LucideX class="h-4 w-4" />
		</button>
	{/if}
</label>
