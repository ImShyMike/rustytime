export interface Project {
	id: number;
	name: string;
	total_seconds: number;
	project_url: string | null;
	hidden: boolean;
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

export interface AliasRecord {
	id: number;
	project_id: number;
}

export interface ProjectAliasEntry {
	project_id: number;
	aliases: AliasRecord[];
}

export interface ProjectAliasesResponse {
	aliases: ProjectAliasEntry[];
}

export interface SimpleProject {
	id: number;
	name: string;
	project_url: string | null;
	hidden: boolean;
}

export interface FullProjectListResponse {
	projects: SimpleProject[];
}
