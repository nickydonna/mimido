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
	console.log(token);

	const { payload, newToken } = await verifyToken(token.access_token);
	const accessToken = newToken ?? token.access_token;
	console.log(newToken);
	cookies.set('token', accessToken, {
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
