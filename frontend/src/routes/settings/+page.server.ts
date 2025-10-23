import type { PageServerLoad } from './$types';
import { serverApi, ServerApiError } from '$lib/utils/serverApi';
import type { SettingsResponse } from '$lib/types/settings';
import { redirect, error } from '@sveltejs/kit';

export const load: PageServerLoad = async (event) => {
	try {
		// Fetch settings data on the server
		const settingsData = await serverApi.get<SettingsResponse>('/page/settings', event);

		return {
			settingsData
		};
	} catch (err) {
		// Re-throw SvelteKit redirects
		if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
			throw err;
		}

		if (err instanceof ServerApiError) {
			// If unauthorized, redirect to home with auth error parameter
			if (err.status === 401 || err.status === 403) {
				throw redirect(302, '/?auth_error=unauthorized');
			}

			// For other API errors, throw a SvelteKit error
			throw error(err.status, err.message);
		}

		// For unexpected errors
		console.error('Unexpected error in dashboard load:', err);
		throw error(500, 'Internal server error');
	}
};
