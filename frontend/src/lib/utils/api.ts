import { PUBLIC_BACKEND_API_URL } from '$env/static/public';
import { redirect, error } from '@sveltejs/kit';

const API_BASE = PUBLIC_BACKEND_API_URL || 'http://localhost:3000';

export class ApiError extends Error {
	status: number;

	constructor(message: string, status: number) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

let globalErrorCallback: ((error: ApiError) => void) | null = null;

export function setGlobalErrorCallback(callback: (error: ApiError) => void) {
	globalErrorCallback = callback;
}

async function request<T>(
	endpoint: string,
	fetch: typeof globalThis.fetch,
	options: RequestInit = {}
): Promise<T> {
	const url = `${API_BASE}${endpoint}`;

	const config: RequestInit = {
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json',
			...options.headers
		},
		...options
	};

	try {
		const response = await fetch(url, config);

		if (!response.ok) {
			const apiError = new ApiError(
				`HTTP ${response.status}: ${response.statusText}`,
				response.status
			);

			if (globalErrorCallback) {
				globalErrorCallback(apiError);
			}

			if (response.status === 401 || response.status === 403) {
				throw redirect(302, '/?auth_error=unauthorized');
			}
			throw apiError;
		}

		const contentType = response.headers.get('content-type');
		if (contentType && contentType.includes('application/json')) {
			return await response.json();
		}

		return (await response.text()) as unknown as T;
	} catch (err) {
		if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
			throw err;
		}
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		const networkError = new ApiError(`Network error: ${err}`, 0);
		if (globalErrorCallback) {
			globalErrorCallback(networkError);
		}
		throw error(500, `Network error: ${err}`);
	}
}

export const api = {
	get: <T>(endpoint: string, fetch: typeof globalThis.fetch) =>
		request<T>(endpoint, fetch, { method: 'GET' }),

	post: <T>(endpoint: string, fetch: typeof globalThis.fetch, data?: unknown) =>
		request<T>(endpoint, fetch, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
		}),

	put: <T>(endpoint: string, fetch: typeof globalThis.fetch, data?: unknown) =>
		request<T>(endpoint, fetch, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined
		}),

	delete: <T>(endpoint: string, fetch: typeof globalThis.fetch) =>
		request<T>(endpoint, fetch, { method: 'DELETE' })
};
