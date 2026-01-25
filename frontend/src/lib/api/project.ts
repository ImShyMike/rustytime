import type { FullProjectListResponse, ProjectAliasesResponse } from '$lib/types/projects';
import type { Api } from './api';

export interface UpdateProjectRequest {
	project_url?: string;
	hidden?: boolean;
}

export async function updateProject(api: Api, id: number, updates: UpdateProjectRequest) {
	await api.put<void>(`/data/projects/${id}`, updates);
}

export async function getProjectAliases(api: Api) {
	return api.get<ProjectAliasesResponse>('/data/project_aliases');
}

export async function addProjectAlias(api: Api, projectId: number, aliasId: number) {
	await api.put<void>(`/data/project_aliases/${projectId}/${aliasId}`, {});
}

export async function deleteProjectAlias(api: Api, aliasId: number) {
	await api.delete<void>(`/data/project_aliases/${aliasId}`);
}

export async function getProjects(api: Api) {
	return api.get<FullProjectListResponse>('/data/projects');
}
