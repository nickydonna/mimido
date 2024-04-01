import { SESSION_KEY } from '$env/static/private';
import { getAuthUrl } from '$lib/server/google';
import { redirect } from '@sveltejs/kit';
import jwt from 'jsonwebtoken';

/** @type {import('./$types').PageServerLoad<{ token: string, googleUrl: string }>} */
export const load = async ({ locals }) => {
  // This doesn't seem safe ... but ok for no
  const token = jwt.sign(locals.session.data, SESSION_KEY);
  const googleUrl = getAuthUrl();
  return { token: token, googleUrl } 
}

/** @type {import('@sveltejs/kit').Actions} */
export const actions = {
  addCalendarView: async ({ request, locals }) => {

    const data = await request.formData();
    const calendarName = /** @type {string} */ (data.get('calendarName'));

    await locals.backend.check(calendarName);

    await locals.session.set({
      ...locals.session.data,
      // Replace for now
      calendars: [{
        provider: 'parent',
        type: 'extend',
        name: calendarName,
      }]
    })

    throw redirect(302, '/day')
  }
}
