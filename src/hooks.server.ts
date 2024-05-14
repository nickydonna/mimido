import { getBackend } from '$lib/server/calendar/index.js';
import { redirect } from '@sveltejs/kit';
import { LRUCache } from 'lru-cache';
import jwt from 'jsonwebtoken'
import { SUDO_PASSWORD } from '$env/static/private';

const unProtectedRoutes = ['/', '/create', '/sign-in', '/sign-up'];

export const handle: import('@sveltejs/kit').Handle = async ({ resolve, event }) => {
	const token = event.cookies.get('session');
	let user;
	try {
		user = jwt.verify(token ?? '', SUDO_PASSWORD)
	} catch (e) { /* noop */ }

	if (!token || !user) {
		if (!unProtectedRoutes.includes(event.url.pathname)) {
			throw redirect(303, '/');
		}
		return resolve(event);
	}

	event.locals.loggedIn = true;

	event.locals.user = user as { email: string, id: number };
	// if (user.main) {
	// event.locals.backend = await getBackend(user);
	// }
	return resolve(event);
};
