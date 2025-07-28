import { addWeeks, constructNow, format, isSameWeek, isThisWeek, isToday, isTomorrow, isYesterday, startOfWeek, subWeeks, type Day } from 'date-fns';
import { type VEvent } from '../bindings';

export type ParsedEvent = Omit<VEvent, "starts_at" | "ends_at"> & {
  starts_at: Date;
  ends_at: Date;
  natural_recurrence?: string;
  natural_string: string;
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

const dateFnsOps = {
  weekStartsOn: 1 as Day
}

export const importanceToString = (importance: number = 0, sufix?: string) =>
  getString(IMPORTANCE_STRINGS)(importance + 3, sufix);
export const urgencyToString = getString(URGENCY_STRINGS);
export const loadToString = getString(LOAD_STRINGS);

export function formatRelativeDay(date: Date): string | null {
  if (isToday(date)) {
    return 'Today';
  }
  if (isTomorrow(date)) {
    return 'Tomorrow';
  }
  if (isThisWeek(date, dateFnsOps)) {
    return `This ${format(date, 'EEEE')}`
  }

  const today = constructNow(date);
  const nextWeek = startOfWeek(addWeeks(today, 1), dateFnsOps);
  if (isSameWeek(date, nextWeek, dateFnsOps)) {
    return `Next ${format(date, 'EEEE')}`
  }

  if (isYesterday(date)) {
    return 'Yesterday';
  }

  const pastWeek = startOfWeek(subWeeks(today, 1), dateFnsOps);
  if (isSameWeek(date, pastWeek, dateFnsOps)) {
    return `Past ${format(date, 'EEEE')}`
  }

  return null
}
