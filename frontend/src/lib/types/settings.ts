export interface SettingsResponse {
	api_key?: string;
}

export interface ImportStartResponse {
	job_id: number;
	status: string;
	message: string;
}

export interface ImportStatusResponse {
	job_id: number;
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

export interface ImportResponse {
	imported: number;
	processed: number;
	requests: number;
	start_date: string;
	time_taken: number;
}
