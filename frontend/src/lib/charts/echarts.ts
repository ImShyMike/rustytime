import type { UsageStat } from '$lib/types/dashboard';
import type { EChartsOption } from 'echarts';

const FALLBACK_COLORS: Record<'light' | 'dark', string[]> = {
	dark: [
		'#89b4fa',
		'#f5c2e7',
		'#cba6f7',
		'#f38ba8',
		'#fab387',
		'#f9e2af',
		'#a6e3a1',
		'#94e2d5',
		'#89dceb',
		'#74c7ec'
	],
	light: [
		'#1e66f5',
		'#ea76cb',
		'#8839ef',
		'#d20f39',
		'#fe640b',
		'#df8e1d',
		'#40a02b',
		'#179299',
		'#04a5e5',
		'#209fb5'
	]
};

const resolveColors = (colors: string[], theme: 'light' | 'dark') =>
	colors.length > 0 ? colors : FALLBACK_COLORS[theme];

export function createPieChartOptions(
	data: UsageStat[],
	colors: string[],
	theme: 'light' | 'dark' = 'dark'
): EChartsOption {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const resolvedColors = resolveColors(colors, theme);

	return {
		color: resolvedColors,
		tooltip: {
			trigger: 'item',
			formatter: (params) => {
				const p = params as { name?: string };
				const item = data.find((d) => d.name === p.name);
				return `${p.name}: ${item?.text ?? ''}`;
			}
		},
		legend: {
			orient: 'vertical',
			right: 5,
			top: 'center',
			textStyle: {
				color: textColor
			}
		},
		series: [
			{
				type: 'pie',
				radius: ['0%', '90%'],
				center: ['45%', '50%'],
				avoidLabelOverlap: false,
				label: {
					show: false
				},
				emphasis: {
					scale: false
				},
				data: data.map((item) => ({
					value: item.total_seconds,
					name: item.name,
					text: item.text
				}))
			}
		],
		media: [
			{
				query: { maxWidth: 480 },
				option: {
					legend: {
						orient: 'horizontal',
						bottom: 0,
						left: 'center',
						right: undefined,
						top: undefined
					},
					series: [
						{
							radius: ['0%', '75%'],
							center: ['50%', '45%']
						}
					]
				}
			}
		]
	};
}

export function createBarChartOptions(
	data: UsageStat[],
	colors: string[],
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
): EChartsOption {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const resolvedColors = resolveColors(colors, theme);

	const categoryAxis = {
		type: 'category' as const,
		data: data.map((item) => item.name),
		axisLabel: {
			color: textColor
		},
		axisLine: {
			lineStyle: {
				color: gridBorderColor
			}
		}
	};

	const valueAxis = {
		type: 'value' as const,
		axisLabel: {
			color: textColor,
			formatter: (value: number) => `${Math.floor(value / 3600)}h`
		},
		axisLine: {
			lineStyle: {
				color: gridBorderColor
			}
		},
		splitLine: {
			lineStyle: {
				color: gridBorderColor
			}
		}
	};

	return {
		color: resolvedColors,
		tooltip: {
			trigger: 'axis',
			axisPointer: {
				type: 'shadow'
			},
			formatter: (params) => {
				const p = params as Array<{ dataIndex?: number }>;
				const idx = p[0]?.dataIndex ?? 0;
				const item = data[idx];
				return `${item?.name ?? ''}: ${item?.text ?? ''}`;
			}
		},
		grid: {
			left: '1%',
			right: '2%',
			bottom: '1%',
			top: '3%',
			containLabel: true
		},
		xAxis: horizontal ? valueAxis : categoryAxis,
		yAxis: horizontal ? categoryAxis : valueAxis,
		series: [
			{
				type: 'bar',
				data: data.map((item, index) => ({
					value: item.total_seconds,
					itemStyle: {
						color: resolvedColors[index % resolvedColors.length]
					}
				})),
				barWidth: '60%'
			}
		]
	};
}

export function createDateBarChartOptions(
	data: { date: string; count: number }[],
	colors: string[],
	seriesName: string = 'Time Spent',
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
): EChartsOption {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const resolvedColors = resolveColors(colors, theme);

	const categoryAxis = {
		type: 'category' as const,
		data: data.map((item) => item.date),
		axisLabel: {
			color: textColor
		},
		axisLine: {
			lineStyle: {
				color: gridBorderColor
			}
		}
	};

	const valueAxis = {
		type: 'value' as const,
		axisLabel: {
			color: textColor
		},
		axisLine: {
			lineStyle: {
				color: gridBorderColor
			}
		},
		splitLine: {
			lineStyle: {
				color: gridBorderColor
			}
		}
	};

	return {
		color: resolvedColors,
		tooltip: {
			trigger: 'axis',
			axisPointer: {
				type: 'shadow'
			}
		},
		grid: {
			left: '1%',
			right: '1%',
			bottom: '1%',
			top: '3%',
			containLabel: true
		},
		xAxis: horizontal ? valueAxis : categoryAxis,
		yAxis: horizontal ? categoryAxis : valueAxis,
		series: [
			{
				name: seriesName,
				type: 'bar',
				data: data.map((item, index) => ({
					value: item.count,
					itemStyle: {
						color: resolvedColors[index % resolvedColors.length]
					}
				})),
				barWidth: '60%'
			}
		]
	};
}
