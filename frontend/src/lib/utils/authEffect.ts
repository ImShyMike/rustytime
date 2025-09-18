import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
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
		goto(resolve(redirectTo));
		return;
	}

	if (requireAdmin && user && !user.is_admin) {
		goto(resolve('/dashboard'));
		return;
	}

	if (!data && !loading && !error) {
		loadData();
	}
}
