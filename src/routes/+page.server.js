import jwt from 'jsonwebtoken';
import { Backend } from '$lib/server/calendar';
import { dev } from '$app/environment';
import { error, redirect } from '@sveltejs/kit';


import { SESSION_KEY } from '$env/static/private';

/** @type {import('./$types').PageServerLoad} */
export const load = async ({ cookies }) => {
  if (cookies.get('session')) {
    throw redirect(303, '/day');
  }
  return;
}

/** @type {import('./$types').Actions} */
export const actions = {
  default: async({ request, cookies }) => {
		const data = await request.formData();
		const email = /** @type {string} */ (data.get('email'));
		const password = /** @type {string} */ (data.get('password'));
		const server = /** @type {string} */ (data.get('server'));
    const calendar = /** @type {string} */ (data.get('calendar'));
    const user = { email, password, server, calendar};
    
    const back = new Backend(user);
    try {
      await back.check();
      const token = jwt.sign(user, SESSION_KEY);
      cookies.set('session', token, {
        path: '/',
        httpOnly: true,
        sameSite: 'strict',
        secure: !dev 
      })
    } catch (e) {
      return error(500, e instanceof Error ? e.message : "")
    }
    throw redirect(303, '/day');
	}
};