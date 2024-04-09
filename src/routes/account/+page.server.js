import { error, redirect } from '@sveltejs/kit';
import { CalendarBackend } from '$lib/server/calendar';

/** @type {import('./$types').PageServerLoad<{ calendars: import('../../app').ExtendCalendarAccess[]  }>} */
export const load = async ({ locals }) => {
  return { calendars: locals.calendars } 
}

/** @type {import('@sveltejs/kit').Actions} */
export const actions = {
  setCalendar: async({ request, locals }) => {
		const data = await request.formData();
		const email = /** @type {string} */ (data.get('email'));
		const password = /** @type {string} */ (data.get('password'));
		const server = /** @type {string} */ (data.get('server'));
    const calendar = /** @type {string} */ (data.get('calendar'));
    const auth = { email, password, server, calendar}
    const back = new CalendarBackend(auth);
    try {
      await back.check();
      locals.user.main = auth;
      await locals.user.save()
    } catch (e) {
      return error(500, e instanceof Error ? e.message : "")
    }
    throw redirect(303, '/day'); 
  },
  addCalendarView: async ({ request, locals }) => {

    const data = await request.formData();
    const calendarName = /** @type {string} */ (data.get('calendarName'));

    await locals.backend.check(calendarName);

    // await locals.session.set({
    //   ...locals.session.data,
    //   // Replace for now
    //   calendars: [{
    //     provider: 'parent',
    //     type: 'extend',
    //     name: calendarName,
    //   }]
    // })

    throw redirect(302, '/day')
  }
}
