// Make SPA
export const ssr = false;

/** @type {import('./$types').LayoutServerLoad} */
export const load = async ({ locals }) => {
  return {
		loggedIn: locals.loggedIn,
	};
}

