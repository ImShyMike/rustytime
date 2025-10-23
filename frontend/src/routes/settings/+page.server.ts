import type { PageServerLoad } from './$types';
import type { SettingsResponse } from '$lib/types/settings';
import { loadData } from '$lib/server/data';

export const load: PageServerLoad = async (event) => {
	return loadData<SettingsResponse>('/page/settings', event);
};
