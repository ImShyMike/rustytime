<script lang="ts">
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
</section>
