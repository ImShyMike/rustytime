export interface Project {
	id: number;
	name: string;
	total_seconds: number;
	repo_url: string | null;
	human_readable_total: string;
	created_at: string | null;
	updated_at: string | null;
}

export interface ProjectsResponse {
	projects: Project[];
}
