import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import { prisma } from '$lib/server/prisma';
import bcrypt from 'bcrypt'
import jwt from 'jsonwebtoken'
import { SUDO_PASSWORD } from '$env/static/private';

export const load: PageServerLoad = async ({ locals }) => {
	if (locals.loggedIn) {
		throw redirect(303, '/day');
	}
};

export const actions: Actions = {
	login: async ({ request, locals, params, cookies }) => {
		const data = await request.formData();
		const email = data.get('email') as string;
		const password = data.get('password') as string;

		if (!email || !password) {
			return fail(400, { error: 'Bad Params', email, })
		}

		const user = await prisma.user.findUnique({ where: { email } })
		if (!user) {
			return fail(400, { error: 'Bad Email', email })
		}

		const correctPassword = bcrypt.compareSync(password, user?.password)
		if (!correctPassword) {
			return fail(400, { error: 'Bad Password', email })
		}

		const session = jwt.sign({ email: user.email, id: user.id }, SUDO_PASSWORD)

		cookies.set('session', session, { path: '/' });
		return redirect(303, '/account')

	}
}
