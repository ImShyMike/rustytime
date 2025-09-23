import { type User } from '$lib/stores/auth';

interface AdminStats {
    total_users: number;
    total_heartbeats: number;
    heartbeats_last_hour: number;
    heartbeats_last_24h: number;
    requests_per_second: number;
    top_languages: Array<{ language: string; count: number }>;
    top_projects: Array<{ project: string; count: number }>;
    daily_activity: Array<{ date: string; count: number }>;
    all_users: Array<{
        id: number;
        name: string | null;
        avatar_url: string | null;
        created_at: string;
        api_key: string;
        github_id: number;
        is_admin: boolean;
    }>;
}

export interface AdminResponse {
    stats: AdminStats;
    current_user: User;
}