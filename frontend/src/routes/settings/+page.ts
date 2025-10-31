import type { PageLoad } from './$types';
import type { SettingsResponse } from '$lib/types/settings';
import { api } from '$lib/utils/api';

export const load: PageLoad = async ({ fetch }) => {
	const data = await api.get<SettingsResponse>('/page/settings', fetch);
	return data;
};
