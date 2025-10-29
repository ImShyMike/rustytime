export interface LeaderboardEntry {
	user_id: number;
	user_name: string;
	avatar_url: string;
	total_seconds: number;
	rank: number;
}

export interface Leaderboard {
	generated_at: string;
	entries: LeaderboardEntry[];
}

export interface LeaderboardResponse {
	daily: Leaderboard;
	weekly: Leaderboard;
	all_time: Leaderboard;
}
