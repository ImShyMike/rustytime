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

export type ProfileResponse = {
	username: string;
	avatar_url: string;
	projects: Project[];
	time: TimeInfo;
};
