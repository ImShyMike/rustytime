import type { UsageStat } from '$lib/types/dashboard';
import type { ApexOptions } from 'apexcharts';

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
		'#74c7ec',
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
		'#209fb5',
	]
};

const resolveColors = (colors: string[], theme: 'light' | 'dark') =>
	colors.length > 0 ? colors : FALLBACK_COLORS[theme];

export function createPieChartOptions(
	data: UsageStat[],
	colors: string[],
	theme: 'light' | 'dark' = 'dark'
): ApexOptions {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const resolvedColors = resolveColors(colors, theme);
	return {
		series: data.map((item) => item.total_seconds),
		chart: {
			foreColor: textColor,
			type: 'pie',
			height: 350,
			background: 'transparent',
			animations: {
				enabled: true,
				speed: 800
			}
		},
		labels: data.map((item) => item.name),
		dataLabels: {
			enabled: false
		},
		colors: resolvedColors,
		fill: {
			type: 'solid',
			colors: resolvedColors,
			opacity: 1
		},
		legend: {
			position: 'right',
			offsetY: 0,
			labels: {
				colors: textColor
			}
		},
		stroke: {
			show: false
		},
		plotOptions: {
			pie: {
				expandOnClick: false
			}
		},
		tooltip: {
			theme: theme,
			y: {
				formatter: (_value, { seriesIndex }) => data[seriesIndex ?? 0]?.text ?? ''
			},
			followCursor: false
		},
		responsive: [
			{
				breakpoint: 480,
				options: {
					legend: {
						position: 'bottom'
					}
				}
			}
		],
		grid: {
			borderColor: gridBorderColor
		}
	} satisfies ApexOptions;
}

export function createBarChartOptions(
	data: UsageStat[],
	colors: string[],
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
): ApexOptions {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const strokeColor = theme === 'dark' ? '#ffffff' : '#000000';
	const resolvedColors = resolveColors(colors, theme);
	return {
		series: [
			{
				name: 'Time Spent',
				data: data.map((item) => item.total_seconds)
			}
		],
		theme: {
			mode: theme
		},
		chart: {
			foreColor: textColor,
			type: 'bar',
			height: 350,
			background: 'transparent',
			animations: {
				enabled: true,
				speed: 800
			},
			toolbar: {
				show: false
			}
		},
		plotOptions: {
			bar: {
				horizontal: horizontal,
				barHeight: '85%',
				distributed: true
			}
		},
		colors: resolvedColors,
		dataLabels: {
			enabled: false
		},
		stroke: {
			width: 1,
			colors: [strokeColor]
		},
		fill: {
			type: 'solid',
			colors: resolvedColors,
			opacity: 1
		},
		xaxis: {
			categories: data.map((item) => item.name),
			labels: {
				show: true,
				style: {
					colors: textColor
				},
				formatter: (value: string) => {
					const numericValue = Number(value);
					return Number.isFinite(numericValue) ? `${Math.floor(numericValue / 3600)}h` : value;
				}
			}
		},
		yaxis: {
			labels: {
				show: true,
				style: {
					colors: textColor
				}
			}
		},
		tooltip: {
			theme: theme,
			x: {
				show: false
			},
			y: {
				title: {
					formatter: (_seriesName, { dataPointIndex }) => data[dataPointIndex ?? 0]?.name ?? ''
				},
				formatter: (value, { dataPointIndex }) => data[dataPointIndex ?? 0]?.text ?? ''
			}
		},
		grid: {
			show: true,
			borderColor: gridBorderColor
		},
		legend: {
			show: false,
			labels: {
				colors: textColor
			}
		}
	} satisfies ApexOptions;
}

export function createDateBarChartOptions(
	data: {
		date: string;
		count: number;
	}[],
	colors: string[],
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
): ApexOptions {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const strokeColor = theme === 'dark' ? '#ffffff' : '#000000';
	const resolvedColors = resolveColors(colors, theme);
	return {
		series: [
			{
				name: 'Time Spent',
				data: data.map((item) => item.count)
			}
		],
		theme: {
			mode: theme
		},
		chart: {
			foreColor: textColor,
			type: 'bar',
			height: 350,
			background: 'transparent',
			animations: {
				enabled: true,
				speed: 800
			},
			toolbar: {
				show: false
			}
		},
		plotOptions: {
			bar: {
				horizontal: horizontal,
				barHeight: '85%',
				distributed: true
			}
		},
		colors: resolvedColors,
		dataLabels: {
			enabled: false
		},
		stroke: {
			width: 1,
			colors: [strokeColor]
		},
		fill: {
			type: 'solid',
			colors: resolvedColors,
			opacity: 1
		},
		xaxis: {
			categories: data.map((item) => item.date),
			labels: {
				show: true,
				style: {
					colors: textColor
				}
			}
		},
		yaxis: {
			labels: {
				show: true,
				style: {
					colors: textColor
				}
			}
		},
		tooltip: {
			theme: theme,
			x: {
				show: false
			}
		},
		grid: {
			show: true,
			borderColor: gridBorderColor
		},
		legend: {
			show: false,
			labels: {
				colors: textColor
			}
		}
	} satisfies ApexOptions;
}
