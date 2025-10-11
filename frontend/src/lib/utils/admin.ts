import { PUBLIC_BACKEND_API_URL } from '$env/static/public';

export async function impersonateUser(userId: number) {
	try {
		await fetch(`${PUBLIC_BACKEND_API_URL}/admin/impersonate/${userId}`, {
			method: 'GET',
			credentials: 'include',
			redirect: 'manual'
		});

		window.location.reload();
	} catch (error) {
		console.error('Error impersonating user:', error);
	}
}
