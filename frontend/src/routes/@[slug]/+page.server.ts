import type { PageServerLoad } from './$types';
import type { ProfileResponse } from '$lib/types/profile';
import { createApi, ApiError } from '$lib/api/api';

export const load: PageServerLoad = async ({ fetch, params }) => {
	const loadProfile = async (): Promise<ProfileResponse | null> => {
		try {
			const api = createApi(fetch);
			return await api.get<ProfileResponse>(`/page/profile/${params.slug}`);
		} catch (e) {
			if (e instanceof ApiError && e.status === 404) {
				return null;
			}
			throw e;
		}
	};

	return {
		profile: await loadProfile()
	};
};
