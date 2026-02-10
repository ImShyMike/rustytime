import type { PageServerLoad } from './$types';
import type { LeaderboardResponse } from '$lib/types/leaderboard';
import { createApi, ApiError } from '$lib/api/api';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, depends }) => {
	depends('app:leaderboard');

	try {
		const api = createApi(fetch);
		return await api.get<LeaderboardResponse>('/page/leaderboard');
	} catch (e) {
		console.error('Error loading leaderboard page data:', e);
		const err = e as ApiError;
		throw error(err.status || 500, err.message);
	}
};
