import { error, redirect } from '@sveltejs/kit';
import { CalendarBackend } from '$lib/server/calendar';

/** @type {import('./$types').PageServerLoad} */
export const load = async ({ locals }) => {
  return { main: locals.user.main, calendars: locals.user.calendars } 
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
    const back = new CalendarBackend(locals.user.username, auth);
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
    const calendars = locals.user.calendars ?? [];
    locals.user.calendars = [
      ...calendars, {
        provider: 'parent',
        type: 'extend',
        name: calendarName,
      }]
    await locals.user.save();
    throw redirect(302, '/day')
  },
  resync: async ({ locals }) => {
    const { backend } = locals;
	  await backend.initialSync(true);
	  await Promise.all(locals.user.calendars.map(async c => 
			backend.initialSync(false, c.name)
	  ))
    return { ok: true }
  }
}
