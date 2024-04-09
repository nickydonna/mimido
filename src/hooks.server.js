import { getBackend } from "$lib/server/calendar";
import { verifyToken } from "$lib/server/cognito";
import { UserModel } from "$lib/server/db";
import { redirect } from "@sveltejs/kit";

const unProtectedRoutes = ['/', '/cognito', '/sign-in', '/sign-up'];

/** @type {import("@sveltejs/kit").Handle} */
export const handle = async ({  resolve, event,  }) => {
  event.locals.loggedIn = false;
  const token = event.cookies.get('token');
  const refresh = event.cookies.get('refresh_token');

  if (!token || !refresh) {
    if (!unProtectedRoutes.includes(event.url.pathname)) {
      throw redirect(303, '/');
    }
    return resolve(event)
  }
  const { payload, newToken } = await verifyToken(token, refresh);
  if (newToken) {
    event.cookies.set('token', newToken, {path: '/'})
  }
  event.locals.loggedIn = true;

  const user = await UserModel.get({ username: payload.username })
  event.locals.user = user;
  if (user.main) {
    event.locals.backend = await getBackend(user);
  }

  return resolve(event);
}