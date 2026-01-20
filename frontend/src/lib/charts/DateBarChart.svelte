<script lang="ts">
	import { Chart } from 'svelte-echarts';
	import { init, use } from 'echarts/core';
	import { BarChart } from 'echarts/charts';
	import { TooltipComponent, GridComponent } from 'echarts/components';
	import { CanvasRenderer } from 'echarts/renderers';
	import { createDateBarChartOptions } from './echarts';

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
	<Chart {init} {options} />
</div>
