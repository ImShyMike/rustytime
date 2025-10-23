import type { PageServerLoad } from './$types';
import type { AdminResponse } from '$lib/types/admin';
import { loadData } from '$lib/server/data';

export const load: PageServerLoad = async (event) => {
	event.depends('app:admin');
	return loadData<AdminResponse>('/page/admin', event);
};
