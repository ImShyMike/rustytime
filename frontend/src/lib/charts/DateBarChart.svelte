<script lang="ts">
	import { browser } from '$app/environment';
	import { Chart } from 'svelte-echarts';
	import { init, use } from 'echarts/core';
	import { BarChart } from 'echarts/charts';
	import { TooltipComponent, GridComponent } from 'echarts/components';
	import { CanvasRenderer } from 'echarts/renderers';
	import { createDateBarChartOptions } from './echarts';
	import Skeleton from '$lib/components/ui/Skeleton.svelte';

	use([BarChart, TooltipComponent, GridComponent, CanvasRenderer]);

	interface Props {
		data: { date: string; count: number }[];
		colors?: string[];
		seriesName?: string;
		horizontal?: boolean;
		theme?: 'light' | 'dark';
		class?: string;
	}

	let {
		data,
		colors = [],
		seriesName = 'Count',
		horizontal = true,
		theme = 'dark',
		class: className = ''
	}: Props = $props();

	let options = $derived(createDateBarChartOptions(data, colors, seriesName, horizontal, theme));
</script>

<div class={className}>
	{#if browser}
		<Chart {init} {options} />
	{:else}
		<Skeleton className="w-full h-full" />
	{/if}
</div>
