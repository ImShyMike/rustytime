import { api } from '$lib/utils/api';
import { writable, type Writable } from 'svelte/store';

export function createDataLoader<T>(endpoint: string) {
	const data: Writable<T | null> = writable(null);
	const loading: Writable<boolean> = writable(false);
	const error: Writable<string | null> = writable(null);

	async function loadData() {
		loading.set(true);
		error.set(null);
		try {
			const response = await api.get<T>(endpoint);
			data.set(response);
		} catch (err) {
			console.error(`Failed to fetch data from ${endpoint}:`, err);
			error.set('Failed to load data');
		} finally {
			loading.set(false);
		}
	}

	return { data, loading, error, loadData };
}
