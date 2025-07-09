// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)

import { unwrap } from "$lib/result";
import { commands } from "../bindings";
import type { LayoutLoad } from "./$types";

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load: LayoutLoad = async () => {
  const result = await commands.listCalendars();
  const calendars = unwrap(result);

  return { defaultCalendar: calendars.find(cal => cal.default_value) };

}


