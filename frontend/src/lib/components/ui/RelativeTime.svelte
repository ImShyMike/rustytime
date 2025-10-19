<script lang="ts">
	import { onMount } from 'svelte';
	import { formatRelativeTime } from '$lib/utils/time';

	type Props = {
		datetime?: Date;
		className?: string;
		updateInterval?: number;
	};

	let { datetime, className, updateInterval }: Props = $props();

	const resolvedClassName = $derived(className ?? 'inline');
	const fallbackDatetime = datetime ?? new Date();
	let fixedDatetime: Date = fallbackDatetime;

	let text = $state(formatRelativeTime(fallbackDatetime));

	$effect(() => {
		const nextDatetime = datetime ?? fallbackDatetime;
		fixedDatetime = nextDatetime;
		text = formatRelativeTime(nextDatetime);
	});

	onMount(() => {
		const interval = setInterval(() => {
			text = formatRelativeTime(fixedDatetime);
		}, updateInterval ?? 1000);

		return () => clearInterval(interval);
	});
</script>

<span class={resolvedClassName}>{text}</span>
