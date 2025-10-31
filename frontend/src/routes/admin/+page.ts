import type { PageLoad } from './$types';
import type { AdminResponse } from '$lib/types/admin';
import { api } from '$lib/utils/api';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:admin');
	return await api.get<AdminResponse>('/page/admin', fetch);
};
