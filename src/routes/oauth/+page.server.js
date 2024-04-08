
/** @type {import('./$types').PageServerLoad} */
export const load = async () => {
  throw new Error('Not implemented');
  // const code = url.searchParams.get('code');
  // if (!code) {
  //   console.error('no code in oauth');
  //   throw redirect(302, '/');
  // }

  // const {tokens} = await handleCode(code)
  // const { access_token, refresh_token } = tokens;
  // console.log(tokens);
  // if (!access_token || !refresh_token) {
  //   if (dev) {
  //     console.error('no token in handling code', access_token, refresh_token);
  //   } else {
  //     console.error('no token in handling code');
  //   }
  //   throw redirect(302, '/');
  // }
  // /** @type {import('../../app').GoogleCalendarAccess} */
  // const nCal = {
  //   accessToken: access_token,
  //   refreshToken: refresh_token,
  //   provider: 'google',
  //   type: 'oauth',
  // }

  // const { user } = locals.session.data;
  // await locals.session.set({
  //   user,
  //   calendars: [nCal]
  // })
  // throw redirect(302, '/');
}

