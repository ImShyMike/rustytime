<script lang="ts">
	import LucideArrowLeft from '~icons/lucide/arrow-left';

	export let status: number = 500;
	export let error: { message?: string } | null = null;

	$: meaning = error?.message ?? (status >= 500 ? 'Server Error' : 'Unexpected Error');
</script>

<section
	class="flex min-h-[70vh] flex-col items-center justify-center gap-6 px-6 py-12 text-center"
	aria-labelledby="error-title"
>
	<header class="relative grid max-w-xl place-items-center" id="error-title">
		<span
			class="text-[clamp(6rem,18vw,12rem)] font-bold tracking-[-0.05em] text-subtext0/10"
			aria-hidden="true"
		>
			{status}
		</span>
		<div class="absolute">
			<p class="m-0 text-2xl font-bold text-lavender">{meaning}</p>
		</div>
	</header>

	<button
		on:click={() => window.history.back()}
		class="inline-flex items-center gap-2 rounded-md bg-surface0/50 px-4 py-2 text-sm font-medium text-text hover:bg-surface0 cursor-pointer"
	>
		<LucideArrowLeft class="h-4 w-4" aria-hidden="true" />
		<span>Go Back</span>
	</button>
</section>
