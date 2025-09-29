import { readable } from 'svelte/store';

export const apexcharts = readable<unknown>(null, (set) => {
	if (typeof window !== 'undefined') {
		import('apexcharts').then((module) => set(module.default));
	}
});