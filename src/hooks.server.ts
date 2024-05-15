import { getBackend } from '$lib/server/calendar/index.js';
import { redirect } from '@sveltejs/kit';
import jwt from 'jsonwebtoken'
import { env } from '$env/dynamic/private';
import { prisma } from '$lib/server/prisma';

const unProtectedRoutes = ['/', '/create', '/sign-in', '/sign-up'];

export const handle: import('@sveltejs/kit').Handle = async ({ resolve, event }) => {
	const token = event.cookies.get('session');
	let user;
	try {
		user = jwt.verify(token ?? '', env.SUDO_PASSWORD) as { id: number, email: string }
	} catch (e) { /* noop */ }

	if (!token || !user) {
		if (!unProtectedRoutes.includes(event.url.pathname)) {
			throw redirect(303, '/');
		}
		return resolve(event);
	}

	event.locals.loggedIn = true;

	event.locals.user = await prisma.user.findUniqueOrThrow({
		where: { id: user.id },
		include: { calendars: true }
	})

	const main = event.locals.user.calendars.find(c => c.type === 'main')

	if (main) {
		event.locals.backend = await getBackend(main.id, main);
	}
	return resolve(event);
};
