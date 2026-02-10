export interface UsageStat {
	name: string;
	total_seconds: number;
	percent: number;
	text: string;
}

export interface DashboardResponse {
	total_heartbeats: number;
	human_readable_total: string;
	range: string;
	projects: UsageStat[];
	editors: UsageStat[];
	operating_systems: UsageStat[];
	languages: UsageStat[];
}
