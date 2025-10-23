import { serverApi, ServerApiError } from '$lib/utils/serverApi';
import { redirect, error, type ServerLoadEvent } from '@sveltejs/kit';

export async function loadData<T>(url: string, event: ServerLoadEvent): Promise<T> {
	try {
		// Fetch data on the server
		const data = await serverApi.get<T>(url, event);

		return data;
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
		console.error(`Unexpected error in "${url}" load:`, err);
		throw error(500, 'Internal server error');
	}
}
