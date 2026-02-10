export type TimeInfo = {
	today: number;
	week: number;
	all_time: number;
};

export type Project = {
	name: string;
	project_url: string | null;
	total_seconds: number;
};

export type ProfileUser = {
	username: string;
	avatar_url: string;
	admin_level: number;
};

export type ProfileResponse = {
	user: ProfileUser;
	projects: Project[];
	time: TimeInfo;
};
