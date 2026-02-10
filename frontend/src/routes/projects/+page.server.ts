import type { PageServerLoad } from './$types';
import { createApi, ApiError } from '$lib/api/api';
import { redirect, error } from '@sveltejs/kit';
import type { ProjectsResponse } from '$lib/types/projects';

export const load: PageServerLoad = async ({ fetch, depends, request }) => {
	depends('app:projects');

	const cookieHeader = request.headers.get('cookie') || undefined;

	const loadProjects = async (): Promise<ProjectsResponse> => {
		try {
			const api = createApi(fetch, cookieHeader);
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

	return {
		projects: loadProjects()
	};
};
