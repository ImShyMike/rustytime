import type { Handle } from '@sveltejs/kit';
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
	impersonation?: {
		admin_id: number;
		admin_name: string | null;
		admin_avatar_url: string | null;
	} | null;
}

const DEFAULT_AUTH_STATE: App.Locals['auth'] = {
	isAuthenticated: false,
	sessionId: null,
	user: null,
	impersonation: null
};

export const handle: Handle = async ({ event, resolve }) => {
	const sessionId = event.cookies.get('rustytime_session');

	event.locals.auth = {
		...DEFAULT_AUTH_STATE
	};

	if (sessionId) {
		try {
			const response = await event.fetch(
				`${BACKEND_API_URL}/auth/github/verify?session_id=${sessionId}`
			);

			if (response.ok) {
				const data = (await response.json()) as RawVerifyResponse;

				if (data.valid && data.user) {
					const name = data.user.username ?? data.user.name ?? null;

					event.locals.auth = {
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
			}
		} catch (error) {
			console.error('SSR auth verification failed', error);
		}
	}

	return resolve(event, {
		filterSerializedResponseHeaders: (name) => name === 'content-type'
	});
};
