import { getSiteUrl } from '$lib/utils/site';
import type { RequestHandler } from './$types';

const SITE_URL = getSiteUrl();
const CACHE_CONTROL_HEADER = 'max-age=0, s-maxage=3600';
const BODY = `User-agent: *\nDisallow:\n\nSitemap: ${SITE_URL}/sitemap.xml\n`;

export const prerender = true;

export const GET: RequestHandler = async () => {
	return new Response(BODY, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'Cache-Control': CACHE_CONTROL_HEADER
		}
	});
};
