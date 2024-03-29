import { getBackend } from "$lib/server/calendar";
import { redirect } from "@sveltejs/kit";
import jwt from 'jsonwebtoken';
import { SESSION_KEY } from '$env/static/private';
import { dev } from "$app/environment";

const unProtectedRoutes = ['/', '/sign-in', '/sign-up'];

/** @type {import('@sveltejs/kit').Handle} */
export const handle = async ({ event, resolve }) => {
  const session = event.cookies.get('session');

  if (!session) {
    if (!unProtectedRoutes.includes(event.url.pathname)) {
      throw redirect(303, '/');
    }
    return resolve(event)
  }

  try {
    const user = /** @type {App.Locals['user']} */ (jwt.verify(session, SESSION_KEY))
    event.locals.user = user;
    event.locals.backend = await getBackend(user);
    return resolve(event);
  } catch (e) {
    if (e instanceof jwt.JsonWebTokenError) {
      event.cookies.delete('session', {
        path: '/',
        httpOnly: true,
        sameSite: 'strict',
        secure: !dev 
      });
      throw redirect(303, '/');
    }
    throw e;
  }
}