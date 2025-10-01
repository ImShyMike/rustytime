import { PUBLIC_BACKEND_API_URL } from '$env/static/public';
import type { RequestEvent } from '@sveltejs/kit';

// Use environment variable in production, fallback to localhost in development
const API_BASE = PUBLIC_BACKEND_API_URL || 'http://localhost:3000';

export class ServerApiError extends Error {
	status: number;

	constructor(message: string, status: number) {
		super(message);
		this.name = 'ServerApiError';
		this.status = status;
	}
}

class ServerApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = API_BASE) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(
		endpoint: string,
		event: RequestEvent,
		options: RequestInit = {}
	): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;

		// Forward cookies from the incoming request
		const cookieHeader = event.request.headers.get('cookie');
		console.error('Requesting', url, 'with cookies:', cookieHeader);

		const config: RequestInit = {
			credentials: 'include',
			mode: 'cors',
			headers: {
				'Content-Type': 'application/json',
				...(cookieHeader && { Cookie: cookieHeader }),
				...options.headers
			},
			...options
		};

		try {
			const response = await event.fetch(url, config);

			if (!response.ok) {
				throw new ServerApiError(
					`HTTP ${response.status}: ${response.statusText}`,
					response.status
				);
			}

			// Handle empty responses
			const contentType = response.headers.get('content-type');
			if (contentType && contentType.includes('application/json')) {
				return await response.json();
			}

			return response.text() as unknown as T;
		} catch (error) {
			if (error instanceof ServerApiError) {
				throw error;
			}
			throw new ServerApiError(`Network error: ${error}`, 0);
		}
	}

	// GET request
	async get<T>(endpoint: string, event: RequestEvent): Promise<T> {
		return this.request<T>(endpoint, event, { method: 'GET' });
	}

	// POST request
	async post<T>(endpoint: string, event: RequestEvent, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, event, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	// PUT request
	async put<T>(endpoint: string, event: RequestEvent, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, event, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	// DELETE request
	async delete<T>(endpoint: string, event: RequestEvent): Promise<T> {
		return this.request<T>(endpoint, event, { method: 'DELETE' });
	}
}

export const serverApi = new ServerApiClient();
