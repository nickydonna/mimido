import { Backend } from "$lib/server/calendar";
import { redirect } from "@sveltejs/kit";
import jwt from 'jsonwebtoken';

const key = process.env.SESSION_KEY ?? 'test';
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

  const user = /** @type {App.Locals['user']} */ (jwt.verify(session, key))
  const back = new Backend(user);
  event.locals.user = user;
  event.locals.backend = back;
  return resolve(event);
}