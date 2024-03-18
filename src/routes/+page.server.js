import jwt from 'jsonwebtoken';
import { Backend } from '$lib/server/calendar';
import { dev } from '$app/environment';
import { redirect } from '@sveltejs/kit';

const key = process.env.SESSION_KEY ?? 'test';

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
      await back.test();
      const token = jwt.sign(user, key);
      cookies.set('session', token, {
        path: '/',
        httpOnly: true,
        sameSite: 'strict',
        secure: !dev 
      })
      throw redirect(303, '/day');
    } catch (e) {
      return {error: e }
    }
	}
};