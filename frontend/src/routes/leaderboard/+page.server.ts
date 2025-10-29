import type { PageServerLoad } from './$types';
import type { LeaderboardResponse } from '$lib/types/leaderboard';
import { loadData } from '$lib/server/data';

export const load: PageServerLoad = async (event) => {
	event.depends('app:leaderboard');
	return await loadData<LeaderboardResponse>('/page/leaderboard', event);
};
