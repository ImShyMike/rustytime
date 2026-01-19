import type { PageServerLoad } from './$types';
import type { AdminResponse } from '$lib/types/admin';
import { createApi, ApiError } from '$lib/api/api';
import { redirect, error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ fetch, depends, request, url }) => {
	depends('app:admin');

	const limit = parseInt(url.searchParams.get('limit') || '25', 10);
	const offset = parseInt(url.searchParams.get('offset') || '0', 10);

	try {
		const cookieHeader = request.headers.get('cookie') || undefined;
		const api = createApi(fetch, cookieHeader);
		return await api.get<AdminResponse>(`/page/admin?limit=${limit}&offset=${offset}`);
	} catch (e) {
		console.error('Error loading admin page data:', e);
		const err = e as ApiError;
		if (err.status === 401 || err.status === 403) {
			throw redirect(302, '/?auth_error=unauthorized');
		}
		throw error(err.status || 500, err.message);
	}
};
