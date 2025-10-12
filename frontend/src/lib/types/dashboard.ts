export interface UsageStat {
	name: string;
	total_seconds: number;
	percent: number;
	text: string;
}

export interface DashboardResponse {
	auth_url: string | null;
	avatar_url: string;
	username: string;
	user_id: number;
	github_id: number;
	created_at: string;
	expires_at: string;
	api_key: string;
	total_heartbeats: number;
	human_readable_total: string;
	admin_level: number;
	dev_mode: boolean;
	projects: UsageStat[];
	editors: UsageStat[];
	operating_systems: UsageStat[];
	languages: UsageStat[];
}
