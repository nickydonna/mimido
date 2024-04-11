import { redirect } from '@sveltejs/kit';
import { getCognitoUIUrl } from '$lib/server/cognito/index.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.loggedIn) {
		throw redirect(302, getCognitoUIUrl());
	} else {
		throw redirect(303, '/day');
	}
};
