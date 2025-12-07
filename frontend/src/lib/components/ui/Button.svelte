<script lang="ts">
	export let className = '';
	export let onClick: () => void = () => {};
	export let disabled: boolean = false;
</script>

<button
	type="button"
	class={`rounded-lg border px-4 py-2 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:border-ctp-surface1 disabled:bg-ctp-surface0/40 disabled:text-ctp-subtext1 cursor-pointer border-ctp-blue text-ctp-blue hover:bg-ctp-blue/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ctp-blue/60 ${className}`.trim()}
	onclick={onClick}
	{disabled}
>
	<slot />
</button>
