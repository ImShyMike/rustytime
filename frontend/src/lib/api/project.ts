import type { Api } from './api';

export async function setRepoUrl(api: Api, id: number, url: string | null) {
	await api.post<void>(`/data/projects/${id}/repo`, { repo_url: url });
}
