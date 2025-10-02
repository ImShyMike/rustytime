import type { ServerLoadEvent } from '@sveltejs/kit';

export const load = async ({ locals }: ServerLoadEvent) => {
	return {
		auth: locals.auth
	} satisfies App.PageData;
};
