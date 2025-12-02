import type { ImportResponse } from '$lib/types/settings';
import type { Api } from './api';

export async function importFromHackatime(api: Api, api_key: string) {
    return api.post<ImportResponse>('/data/import?api_key=' + encodeURIComponent(api_key), {});
}
