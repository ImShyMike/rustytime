import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import { api, ApiError, setGlobalErrorCallback } from '../utils/api.js';

export interface User {
	id: number;
	github_id: number;
	name: string | null;
	avatar_url: string | null;
	is_admin: boolean;
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
}

// Create auth store
const createAuthStore = () => {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		sessionId: null,
		isAuthenticated: false,
		isLoading: true,
		error: null
	});

	// Helper function to create auth errors
	const createAuthError = (type: AuthErrorType, message: string): AuthError => ({
		type,
		message,
		timestamp: new Date()
	});

	// Helper function to classify errors
	const classifyError = (error: unknown): AuthError => {
		if (error instanceof ApiError) {
			if (error.status === 0) {
				return createAuthError(
					'network',
					'Unable to connect to server. Please check your connection.'
				);
			} else if (error.status === 401 || error.status === 403) {
				return createAuthError('unauthorized', `Authentication failed: ${error.message}`);
			} else if (error.status >= 500) {
				return createAuthError('server', `Server error: ${error.message}`);
			} else {
				return createAuthError('unknown', `HTTP error: ${error.message}`);
			}
		} else if (error instanceof Error) {
			if (
				error.message.toLowerCase().includes('network') ||
				error.message.toLowerCase().includes('fetch') ||
				error.message.toLowerCase().includes('failed to fetch')
			) {
				return createAuthError(
					'network',
					'Unable to connect to server. Please check your connection.'
				);
			} else {
				return createAuthError('unknown', `Unexpected error: ${error.message}`);
			}
		} else {
			return createAuthError('unknown', 'An unknown error occurred');
		}
	};

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

				localStorage.removeItem('rustytime_session_id');

				set({
					user: null,
					sessionId: null,
					isAuthenticated: false,
					isLoading: false,
					error: null
				});
			} catch (error) {
				console.error('Logout error:', error);
			}
		},

		// Verify session using stored session ID
		verifySession: async () => {
			if (!browser) {
				update((state) => ({ ...state, isLoading: false }));
				return;
			}

			const sessionId = localStorage.getItem('rustytime_session_id');

			if (!sessionId) {
				update((state) => ({ ...state, isLoading: false }));
				return;
			}

			try {
				const data = await api.get<{
					valid: boolean;
					user?: {
						id: number;
						github_id: number;
						username?: string;
						name?: string;
						avatar_url: string | null;
						is_admin: boolean;
					};
				}>(`/auth/github/verify?session_id=${sessionId}`);

				if (data.valid && data.user) {
					const user: User = {
						id: data.user.id,
						github_id: data.user.github_id,
						name: data.user.username || data.user.name || null,
						avatar_url: data.user.avatar_url,
						is_admin: data.user.is_admin
					};

					set({
						user,
						sessionId,
						isAuthenticated: true,
						isLoading: false,
						error: null
					});
				} else {
					// Session is invalid, clear it
					localStorage.removeItem('rustytime_session_id');
					set({
						user: null,
						sessionId: null,
						isAuthenticated: false,
						isLoading: false,
						error: createAuthError('unauthorized', 'Session is invalid or expired')
					});
				}
			} catch (error) {
				console.error('Session verification error:', error);

				const authError = classifyError(error);

				// Only clear session for unauthorized errors
				if (authError.type === 'unauthorized') {
					localStorage.removeItem('rustytime_session_id');
					set({
						user: null,
						sessionId: null,
						isAuthenticated: false,
						isLoading: false,
						error: authError
					});
				} else {
					// For network/server errors, just update loading and error state
					update((state) => ({
						...state,
						isLoading: false,
						error: authError
					}));
				}
			}
		},

		// Set session after OAuth callback
		setSession: (user: User, sessionId: string) => {
			if (!browser) return;

			localStorage.setItem('rustytime_session_id', sessionId);
			set({
				user,
				sessionId,
				isAuthenticated: true,
				isLoading: false,
				error: null
			});
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

		// Retry session verification
		retryVerification: async () => {
			update((state) => ({ ...state, isLoading: true, error: null }));
			await auth.verifySession();
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

// Auto-verify session on load
if (browser) {
	auth.verifySession();
}
