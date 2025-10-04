import { getSiteUrl } from '$lib/utils/site';
import type { RequestHandler } from './$types';

type SitemapEntry = {
	path: string;
	changefreq?: string;
	priority?: string;
	lastmod?: string;
};

const SITE_URL = getSiteUrl();
const CACHE_CONTROL_HEADER = 'max-age=0, s-maxage=3600';
const DEFAULT_CHANGE_FREQUENCY = 'daily';
const DEFAULT_PRIORITY = '0.5';

const PAGE_ENTRIES: Record<string, Omit<SitemapEntry, 'path'>> = {
	'/': {
		changefreq: 'daily',
		priority: '1.0'
	}
};

const generatedAt = new Date().toISOString();
const sitemapXml = createSitemap(makeEntries(PAGE_ENTRIES, generatedAt));

export const prerender = true;

export const GET: RequestHandler = async () => {
	return new Response(sitemapXml, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': CACHE_CONTROL_HEADER
		}
	});
};

function makeEntries(
	pages: Record<string, Omit<SitemapEntry, 'path'>>,
	generatedAtIso: string
): SitemapEntry[] {
	return Object.entries(pages)
		.map(([path, config]) => ({
			path,
			changefreq: config.changefreq ?? DEFAULT_CHANGE_FREQUENCY,
			priority: config.priority ?? DEFAULT_PRIORITY,
			lastmod: config.lastmod ?? generatedAtIso
		}))
		.sort((a, b) => {
			if (a.path === '/') return -1;
			if (b.path === '/') return 1;
			return a.path.localeCompare(b.path);
		});
}

function createSitemap(entries: SitemapEntry[]): string {
	const urls = entries.map(formatUrlEntry).join('\n');

	return `<?xml version="1.0" encoding="UTF-8" ?>\n<urlset\n  xmlns="https://www.sitemaps.org/schemas/sitemap/0.9"\n  xmlns:news="https://www.google.com/schemas/sitemap-news/0.9"\n  xmlns:xhtml="https://www.w3.org/1999/xhtml"\n  xmlns:mobile="https://www.google.com/schemas/sitemap-mobile/1.0"\n  xmlns:image="https://www.google.com/schemas/sitemap-image/1.1"\n  xmlns:video="https://www.google.com/schemas/sitemap-video/1.1"\n>\n${urls}\n</urlset>`;
}

function formatUrlEntry({ path, changefreq, priority, lastmod }: SitemapEntry): string {
	const loc = path === '/' ? SITE_URL : `${SITE_URL}${path}`;

	return `  <url>\n    <loc>${loc}</loc>\n    <lastmod>${lastmod}</lastmod>\n    <changefreq>${changefreq}</changefreq>\n    <priority>${priority}</priority>\n  </url>`;
}

