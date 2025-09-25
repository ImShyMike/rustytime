import type { UsageStat } from '$lib/types/dashboard';

export function createPieChartOptions(data: UsageStat[], colors: string[]) {
	return {
		series: data.map((item) => item.total_seconds),
		chart: {
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
		},
		stroke: {
			show: false,
		},
		plotOptions: {
			pie: {
				expandOnClick: false
			}
		},
		tooltip: {
			y: {
				formatter: function (value: number, { seriesIndex }: { seriesIndex: number }) {
					return data[seriesIndex]?.text || '';
				}
			},
			followCursor: false,
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
		]
	};
}

export function createHorizontalBarChartOptions(data: UsageStat[], colors: string[]) {
	return {
		series: [{
			name: 'Time Spent',
			data: data.map((item) => item.total_seconds)
		}],
		chart: {
			type: 'bar',
			height: 350,
			animations: {
				enabled: true,
				easing: 'easeinout',
				speed: 800
			},
			toolbar: {
				show: false,
			}
		},
		plotOptions: {
			bar: {
				horizontal: true,
				barHeight: '85%',
				distributed: true
			}
		},
		colors: colors,
		dataLabels: {
			enabled: false,
		},
		stroke: {
			width: 1,
			colors: ['#fff']
		},
		xaxis: {
			categories: data.map((item) => item.name),
			labels: {
				show: true,
				formatter: function (value: number) {
					return `${Math.floor(value / 3600)}h`;
				}
			}
		},
		yaxis: {
			labels: {
				show: true
			}
		},
		tooltip: {
			theme: 'dark',
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
			},
		},
		grid: {
			show: false
		},
		legend: {
			show: false
		}
	};
}
