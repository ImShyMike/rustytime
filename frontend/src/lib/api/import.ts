import type { ImportStartResponse, ImportStatusResponse } from '$lib/types/settings';
import type { Api } from './api';

export async function startImport(api: Api, api_key: string): Promise<ImportStartResponse> {
	return api.post<ImportStartResponse>('/data/import?api_key=' + encodeURIComponent(api_key), {});
}

export async function getImportStatus(api: Api): Promise<ImportStatusResponse> {
	return api.get<ImportStatusResponse>('/data/import/status');
}
