import { env } from '$env/dynamic/public';

const DEFAULT_SITE_URL = 'https://rustytime.shymike.dev';

export function getSiteUrl(): string {
	const siteUrl = env.PUBLIC_SITE_URL ?? DEFAULT_SITE_URL;

	return siteUrl.replace(/\/+$/, '');
}
