import { SESSION_KEY } from '$env/static/private';
import { getAuthUrl } from '$lib/server/google';
import jwt from 'jsonwebtoken';

/** @type {import('./$types').PageServerLoad<{ token: string, googleUrl: string }>} */
export const load = async ({ locals }) => {
  // This doesn't seem safe ... but ok for no
  const token = jwt.sign(locals.session.data, SESSION_KEY);
  const googleUrl = getAuthUrl();
  return { token: token, googleUrl } 
}

