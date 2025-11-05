import type { FullProjectListResponse, ProjectAliasesResponse } from '$lib/types/projects';
import type { Api } from './api';

export async function setRepoUrl(api: Api, id: number, url: string | null) {
	await api.post<void>(`/data/projects/${id}/repo`, { repo_url: url });
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
