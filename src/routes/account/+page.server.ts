import { error, redirect } from '@sveltejs/kit';
import { CalendarBackend } from '$lib/server/calendar/index.js';
import type { Actions, PageServerLoad } from './$types';

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
			const syncResult = await back.initialSync(true);
			locals.user.main = {
				...auth,
				...syncResult
			};
			await locals.user.save();
			locals.loginCache.delete(locals.user.username);
		} catch (e) {
			return error(500, e instanceof Error ? e.message : '');
		}
		throw redirect(303, '/day');
	},
	addCalendarView: async ({ request, locals }) => {
		const data = await request.formData();
		const calendarName = data.get('calendarName') as string;

		await locals.backend.check(calendarName);
		const syncResult = await locals.backend.initialSync(false, calendarName);
		const calendars = locals.user.calendars ?? [];
		locals.user.calendars = [
			...calendars,
			{
				provider: 'parent',
				type: 'extend',
				name: calendarName,
				...syncResult
			}
		];
		await locals.user.save();
		locals.loginCache.delete(locals.user.username);
		throw redirect(302, '/day');
	},
	resync: async ({ locals }) => {
		const { backend, user } = locals;
		const syncResult = await backend.initialSync(true);
		console.log('main', syncResult);
		user.calendars = await Promise.all(
			user.calendars.map(async (c) => {
				const syncResult = await backend.initialSync(false, c.name);
				console.log(c.name, syncResult);
				return { ...c, ...syncResult };
			})
		);
		user.main = { ...user.main, ...syncResult };
		console.log(user.toJSON());
		await user.save();
		return { ok: true };
	}
};
