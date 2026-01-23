import { browser } from '$app/environment';
import { invalidateAll } from '$app/navigation';
import { writable } from 'svelte/store';
import { createApi, ApiError } from '$lib/api/api';

export interface User {
	id: number;
	github_id: number;
	name: string | null;
	avatar_url: string | null;
	admin_level: number;
}

export type AuthErrorType =
	| 'unauthorized' // 401/403 - session invalid, should clear
	| 'network' // Network connectivity issues
	| 'server' // 5xx server errors
	| 'unknown' // Other unexpected errors
	| null; // No error

export interface AuthError {
	type: AuthErrorType;
	message: string;
	timestamp: Date;
}

export interface AuthState {
	user: User | null;
	sessionId: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: AuthError | null;
	impersonation: App.ImpersonationInfo | null;
}

interface VerifyResponse {
	valid: boolean;
	user?: User & { username?: string | null };
	impersonation?: App.ImpersonationInfo | null;
}

type AuthSnapshot = App.Locals['auth'];

const DEFAULT_AUTH_SNAPSHOT: AuthSnapshot = {
	isAuthenticated: false,
	sessionId: null,
	user: null,
	impersonation: null
};

const toState = (snapshot: AuthSnapshot): AuthState => ({
	user: snapshot.user,
	sessionId: snapshot.sessionId,
	isAuthenticated: snapshot.isAuthenticated,
	isLoading: false,
	error: null,
	impersonation: snapshot.impersonation
});

const createAuthStore = () => {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		sessionId: null,
		isAuthenticated: false,
		isLoading: true,
		error: null,
		impersonation: null
	});

	const setError = (type: AuthErrorType, message: string) => {
		if (!browser) return;
		update((s) => ({ ...s, error: { type, message, timestamp: new Date() } }));
	};

	async function verify(fetchFn: typeof globalThis.fetch) {
		update((s) => ({ ...s, isLoading: true, error: null }));

		try {
			const api = createApi(fetchFn);
			const data = await api.get<VerifyResponse>('/auth/github/verify');

			if (data?.valid && data.user) {
				const name = data.user.name ?? data.user.username ?? null;
				set({
					user: { ...data.user, name },
					sessionId: null,
					isAuthenticated: true,
					isLoading: false,
					error: null,
					impersonation: data.impersonation ?? null
				});
			} else {
				set({
					user: null,
					sessionId: null,
					isAuthenticated: false,
					isLoading: false,
					error: null,
					impersonation: null
				});
			}
		} catch (e) {
			const err = e as ApiError;
			if (err.status === 400 || err.status === 401 || err.status === 403) {
				set({
					user: null,
					sessionId: null,
					isAuthenticated: false,
					isLoading: false,
					error: null,
					impersonation: null
				});
			} else if (err.status >= 500 || err.status === 0) {
				setError(
					err.status === 0 ? 'network' : 'server',
					err.status === 0
						? 'Unable to connect to server. Please check your connection.'
						: `Server error: ${err.message}`
				);
				update((s) => ({ ...s, isLoading: false }));
			} else {
				set({
					user: null,
					sessionId: null,
					isAuthenticated: false,
					isLoading: false,
					error: null,
					impersonation: null
				});
			}
		}
	}

	return {
		subscribe,

		hydrate: (snapshot?: AuthSnapshot) => {
			const next = snapshot ? toState(snapshot) : toState(DEFAULT_AUTH_SNAPSHOT);
			set(next);
		},

		clearError: () => update((s) => ({ ...s, error: null })),

		setError: (type: AuthErrorType, message: string) => setError(type, message),

		verify: async (fetchFn = fetch) => {
			if (browser) await verify(fetchFn);
		},

		login: async () => {
			if (!browser) return;

			try {
				const api = createApi(fetch);
				const { auth_url } = await api.get<{ auth_url: string }>('/auth/github/login');
				if (auth_url) window.location.href = auth_url;
			} catch (e) {
				console.log('Login error:', e);
				setError('unknown', 'Login failed. Please try again. Please check https://stats.uptimerobot.com/S3OiEoxnNo for status updates');
			}
		},

		clear: () => {
			set({
				user: null,
				sessionId: null,
				isAuthenticated: false,
				isLoading: false,
				impersonation: null,
				error: null
			});
		},

		logout: async () => {
			if (!browser) return;

			try {
				const api = createApi(fetch);
				await api.get('/auth/github/logout');
			} catch (e) {
				console.log('Logout error:', e);
			} finally {
				set({
					user: null,
					sessionId: null,
					isAuthenticated: false,
					isLoading: false,
					error: null,
					impersonation: null
				});
				await invalidateAll();
			}
		}
	};
};

export const auth = createAuthStore();
