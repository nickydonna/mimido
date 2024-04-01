// Make SPA
export const ssr = false;

/** @type {import('./$types').LayoutServerLoad} */
export const load = async ({ locals }) => {
  return {
		session: locals.session.data
	};
}

