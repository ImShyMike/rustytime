export interface SettingsResponse {
	api_key?: string;
}

export interface ImportResponse {
	imported: number;
	processed: number;
	requests: number;
	start_date: string;
	time_taken: number;
}
