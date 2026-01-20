<script lang="ts">
	import { Chart } from 'svelte-echarts';
	import { init, use } from 'echarts/core';
	import { BarChart } from 'echarts/charts';
	import { TooltipComponent, GridComponent } from 'echarts/components';
	import { CanvasRenderer } from 'echarts/renderers';
	import type { UsageStat } from '$lib/types/dashboard';
	import { createBarChartOptions } from './echarts';

	use([BarChart, TooltipComponent, GridComponent, CanvasRenderer]);

	interface Props {
		data: UsageStat[];
		colors?: string[];
		horizontal?: boolean;
		theme?: 'light' | 'dark';
		class?: string;
	}

	let {
		data,
		colors = [],
		horizontal = true,
		theme = 'dark',
		class: className = ''
	}: Props = $props();

	let options = $derived(createBarChartOptions(data, colors, horizontal, theme));
</script>

<div class={className}>
	<Chart {init} {options} />
</div>
