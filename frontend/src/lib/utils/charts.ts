import type { UsageStat } from '$lib/types/dashboard';

export function createPieChartOptions(
	data: UsageStat[],
	colors: string[],
	theme: 'light' | 'dark' = 'dark'
) {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	return {
		series: data.map((item) => item.total_seconds),
		chart: {
			foreColor: textColor,
			type: 'pie',
			height: 350,
			animations: {
				enabled: true,
				easing: 'easeinout',
				speed: 800
			}
		},
		labels: data.map((item) => item.name),
		dataLabels: {
			enabled: false
		},
		colors: colors,
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
				formatter: function (value: number, { seriesIndex }: { seriesIndex: number }) {
					return data[seriesIndex]?.text || '';
				}
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
	};
}

export function createBarChartOptions(
	data: UsageStat[],
	colors: string[],
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
) {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const strokeColor = theme === 'dark' ? '#ffffff' : '#000000';
	return {
		series: [
			{
				name: 'Time Spent',
				data: data.map((item) => item.total_seconds)
			}
		],
		chart: {
			theme: {
				mode: theme
			},
			foreColor: textColor,
			type: 'bar',
			height: 350,
			animations: {
				enabled: true,
				easing: 'easeinout',
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
		colors: colors,
		dataLabels: {
			enabled: false
		},
		stroke: {
			width: 1,
			colors: [strokeColor]
		},
		xaxis: {
			categories: data.map((item) => item.name),
			labels: {
				show: true,
				style: {
					colors: textColor
				},
				formatter: function (value: number) {
					return `${Math.floor(value / 3600)}h`;
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
					formatter: function (value: number, { dataPointIndex }: { dataPointIndex: number }) {
						return data[dataPointIndex]?.name || '';
					}
				},
				formatter: function (value: number, { dataPointIndex }: { dataPointIndex: number }) {
					return data[dataPointIndex]?.text || '';
				}
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
	};
}

export function createDateBarChartOptions(
	data: {
		date: string;
		count: number;
	}[],
	colors: string[],
	horizontal: boolean = true,
	theme: 'light' | 'dark' = 'dark'
) {
	const textColor = theme === 'dark' ? '#E6EEF3' : '#111827';
	const gridBorderColor = theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)';
	const strokeColor = theme === 'dark' ? '#ffffff' : '#000000';
	return {
		series: [
			{
				name: 'Time Spent',
				data: data.map((item) => item.count)
			}
		],
		chart: {
			theme: {
				mode: theme
			},
			foreColor: textColor,
			type: 'bar',
			height: 350,
			animations: {
				enabled: true,
				easing: 'easeinout',
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
		colors: colors,
		dataLabels: {
			enabled: false
		},
		stroke: {
			width: 1,
			colors: [strokeColor]
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
	};
}
