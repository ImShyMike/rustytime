import type { Handle } from '@sveltejs/kit';
import { PUBLIC_BACKEND_API_URL } from '$env/static/public';
import { dev } from '$app/environment';

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

// cookie used to cache the verified auth snapshot
const AUTH_SNAPSHOT_COOKIE = 'rustytime_auth_snapshot';

interface AuthSnapshot {
	sessionId: string;
	user: {
		id: number;
		github_id: number;
		name: string | null;
		avatar_url: string | null;
		admin_level: number;
	} | null;
	impersonation: App.ImpersonationInfo | null;
}

export const handle: Handle = async ({ event, resolve }) => {
	const sessionId = event.cookies.get('rustytime_session');

	event.locals.auth = { ...DEFAULT_AUTH_STATE };

	if (!sessionId) {
		const hasSnapshot = event.cookies.get(AUTH_SNAPSHOT_COOKIE);
		if (hasSnapshot) {
			try {
				event.cookies.delete(AUTH_SNAPSHOT_COOKIE, { path: '/' });
			} catch {
				event.cookies.set(AUTH_SNAPSHOT_COOKIE, '', { path: '/', maxAge: 0 });
			}
		}
		return resolve(event, {
			filterSerializedResponseHeaders: (name) => name === 'content-type'
		});
	}

	const snapshotRaw = event.cookies.get(AUTH_SNAPSHOT_COOKIE);
	if (snapshotRaw) {
		try {
			const snapshot = JSON.parse(snapshotRaw) as AuthSnapshot;
			if (snapshot.sessionId === sessionId) {
				event.locals.auth = {
					isAuthenticated: !!snapshot.user,
					sessionId: snapshot.sessionId,
					user: snapshot.user,
					impersonation: snapshot.impersonation ?? null
				};
				return resolve(event, {
					filterSerializedResponseHeaders: (name) => name === 'content-type'
				});
			}
			try {
				event.cookies.delete(AUTH_SNAPSHOT_COOKIE, { path: '/' });
			} catch {
				event.cookies.set(AUTH_SNAPSHOT_COOKIE, '', { path: '/', maxAge: 0 });
			}
		} catch {
			try {
				event.cookies.delete(AUTH_SNAPSHOT_COOKIE, { path: '/' });
			} catch {
				event.cookies.set(AUTH_SNAPSHOT_COOKIE, '', { path: '/', maxAge: 0 });
			}
		}
	}

	try {
		const response = await event.fetch(
			`${BACKEND_API_URL}/auth/github/verify?session_id=${sessionId}`,
			{
				credentials: 'include'
			}
		);

		if (response.ok) {
			const data = (await response.json()) as RawVerifyResponse;

			if (data.valid && data.user) {
				const name = data.user.username ?? data.user.name ?? null;

				const authState: App.Locals['auth'] = {
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

				event.locals.auth = authState;

				const snapshot: AuthSnapshot = {
					sessionId,
					user: authState.user,
					impersonation: authState.impersonation
				};
				event.cookies.set(AUTH_SNAPSHOT_COOKIE, JSON.stringify(snapshot), {
					path: '/',
					httpOnly: true,
					sameSite: 'lax',
					secure: !dev
				});
			} else {
				try {
					event.cookies.delete(AUTH_SNAPSHOT_COOKIE, { path: '/' });
				} catch {
					event.cookies.set(AUTH_SNAPSHOT_COOKIE, '', { path: '/', maxAge: 0 });
				}
			}
		} else {
			try {
				event.cookies.delete(AUTH_SNAPSHOT_COOKIE, { path: '/' });
			} catch {
				event.cookies.set(AUTH_SNAPSHOT_COOKIE, '', { path: '/', maxAge: 0 });
			}
		}
	} catch (error) {
		console.error('SSR auth verification failed:', error);
	}

	return resolve(event, {
		filterSerializedResponseHeaders: (name) => name === 'content-type'
	});
};
