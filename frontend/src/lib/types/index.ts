export interface User {
	id: number;
	github_id: number;
	name: string;
	avatar_url: string;
	api_key: string; // UUID string
	admin_level: number;
	is_banned: boolean;
	created_at: string; // ISO date string
	updated_at: string; // ISO date string
}

export interface PartialUser {
	id: number;
	github_id: number;
	name: string;
	avatar_url: string;
	api_key?: string; // UUID string
	admin_level: number;
	is_banned: boolean;
	created_at: string; // ISO date string
	updated_at: string; // ISO date string
}
