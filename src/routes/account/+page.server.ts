import { error, redirect } from '@sveltejs/kit';
import { CalendarBackend } from '$lib/server/calendar/index.js';
import type { Actions, PageServerLoad } from './$types';
import { prisma } from '$lib/server/prisma';

/** @type {import('./$types').PageServerLoad} */
export const load: PageServerLoad = async ({ locals }) => {
	const user = await prisma.user.findUniqueOrThrow({
		where: { id: locals.user.id },
		include: { calendars: true }
	})
	const mainIdx = user.calendars.findIndex(c => c.type === 'main')
	if (mainIdx >= 0) {
		return {
			main: user.calendars[mainIdx], calendars: [...user.calendars].splice(mainIdx, 1)
		}
	}
	return { calendars: user.calendars }

};

export const actions: Actions = {
	setCalendar: async ({ request, locals }) => {
		const data = await request.formData();
		const email = data.get('email') as string;
		const password = data.get('password') as string;
		const server = data.get('server') as string;
		const calendar = data.get('calendar') as string;
		const auth = { email, password, server, calendar };
		// Fake id for checking
		const checkBack = new CalendarBackend({ ...auth, id: -1, ctag: null, syncToken: null, url: '', userId: locals.user.id, type: 'basic' });
		try {
			await checkBack.check();
			const { id } = locals.user;
			const user = await prisma.user.findUniqueOrThrow({ where: { id }, include: { calendars: true } })
			const main = user.calendars.find(c => c.type === 'main')
			let calendar;
			if (main) {
				calendar = await prisma.calendar.update({
					where: { id: main.id }, data: {
						...auth,
					}
				})
			} else {
				calendar = await prisma.calendar.create({
					data: {
						...auth,
						type: 'main',
						url: await checkBack.getCalendarUrl(),
						user: {
							connect: {
								id: user.id
							}
						}
					},
				})
			}
			const back = new CalendarBackend(calendar);
			const syncResult = await back.initialSync(true);
			await prisma.calendarObject.update({ where: { id: calendar.id }, data: syncResult })
		} catch (e) {
			return error(500, e instanceof Error ? e.message : '');
		}
		throw redirect(303, '/day');
	},
	addCalendarView: async ({ request, locals }) => {
		const data = await request.formData();
		const calendarName = data.get('calendarName') as string;

		await locals.backend.check(calendarName);
		const { id, ...calendarInfo } = locals.backend.calendarInfo;
		await prisma.calendar.create({
			data: {
				...calendarInfo,
				calendar: calendarName,
				userId: locals.user.id,
				type: 'extend-basic',
			}
		})
		throw redirect(302, '/account');
	},
	resync: async ({ locals }) => {
		const { backend, user } = locals;
		const calendars = await prisma.calendar.findMany({ where: { userId: user.id } });
		await Promise.all(
			calendars.map(async (c) => {
				const syncResult = await backend.initialSync(c.type === 'main', c);
				console.log(c.calendar, syncResult);
				await prisma.calendar.update({ where: { id: c.id }, data: syncResult })
			})
		);
		return { ok: true };
	}
};
