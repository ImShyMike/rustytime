import type { ServerLoad } from '@sveltejs/kit';
import { PUBLIC_BACKEND_API_URL } from '$env/static/public';

const BACKEND_API_URL = PUBLIC_BACKEND_API_URL || 'http://localhost:3000';

interface RawVerifyResponse {
	valid: boolean;
	user?: {
		id: number;
		github_id: number;
		username?: string | null;
		name?: string | null;
		avatar_url: string | null;
		admin_level: number;
	};
	impersonation?: App.ImpersonationInfo | null;
}

const DEFAULT_AUTH_STATE: App.Locals['auth'] = {
	isAuthenticated: false,
	sessionId: null,
	user: null,
	impersonation: null
};

export const load: ServerLoad = async ({ fetch, cookies, locals }) => {
	const sessionId = cookies.get('rustytime_session');
	let authState: App.Locals['auth'] = { ...DEFAULT_AUTH_STATE };

	if (sessionId) {
		try {
			const response = await fetch(
				`${BACKEND_API_URL}/auth/github/verify?session_id=${encodeURIComponent(sessionId)}`,
				{ credentials: 'include' }
			);

			if (response.ok) {
				const data = (await response.json()) as RawVerifyResponse;

				if (data.valid && data.user) {
					const name = data.user.username ?? data.user.name ?? null;
					authState = {
						isAuthenticated: true,
						sessionId,
						user: {
							id: data.user.id,
							github_id: data.user.github_id,
							name,
							avatar_url: data.user.avatar_url,
							admin_level: data.user.admin_level
						},
						impersonation: data.impersonation ?? null
					};
				}
			} else {
				console.error('Auth verification failed with status:', response.status);
			}
		} catch (error) {
			console.error('Auth verification failed:', error);
		}
	}

	locals.auth = authState;

	return {
		auth: authState
	} satisfies App.PageData;
};
