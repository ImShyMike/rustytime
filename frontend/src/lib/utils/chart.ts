import type { UsageStat } from '$lib/types/dashboard';

export function createPieChartOptions(data: UsageStat[], colors: string[], title: string) {
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
		colors: colors,
		title: {
			text: title,
			align: 'center',
			style: {
				fontSize: '16px',
				fontWeight: 600,
				color: '#374151'
			}
		},
		legend: {
			position: 'right',
			offsetY: 0,
			height: 230
		},
		stroke: {
			show: data.length > 1,
			width: data.length > 1 ? 1 : 0
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
			}
		},
		responsive: [
			{
				breakpoint: 480,
				options: {
					chart: {
						width: 200
					},
					legend: {
						position: 'bottom'
					}
				}
			}
		]
	};
}
