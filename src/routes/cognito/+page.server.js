import { getTokenFromCode, verifyToken } from '$lib/server/cognito';
import { UserModel } from '$lib/server/db';
import { redirect } from '@sveltejs/kit';

/** @type {import('./$types').PageServerLoad} */
export const load = async ({ url, cookies }) => {
  const code = url.searchParams.get('code');
  if (!code) {
    throw redirect(302, '/');
  }
 
  const token = await getTokenFromCode(code)
  cookies.set('token', token.access_token, {
    path: '/'
  });
  cookies.set('refresh_token', token.refresh_token, {
    path: '/'
  });
  const { payload } = await verifyToken(token.access_token, token.refresh_token);

  const results = await UserModel.query("username").eq(payload.username).exec()
  if (results.length === 0) {
    await UserModel.create({ username: payload.username, auth: 'basic' })
  } 
  throw redirect(302, '/account')
}