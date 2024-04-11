import type { LayoutServerLoad } from './$types';
// Make SPA
export const ssr = false;

export const load: LayoutServerLoad = async ({ locals }) => {
	return {
		loggedIn: locals.loggedIn
	};
};
