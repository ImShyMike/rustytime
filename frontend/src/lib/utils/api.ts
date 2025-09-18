import { browser } from '$app/environment';
import { dev } from '$app/environment';
import { PUBLIC_BACKEND_API_URL } from '$env/static/public';

// Use environment variable in production, fallback to localhost in development
const API_BASE = browser
	? dev
		? 'http://localhost:3000'
		: PUBLIC_BACKEND_API_URL || 'http://localhost:3000'
	: 'http://localhost:3000';

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

class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = API_BASE) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;

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
				throw new ApiError(`HTTP ${response.status}: ${response.statusText}`, response.status);
			}

			// Handle empty responses
			const contentType = response.headers.get('content-type');
			if (contentType && contentType.includes('application/json')) {
				return await response.json();
			}

			return response.text() as unknown as T;
		} catch (error) {
			let apiError: ApiError;
			if (error instanceof ApiError) {
				apiError = error;
			} else {
				apiError = new ApiError(`Network error: ${error}`, 0);
			}

			if (globalErrorCallback) {
				globalErrorCallback(apiError);
			}
			
			throw apiError;
		}
	}

	// GET request
	async get<T>(endpoint: string): Promise<T> {
		return this.request<T>(endpoint, { method: 'GET' });
	}

	// POST request
	async post<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	// PUT request
	async put<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	// DELETE request
	async delete<T>(endpoint: string): Promise<T> {
		return this.request<T>(endpoint, { method: 'DELETE' });
	}
}

export const api = new ApiClient();
