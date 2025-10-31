import type { PageLoad } from './$types';
import type { ProjectsResponse } from '$lib/types/projects';
import { api } from '$lib/utils/api';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:projects');
	return await api.get<ProjectsResponse>('/page/projects', fetch);
};
