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

export interface ProjectsListResponse {
	projects: Array<{
		id: number;
		name: string;
	}>;
}

export interface ProjectAliasEntry {
	project_id: number;
	aliases: number[];
}

export interface ProjectAliasesResponse {
	aliases: ProjectAliasEntry[];
}

export interface SetProjectRepoRequest {
	repo_url: string | null;
}
