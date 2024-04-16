import { getBackend } from '$lib/server/calendar/index.js';
import { PUBLIC_COGNITO_CLIENT_ID } from '$env/static/public';
import { verifyToken } from '$lib/server/cognito/index.js';
import { type User, UserModel } from '$lib/server/db/index.js';
import { redirect } from '@sveltejs/kit';
import { LRUCache } from 'lru-cache';

const options = {
	max: 100,
	// return stale items before removing from cache?
	allowStale: false
};

const loginCache = new LRUCache<string, User>(options);

const unProtectedRoutes = ['/', '/cognito', '/sign-in', '/sign-up'];

export const handle: import('@sveltejs/kit').Handle = async ({ resolve, event }) => {
	event.locals.loggedIn = false;
	event.locals.loginCache = loginCache;
	const cookies = event.cookies.getAll()
	const token = cookies.find(({ name }) =>
		name.startsWith(`CognitoIdentityServiceProvider.${PUBLIC_COGNITO_CLIENT_ID}`)
		&& name.endsWith('accessToken')
	)?.value

	if (!token) {
		if (!unProtectedRoutes.includes(event.url.pathname)) {
			throw redirect(303, '/');
		}
		return resolve(event);
	}

	const { payload, newToken } = await verifyToken(token);
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
