import type { PageServerLoad } from './$types';
import { serverApi, ServerApiError } from '$lib/utils/serverApi';
import type { AdminResponse } from '$lib/types/admin';
import { redirect, error } from '@sveltejs/kit';

export const load: PageServerLoad = async (event) => {
	event.depends('app:admin');
	try {
		// Fetch admin data on the server
		const adminData = await serverApi.get<AdminResponse>('/admin', event);

		if (adminData && adminData.auth_url) {
			throw redirect(302, '/?auth_error=unauthorized');
		}

		return {
			adminData
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
		console.error('Unexpected error in admin load:', err);
		throw error(500, 'Internal server error');
	}
};
