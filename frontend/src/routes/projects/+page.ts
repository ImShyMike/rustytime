import type { PageLoad } from './$types';
import type { ProjectsResponse } from '$lib/types/projects';
import { createApi, ApiError } from '$lib/utils/api';
import { redirect, error } from '@sveltejs/kit';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:projects');

	try {
		const api = createApi(fetch);
		return await api.get<ProjectsResponse>('/page/projects');
	} catch (e) {
		console.error('Error loading projects page data:', e);
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
