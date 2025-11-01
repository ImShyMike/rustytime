import { createApi } from '$lib/utils/api';

export async function impersonateUser(userId: number) {
	const api = createApi(fetch);
	await api.get(`/admin/impersonate/${userId}`);
	window.location.href = '/';
}

export async function changeAdminLevel(userId: number, targetLevel: number) {
	if (targetLevel < 0) {
		return;
	}

	const api = createApi(fetch);
	await api.get(`/admin/admin_level/${userId}/${targetLevel}`);
}
