import type { PageServerLoad } from './$types';
import { createApi, ApiError } from '$lib/api/api';
import { redirect, error } from '@sveltejs/kit';
import type { DashboardResponse } from '$lib/types/dashboard';

export const load: PageServerLoad = async ({ fetch, depends, request }) => {
	depends('app:dashboard');

	try {
		const cookieHeader = request.headers.get('cookie') || undefined;
		const api = createApi(fetch, cookieHeader);
		return await api.get<DashboardResponse>('/page/dashboard');
	} catch (e) {
		console.error('Error loading dashboard page data:', e);
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
