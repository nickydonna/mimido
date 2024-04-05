import { writable } from 'svelte/store';

/** @typedef {import('$lib/server/calendar/index.js').TAllTypesWithId} TAllTypesWithId */

/** @type {import('svelte/store').Writable<TAllTypesWithId | undefined>} */
export const selectedEvent = writable();
