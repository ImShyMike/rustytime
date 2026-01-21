import type { Api } from './api';

export interface UpdateSettingsRequest {
	timezone?: string;
}

export interface UpdateSettingsResponse {
	success: boolean;
}

export async function updateSettings(
	api: Api,
	settings: UpdateSettingsRequest
): Promise<UpdateSettingsResponse> {
	return api.put<UpdateSettingsResponse>('/data/settings', settings);
}
