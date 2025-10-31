import type { PageLoad } from './$types';
import { api } from '$lib/utils/api';
import type { DashboardResponse } from '$lib/types/dashboard';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:dashboard');
	return await api.get<DashboardResponse>('/page/dashboard', fetch);
};
