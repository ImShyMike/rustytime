import type { PageServerLoad } from './$types';
import type { ProjectsResponse } from '$lib/types/projects';
import { loadData } from '$lib/server/data';

export const load: PageServerLoad = async (event) => {
	event.depends('app:projects');
	return loadData<ProjectsResponse>('/page/projects', event);
};
