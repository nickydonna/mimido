import { type Event } from '../bindings';

export type ParsedEvent = Omit<Event, "starts_at" | "ends_at"> & {
  starts_at: Date;
  ends_at: Date;
  natural_recurrence?: string;
};



const IMPORTANCE_STRINGS = ['Sub-Zero', 'Very Low', 'Low', undefined, 'Mid', 'High', 'Very High'];
const URGENCY_STRINGS = [undefined, 'Soon', 'Next Up', 'Why are you not doing it'];
const LOAD_STRINGS = [undefined, 'Mid', 'Hard', 'Fat Rolling'];

function withSufix(str: string | undefined, sufix: string | undefined): string {
  if (!str) return '';
  if (!sufix) return str;
  return `${str} ${sufix}`;
}

function getString(list: Array<string | undefined>) {
  return (n: number = 0, sufix?: string) => withSufix(list[n], sufix);
}

export const importanceToString = (importance: number = 0, sufix?: string) =>
  getString(IMPORTANCE_STRINGS)(importance + 3, sufix);
export const urgencyToString = getString(URGENCY_STRINGS);
export const loadToString = getString(LOAD_STRINGS);
