<script lang="ts">
	export let status: number = 500;
	export let error: { message?: string } | null = null;

	const statusMeanings: Record<number, string> = {
		400: 'Bad Request',
		401: 'Unauthorized',
		403: 'Forbidden',
		404: 'Not Found',
		408: 'Request Timeout',
		409: 'Conflict',
		410: 'Gone',
		418: "I'm a teapot",
		422: 'Unprocessable Content',
		429: 'Too Many Requests',
		500: 'Internal Server Error',
		502: 'Bad Gateway',
		503: 'Service Unavailable',
		504: 'Gateway Timeout'
	};

	$: meaning = statusMeanings[status] ?? (status >= 500 ? 'Server Error' : 'Unexpected Error');
	$: description = error?.message ?? 'Something went wrong while loading this page.';
</script>

<section
	class="flex min-h-[70vh] flex-col items-center justify-center gap-6 px-6 py-12 text-center"
	aria-labelledby="error-title"
>
	<header
		class="relative grid max-w-xl place-items-center"
		id="error-title"
	>
		<span
			class="text-[clamp(6rem,18vw,12rem)] font-bold tracking-[-0.05em] text-subtext0/10"
			aria-hidden="true"
		>
			{status}
		</span>
		<div
			class="absolute"
		>
			<p class="m-0 text-2xl font-bold text-lavender">{meaning}</p>
		</div>
	</header>

	<p class="m-0 max-w-md text-md leading-relaxed text-subtext0">
		{description}
	</p>
</section>
