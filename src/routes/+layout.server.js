
/** @type {import('./$types').LayoutServerLoad} */
export const load = async ({ cookies }) => {
  // This doesn't seem safe ... but ok for now
  if (cookies.get('session')) {
    return { token: cookies.get('session')} 
  }
  return {};
}

