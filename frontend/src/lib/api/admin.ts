import { Api } from '$lib/api/api';

export async function impersonateUser(api: Api, userId: number) {
	await api.get(`/admin/impersonate/${userId}`);
	await window.cookieStore.delete({ name: 'rustytime_auth_snapshot', path: '/' });
	window.location.href = '/';
}

export async function changeAdminLevel(api: Api, userId: number, targetLevel: number) {
	if (targetLevel < 0) {
		return;
	}

	await api.put(`/admin/admin_level/${userId}/${targetLevel}`);
}
