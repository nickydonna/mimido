import jwt from 'jsonwebtoken';
import { CalendarBackend } from '$lib/server/calendar';
import { error, redirect } from '@sveltejs/kit';


import { SESSION_KEY } from '$env/static/private';

/** @type {import('./$types').PageServerLoad} */
export const load = async ({ locals }) => {
  if (locals.session.data.user) {
    throw redirect(303, '/day');
  }
  return;
}

/** @type {import('./$types').Actions} */
export const actions = {
  login: async({ request, locals }) => {
		const data = await request.formData();
		const email = /** @type {string} */ (data.get('email'));
		const password = /** @type {string} */ (data.get('password'));
		const server = /** @type {string} */ (data.get('server'));
    const calendar = /** @type {string} */ (data.get('calendar'));
    const auth = { email, password, server, calendar, type: 'basic' }
    // @ts-expect-error fix type
    const back = new CalendarBackend(auth);
    try {
      await back.check();
      await locals.session.set({ user: auth, calendars: [] })
    } catch (e) {
      console.log(e);
      return error(500, e instanceof Error ? e.message : "")
    }
    throw redirect(303, '/day'); 
  },
  import: async ({ request, locals }) => {
    const data = await request.formData();
		const token = /** @type {string} */ (data.get('token'));

    const payload = /** @type {typeof locals.session.data} */ (jwt.verify(token, SESSION_KEY));
    console.log(payload)
    
    const back = new CalendarBackend({ type: 'basic', ...payload.user });
    try {
      await back.check();
      await locals.session.set(payload)
 
    } catch (e) {
      console.log(e);
      return error(500, e instanceof Error ? e.message : "")
    }
    throw redirect(303, '/day');  }
};