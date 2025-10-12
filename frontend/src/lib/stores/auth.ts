import { browser } from '$app/environment';
import { invalidateAll } from '$app/navigation';
import { writable } from 'svelte/store';
import { api, ApiError, setGlobalErrorCallback } from '../utils/api.js';
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

// Create auth store
const createAuthStore = () => {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		sessionId: null,
		isAuthenticated: false,
		isLoading: true,
		error: null,
		impersonation: null
	});

	// Helper function to create auth errors
	const createAuthError = (type: AuthErrorType, message: string): AuthError => ({
		type,
		message,
		timestamp: new Date()
	});

	return {
		subscribe,
		set,
		update,

		// Login by redirecting to GitHub OAuth
		login: async () => {
			if (!browser) return;

			try {
				const data = await api.get<{ auth_url: string }>('/auth/github/login');

				if (data.auth_url) {
					window.location.href = data.auth_url;
				}
			} catch (error) {
				console.error('Login error:', error);
			}
		},

		// Logout
		logout: async () => {
			if (!browser) return;

			try {
				await api.get('/auth/github/logout');

				set(toState(DEFAULT_AUTH_SNAPSHOT));
				await invalidateAll();
			} catch (error) {
				console.error('Logout error:', error);
			}
		},

		hydrate: (snapshot?: AuthSnapshot) => {
			const next = snapshot ? toState(snapshot) : toState(DEFAULT_AUTH_SNAPSHOT);
			set(next);
		},

		// Clear any existing error
		clearError: () => {
			update((state) => ({ ...state, error: null }));
		},

		// Set a specific error
		setError: (type: AuthErrorType, message: string) => {
			if (!browser) return;

			update((state) => ({
				...state,
				error: createAuthError(type, message)
			}));
		},

		// Retry session verification by reloading server-derived auth state
		retryVerification: async () => {
			if (!browser) return;

			update((state) => ({ ...state, isLoading: true, error: null }));

			try {
				await invalidateAll();
			} catch (error) {
				console.error('Auth retry failed:', error);
			} finally {
				update((state) => ({ ...state, isLoading: false }));
			}
		}
	};
};

export const auth = createAuthStore();

// Set up global API error handling
if (browser) {
	setGlobalErrorCallback((error: ApiError) => {
		// Only show errors for network/server issues, not auth issues
		if (error.status === 0 || error.status >= 500) {
			auth.update((state) => {
				// Don't overwrite existing errors
				if (!state.error) {
					const authError =
						error.status === 0
							? {
									type: 'network' as const,
									message: 'Unable to connect to server. Please check your connection.',
									timestamp: new Date()
								}
							: {
									type: 'server' as const,
									message: `Server error: ${error.message}`,
									timestamp: new Date()
								};

					return { ...state, error: authError };
				}
				return state;
			});
		}
	});
}
