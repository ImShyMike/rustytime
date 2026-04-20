<script lang="ts">
	import { browser } from '$app/environment';
	import { Chart } from 'svelte-echarts';
	import { init, use } from 'echarts/core';
	import { PieChart } from 'echarts/charts';
	import { TooltipComponent, LegendComponent } from 'echarts/components';
	import { CanvasRenderer } from 'echarts/renderers';
	import type { UsageStat } from '$lib/types/dashboard';
	import { createPieChartOptions } from './echarts';
	import Skeleton from '$lib/components/ui/Skeleton.svelte';

	use([PieChart, TooltipComponent, LegendComponent, CanvasRenderer]);

	interface Props {
		data: UsageStat[];
		colors?: string[];
		theme?: 'light' | 'dark';
		class?: string;
	}

	let { data, colors = [], theme = 'dark', class: className = '' }: Props = $props();

	let options = $derived(createPieChartOptions(data, colors, theme));
</script>

<div class={className}>
	{#if browser}
		<Chart {init} {options} />
	{:else}
		<Skeleton className="w-full h-full" />
	{/if}
</div>
