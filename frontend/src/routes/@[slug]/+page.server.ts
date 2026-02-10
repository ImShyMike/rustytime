import type { PageServerLoad } from './$types';
import type { ProfileResponse } from '$lib/types/profile';
import { createApi, ApiError } from '$lib/api/api';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, params }) => {
	try {
		const api = createApi(fetch);
		return await api.get<ProfileResponse>(`/page/profile/${params.slug}`);
	} catch (e) {
		console.error('Error loading leaderboard page data:', e);
		const err = e as ApiError;
		throw error(err.status || 500, err.message);
	}
};
