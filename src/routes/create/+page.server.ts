import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { env } from '$env/dynamic/private';
import { prisma } from '$lib/server/prisma';

export const load: PageServerLoad = async ({ locals, url }) => {
  const sudoPassword = url.searchParams.get('sudo');
  if (sudoPassword !== env.SUDO_PASSWORD) {
    return error(500, 'No no no')
  }

  const email = url.searchParams.get('email') as string
  const password = url.searchParams.get('password') as string

  if (!email || !password) {
    return error(400, 'No no no')
  }

  const user = await prisma.user.create({
    data: {
      email,
      password
    }
  })
  console.log(user)

  return { user }

};


