import { redirect } from '@sveltejs/kit';


import { getCognitoUIUrl } from '$lib/server/cognito';

/** @type {import('./$types').PageServerLoad} */
export const load = async ({ locals }) => {
  if (!locals.loggedIn) {
    throw redirect(302, getCognitoUIUrl())
  } else {
    throw redirect(303, '/day');
  }
}
