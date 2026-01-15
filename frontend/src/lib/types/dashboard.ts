export interface UsageStat {
	name: string;
	total_seconds: number;
	percent: number;
	text: string;
}

export interface DashboardResponse {
	avatar_url: string;
	username: string;
	user_id: number;
	github_id: number;
	created_at: string;
	expires_at: string;
	total_heartbeats: number;
	human_readable_total: string;
	admin_level: number;
	dev_mode: boolean;
	range: string;
	projects: UsageStat[];
	editors: UsageStat[];
	operating_systems: UsageStat[];
	languages: UsageStat[];
}
