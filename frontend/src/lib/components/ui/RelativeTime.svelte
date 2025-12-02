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
	const fallbackDatetime = $derived(datetime ?? new Date());
	let fixedDatetime: Date = $state(new Date());
	let text: string = $state('');

	$effect(() => {
		fixedDatetime = fallbackDatetime;
		text = formatRelativeTime(fallbackDatetime);
	});

	onMount(() => {
		const interval = setInterval(() => {
			text = formatRelativeTime(fixedDatetime);
		}, updateInterval ?? 1000);

		return () => clearInterval(interval);
	});
</script>

<span class={resolvedClassName}>{text}</span>
