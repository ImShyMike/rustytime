import type { PageLoad } from './$types';
import type { LeaderboardResponse } from '$lib/types/leaderboard';
import { api } from '$lib/utils/api';

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('app:leaderboard');
	return await api.get<LeaderboardResponse>('/page/leaderboard', fetch);
};
