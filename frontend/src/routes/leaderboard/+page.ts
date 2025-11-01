import type { PageLoad } from './$types';
import type { LeaderboardResponse } from '$lib/types/leaderboard';
import { createApi, ApiError } from '$lib/utils/api';
import { redirect, error } from '@sveltejs/kit';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:leaderboard');

	try {
		const api = createApi(fetch);
		return await api.get<LeaderboardResponse>('/page/leaderboard');
	} catch (e) {
		console.error('Error loading leaderboard page data:', e);
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
