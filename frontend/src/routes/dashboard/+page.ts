import type { PageLoad } from './$types';
import { createApi, ApiError } from '$lib/utils/api';
import { redirect, error } from '@sveltejs/kit';
import type { DashboardResponse } from '$lib/types/dashboard';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:dashboard');

	try {
		const api = createApi(fetch);
		return await api.get<DashboardResponse>('/page/dashboard');
	} catch (e) {
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
