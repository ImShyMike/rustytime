import type { PageLoad } from './$types';
import type { SettingsResponse } from '$lib/types/settings';
import { createApi, ApiError } from '$lib/utils/api';
import { redirect, error } from '@sveltejs/kit';

export const load: PageLoad = async ({ fetch }) => {
	try {
		const api = createApi(fetch);
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
