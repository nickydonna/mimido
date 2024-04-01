import { getBackend } from "$lib/server/calendar";
import { redirect } from "@sveltejs/kit";
import { SESSION_SECRET } from '$env/static/private';
import { handleSession } from "svelte-kit-cookie-session";
import { sequence } from "@sveltejs/kit/hooks";

const unProtectedRoutes = ['/', '/sign-in', '/sign-up'];

const sessionHandler = handleSession({
	secret: SESSION_SECRET
});

export const handle = sequence(sessionHandler, async ({ resolve, event }) => {
  const { user } = event.locals.session.data;
  if (!user) {
    if (!unProtectedRoutes.includes(event.url.pathname)) {
      throw redirect(303, '/');
    }
    return resolve(event)
  }

  // @ts-expect-error type constrain
  event.locals.backend = await getBackend({ ...user, type: 'basic' });
  return resolve(event);
});