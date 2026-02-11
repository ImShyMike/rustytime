import { browser } from '$app/environment';
import { invalidateAll } from '$app/navigation';
import { writable } from 'svelte/store';
import { createApi } from '$lib/api/api';

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
	error: AuthError | null;
}

const createAuthStore = () => {
	const { subscribe, set, update } = writable<AuthState>({ error: null });

	const setError = (type: AuthErrorType, message: string = '') => {
		if (!browser) return;
		update((s) => ({ ...s, error: { type, message, timestamp: new Date() } }));
	};

	return {
		subscribe,

		setError: (type: AuthErrorType, message?: string) => setError(type, message),

		clearError: () => update((s) => ({ ...s, error: null })),

		clear: () => set({ error: null }),

		login: async () => {
			if (!browser) return;

			try {
				const api = createApi(fetch);
				const { auth_url } = await api.get<{ auth_url: string }>('/auth/github/login');
				if (auth_url) window.location.href = auth_url;
			} catch (e) {
				setError('server');
				console.log('Login error:', e);
			}
		},

		logout: async () => {
			if (!browser) return;

			try {
				const api = createApi(fetch);
				await api.get('/auth/github/logout');
			} catch (e) {
				console.log('Logout error:', e);
			} finally {
				set({ error: null });
				await invalidateAll();
			}
		}
	};
};

export const auth = createAuthStore();
