import type { PageServerLoad } from './$types';
import type { SettingsResponse } from '$lib/types/settings';
import { createApi, ApiError } from '$lib/utils/api';
import { redirect, error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, request }) => {
	try {
		const cookieHeader = request.headers.get('cookie') || undefined;
		const api = createApi(fetch, cookieHeader);
		return await api.get<SettingsResponse>('/page/settings');
	} catch (e) {
		console.error('Error loading settings page data:', e);
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
