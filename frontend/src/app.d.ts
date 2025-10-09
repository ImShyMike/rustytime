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

		interface ImpersonationInfo {
			admin_id: number;
			admin_name: string | null;
			admin_avatar_url: string | null;
		}

		interface Locals {
			auth: {
				isAuthenticated: boolean;
				sessionId: string | null;
				user: AuthUser | null;
				impersonation: ImpersonationInfo | null;
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
