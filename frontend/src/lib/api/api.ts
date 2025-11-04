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

export class Api {
	constructor(
		private readonly fetchFn: typeof globalThis.fetch,
		private readonly cookieHeader?: string
	) {}

	private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
		const url = endpoint.startsWith('http') ? endpoint : `${API_BASE}${endpoint}`;
		const headers: HeadersInit = {
			'Content-Type': 'application/json',
			...(options.headers || {})
		};

		if (this.cookieHeader) {
			// Forward cookies in SSR context
			(headers as Record<string, string>)['Cookie'] = this.cookieHeader;
		}

		const res = await this.fetchFn(url, {
			credentials: 'include',
			headers,
			...options
		});

		const contentType = res.headers.get('content-type') || '';
		const parseJson = async () =>
			contentType.includes('application/json') ? await res.json() : undefined;

		if (!res.ok) {
			let body: unknown;
			let errorMessage = `HTTP ${res.status} ${res.statusText}`;

			if (contentType.includes('application/json')) {
				body = await parseJson().catch(() => undefined);
			} else {
				const textBody = await res.text().catch(() => undefined);
				if (textBody) {
					body = textBody;
					errorMessage = textBody;
				}
			}

			throw new ApiError(errorMessage, res.status, body);
		}

		if (contentType.includes('application/json')) {
			return (await res.json()) as T;
		}

		return (await res.text()) as unknown as T;
	}

	get<T>(endpoint: string) {
		return this.request<T>(endpoint, { method: 'GET' });
	}

	post<T>(endpoint: string, data?: unknown) {
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	put<T>(endpoint: string, data?: unknown) {
		return this.request<T>(endpoint, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	delete<T>(endpoint: string) {
		return this.request<T>(endpoint, { method: 'DELETE' });
	}
}

export function createApi(fetchFn: typeof globalThis.fetch, cookieHeader?: string) {
	return new Api(fetchFn, cookieHeader);
}
