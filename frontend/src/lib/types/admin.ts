import type { PartialUser } from '.';

export interface AdminResponse {
	total_users: number;
	total_heartbeats: number;
	heartbeats_last_hour: number;
	heartbeats_last_24h: number;
	requests_per_second: number;
	daily_activity: Array<{ date: string; count: number }>;
	all_users: Array<PartialUser>;
}
