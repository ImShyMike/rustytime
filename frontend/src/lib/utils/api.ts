import { PUBLIC_BACKEND_API_URL } from '$env/static/public';

export const API_BASE = PUBLIC_BACKEND_API_URL || 'http://localhost:3000';

export class ApiError extends Error {
	status: number;
	body?: unknown;

	constructor(message: string, status: number, body?: unknown) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
		this.body = body;
	}
}

export function createApi(fetchFn: typeof globalThis.fetch) {
	async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
		const url = endpoint.startsWith('http') ? endpoint : `${API_BASE}${endpoint}`;
		const res = await fetchFn(url, {
			credentials: 'include',
			headers: { 'Content-Type': 'application/json', ...(options.headers || {}) },
			...options
		});

		const contentType = res.headers.get('content-type') || '';
		const parseJson = async () =>
			contentType.includes('application/json') ? await res.json() : undefined;

		if (!res.ok) {
			const body = await parseJson().catch(() => undefined);
			throw new ApiError(`HTTP ${res.status} ${res.statusText}`, res.status, body);
		}

		if (contentType.includes('application/json')) {
			return (await res.json()) as T;
		}

		return (await res.text()) as unknown as T;
	}

	return {
		get: <T>(endpoint: string) => request<T>(endpoint, { method: 'GET' }),
		post: <T>(endpoint: string, data?: unknown) =>
			request<T>(endpoint, { method: 'POST', body: data ? JSON.stringify(data) : undefined }),
		put: <T>(endpoint: string, data?: unknown) =>
			request<T>(endpoint, { method: 'PUT', body: data ? JSON.stringify(data) : undefined }),
		delete: <T>(endpoint: string) => request<T>(endpoint, { method: 'DELETE' })
	};
}
