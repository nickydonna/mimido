import { getBackend } from '$lib/server/calendar/index.js';
import { refreshToken, verifyToken } from '$lib/server/cognito/index.js';
import { type User, UserModel } from '$lib/server/db/index.js';
import { redirect } from '@sveltejs/kit';
import { LRUCache } from 'lru-cache';

const options = {
	max: 100,
	// return stale items before removing from cache?
	allowStale: false,
}

const loginCache = new LRUCache<string, User>(options)

const unProtectedRoutes = ['/', '/cognito', '/sign-in', '/sign-up'];

export const handle: import('@sveltejs/kit').Handle = async ({ resolve, event }) => {
	event.locals.loggedIn = false;
	event.locals.loginCache = loginCache;
	let token = event.cookies.get('token');
	const refresh = event.cookies.get('refresh_token');

	if (!token && refresh) {
		const res = await refreshToken(refresh);
		token = res.access_token;
		event.cookies.set('token', token, { path: '/' })
	}

	if (!token) {
		if (!unProtectedRoutes.includes(event.url.pathname)) {
			throw redirect(303, '/');
		}
		return resolve(event);
	}

	const { payload, newToken } = await verifyToken(token, refresh);
	if (newToken) {
		event.cookies.set('token', newToken, { path: '/' });
	}
	event.locals.loggedIn = true;

	let user = loginCache.get(payload.username);
	if (!user) {
		user = await UserModel.get({ username: payload.username });
	}

	event.locals.user = user;
	if (user.main) {
		event.locals.backend = await getBackend(user);
	}

	return resolve(event);
};
