import { getTokenFromCode, verifyToken } from '$lib/server/cognito/index.js';
import { UserModel } from '$lib/server/db/index.js';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url, cookies }) => {
	const code = url.searchParams.get('code');
	if (!code) {
		throw redirect(302, '/');
	}

	const token = await getTokenFromCode(code);

	const { payload } = await verifyToken(token.access_token, token.refresh_token);
	cookies.set('token', token.access_token, {
		path: '/'
	});
	cookies.set('refresh_token', token.refresh_token, {
		path: '/'
	});

	const results = await UserModel.query('username').eq(payload.username).exec();
	if (results.length === 0) {
		await UserModel.create({ username: payload.username, auth: 'basic' });
	}
	throw redirect(302, '/account');
};