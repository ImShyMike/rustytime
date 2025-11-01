import { createApi } from '$lib/utils/api';

export async function impersonateUser(userId: number) {
	try {
		const api = createApi(fetch);
		await api.get(`/admin/impersonate/${userId}`);
		window.location.reload();
	} catch (error) {
		console.error('Error impersonating user:', error);
	}
}

export async function changeAdminLevel(userId: number, targetLevel: number) {
	if (targetLevel < 0) {
		return;
	}

	try {
		const api = createApi(fetch);
		await api.get(`/admin/admin_level/${userId}/${targetLevel}`);
		window.location.reload();
	} catch (error) {
		console.error('Failed to update admin level:', error);
	}
}
