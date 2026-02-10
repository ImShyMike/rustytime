import type { PageLoad } from './$types';
import { createApi, ApiError } from '$lib/api/api';
import { redirect, error } from '@sveltejs/kit';
import type { DashboardResponse } from '$lib/types/dashboard';

export const ssr = false;

export const load: PageLoad = async ({ fetch, depends, url }) => {
	depends('app:dashboard');

	const range = url.searchParams.get('range') || 'day';

	const loadDashboard = async (): Promise<DashboardResponse> => {
		try {
			const api = createApi(fetch);
			return await api.get<DashboardResponse>(`/page/dashboard?range=${range}`);
		} catch (e) {
			console.error('Error loading dashboard page data:', e);
			const err = e as ApiError;
			if (err.status === 401 || err.status === 403) {
				throw redirect(302, '/?auth_error=unauthorized');
			}
			throw error(err.status || 500, err.message);
		}
	};

	return {
		dashboard: loadDashboard(),
		range
	};
};
