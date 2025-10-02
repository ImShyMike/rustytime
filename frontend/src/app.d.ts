// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		interface AuthUser {
			id: number;
			github_id: number;
			name: string | null;
			avatar_url: string | null;
			is_admin: boolean;
		}

		interface Locals {
			auth: {
				isAuthenticated: boolean;
				sessionId: string | null;
				user: AuthUser | null;
			};
		}

		interface PageData {
			auth: Locals['auth'];
		}
		// interface Error {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
