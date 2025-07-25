import { commands } from "../../bindings";
import { unwrap } from "$lib/result";
import type { PageLoad } from "./$types";
import {
  parseISO,
  formatISO,
  isValid,
} from "date-fns";

export const load: PageLoad = async ({ url }) => {
  const dateParam = url.searchParams.get("date");
  let date = dateParam ? parseISO(dateParam) : new Date();
  date = isValid(date) ? date : new Date();

  const result = await commands.listEventsForDay(formatISO(date))
  const unwrapped = unwrap(result);
  const events = unwrapped.map((e) => ({
    ...e.event,
    starts_at: parseISO(e.starts_at),
    ends_at: parseISO(e.ends_at),
    natural_recurrence: e.natural_recurrence ?? undefined,
    natural_string: e.natural_string
  }));
  return { events, date };

}
