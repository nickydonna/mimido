import { error, redirect } from '@sveltejs/kit';
import { CalendarBackend } from '$lib/server/calendar/index.js';
import type { PageServerLoad, Actions } from './$types';

/** @type {import('./$types').PageServerLoad} */
export const load: PageServerLoad = async ({ locals }) => {
	return { main: locals.user.main, calendars: locals.user.calendars };
};

export const actions: Actions = {
	setCalendar: async ({ request, locals }) => {
		const data = await request.formData();
		const email = data.get('email') as string;
		const password = data.get('password') as string;
		const server = data.get('server') as string;
		const calendar = data.get('calendar') as string;
		const auth = { email, password, server, calendar };
		const back = new CalendarBackend(locals.user.username, auth);
		try {
			await back.check();
			locals.user.main = auth;
			await locals.user.save();
			locals.loginCache.delete(locals.user.username)
		} catch (e) {
			return error(500, e instanceof Error ? e.message : '');
		}
		throw redirect(303, '/day');
	},
	addCalendarView: async ({ request, locals }) => {
		const data = await request.formData();
		const calendarName = data.get('calendarName') as string;

		await locals.backend.check(calendarName);
		const calendars = locals.user.calendars ?? [];
		locals.user.calendars = [
			...calendars,
			{
				provider: 'parent',
				type: 'extend',
				name: calendarName
			}
		];
		await locals.user.save();
		locals.loginCache.delete(locals.user.username)
		throw redirect(302, '/day');
	},
	resync: async ({ locals }) => {
		const { backend } = locals;
		await backend.initialSync(true);
		await Promise.all(locals.user.calendars.map(async (c) => backend.initialSync(false, c.name)));
		return { ok: true };
	}
};
