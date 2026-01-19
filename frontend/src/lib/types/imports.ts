export interface ImportJobWithUser {
	id: number;
	user_id: number;
	user_name: string | null;
	user_avatar_url: string | null;
	status: string;
	imported_count: number | null;
	processed_count: number | null;
	request_count: number | null;
	start_date: string | null;
	time_taken: number | null;
	error_message: string | null;
	created_at: string;
	updated_at: string;
}

export interface AdminImportsResponse {
	imports: ImportJobWithUser[];
	total: number;
	limit: number;
	offset: number;
}
