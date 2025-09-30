import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { auth } from '$lib/stores/auth';
import { get } from 'svelte/store';
import type { User } from '$lib/stores/auth';

interface AuthEffectParams<T = unknown> {
	isAuthLoading: boolean;
	isAuthenticated: boolean;
	user: User | null;
	data: T | null;
	loading: boolean;
	error: string | null;
	loadData: () => void;
	requireAdmin?: boolean;
	redirectTo?: '/' | '/dashboard';
}

export function handleAuthEffect<T = unknown>(params: AuthEffectParams<T>) {
	const {
		isAuthLoading,
		isAuthenticated,
		user,
		data,
		loading,
		error,
		loadData,
		requireAdmin = false,
		redirectTo = '/'
	} = params;

	if (typeof window === 'undefined') return;
	if (isAuthLoading) return;

	if (!isAuthenticated) {
		try {
			const current = get(auth);
			if (!current.error || current.error.type !== 'unauthorized') {
				auth.update((state) => ({
					...state,
					error: {
						type: 'unauthorized',
						message: 'Please log in to do that.',
						timestamp: new Date()
					}
				}));
			}
		} catch {
			auth.update((state) => ({
				...state,
				error: {
					type: 'unauthorized',
					message: 'Please log in to do that.',
					timestamp: new Date()
				}
			}));
		}

		if (typeof window !== 'undefined') {
			const target = resolve(redirectTo);
			if (window.location.pathname !== target) {
				goto(target);
			}
		}

		return;
	}

	if (requireAdmin && user && !user.is_admin) {
		if (typeof window !== 'undefined') {
			const target = resolve('/dashboard');
			if (window.location.pathname !== target) {
				goto(target);
			}
		}
		return;
	}

	if (!data && !loading && !error) {
		loadData();
	}
}
