import {
	createBarChartOptions,
	createPieChartOptions,
	createDateBarChartOptions
} from '$lib/utils/charts';
import type { UsageStat } from '$lib/types/dashboard';
import type ApexCharts from 'apexcharts';
import type { ApexOptions } from 'apexcharts';

export type { ApexOptions };

export type ApexChartsConstructor = new (
	element: Element | string,
	options: ApexOptions
) => ApexCharts;

export const isApexChartsConstructor = (value: unknown): value is ApexChartsConstructor =>
	typeof value === 'function';

type ChartSeries = Array<{ name?: string; data: number[] }> | number[];

interface BaseEnsureParams {
	ChartCtor: ApexChartsConstructor;
	chart: ApexCharts | null;
	elementId: string;
}

interface EnsureBarChartParams extends BaseEnsureParams {
	data: UsageStat[];
	theme: 'light' | 'dark';
	colors?: string[];
	horizontal?: boolean;
}

interface EnsurePieChartParams extends BaseEnsureParams {
	data: UsageStat[];
	theme: 'light' | 'dark';
	colors?: string[];
}

interface EnsureDateBarChartParams extends BaseEnsureParams {
	data: Array<{ date: string; count: number }>;
	theme: 'light' | 'dark';
	colors?: string[];
	seriesName?: string;
	horizontal?: boolean;
}

export async function ensureBarChart({
	ChartCtor,
	chart,
	elementId,
	data,
	theme,
	colors = [],
	horizontal = true
}: EnsureBarChartParams): Promise<ApexCharts | null> {
	const element = document.getElementById(elementId);

	if (!element || data.length === 0) {
		return destroyChart(chart);
	}

	const options = createBarChartOptions(data, colors, horizontal, theme);

	if (!chart) {
		const instance = new ChartCtor(element, options);
		await instance.render();
		return instance;
	}

	chart.updateOptions(
		{
			chart: options.chart,
			theme: options.theme,
			plotOptions: options.plotOptions,
			colors: options.colors,
			dataLabels: options.dataLabels,
			stroke: options.stroke,
			fill: options.fill,
			xaxis: options.xaxis,
			yaxis: options.yaxis,
			tooltip: options.tooltip,
			grid: options.grid,
			legend: options.legend
		},
		true,
		true
	);
	chart.updateSeries(options.series as ChartSeries, true);
	return chart;
}

export async function ensurePieChart({
	ChartCtor,
	chart,
	elementId,
	data,
	theme,
	colors = []
}: EnsurePieChartParams): Promise<ApexCharts | null> {
	const element = document.getElementById(elementId);

	if (!element || data.length === 0) {
		return destroyChart(chart);
	}

	const options = createPieChartOptions(data, colors, theme);

	if (!chart) {
		const instance = new ChartCtor(element, options);
		await instance.render();
		return instance;
	}

	chart.updateOptions(
		{
			chart: options.chart,
			labels: options.labels,
			colors: options.colors,
			fill: options.fill,
			legend: options.legend,
			tooltip: options.tooltip,
			dataLabels: options.dataLabels,
			plotOptions: options.plotOptions,
			grid: options.grid
		},
		true,
		true
	);
	chart.updateSeries(options.series as ChartSeries, true);
	return chart;
}

export async function ensureDateBarChart({
	ChartCtor,
	chart,
	elementId,
	data,
	theme,
	colors = [],
	seriesName = 'Time Spent',
	horizontal = true
}: EnsureDateBarChartParams): Promise<ApexCharts | null> {
	const element = document.getElementById(elementId);

	if (!element || data.length === 0) {
		return destroyChart(chart);
	}

	const options = createDateBarChartOptions(data, colors, seriesName, horizontal, theme);

	if (!chart) {
		const instance = new ChartCtor(element, options);
		await instance.render();
		return instance;
	}

	chart.updateOptions(
		{
			chart: options.chart,
			theme: options.theme,
			plotOptions: options.plotOptions,
			colors: options.colors,
			dataLabels: options.dataLabels,
			stroke: options.stroke,
			fill: options.fill,
			xaxis: options.xaxis,
			yaxis: options.yaxis,
			tooltip: options.tooltip,
			grid: options.grid,
			legend: options.legend
		},
		true,
		true
	);
	chart.updateSeries(options.series as ChartSeries, true);
	return chart;
}

export function destroyChart(chart: ApexCharts | null): null {
	if (chart) {
		chart.destroy();
	}
	return null;
}
