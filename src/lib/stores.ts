import { writable } from 'svelte/store';
import type { TAllTypesWithId } from '$lib/server/calendar';

export const selectedEvent = writable<TAllTypesWithId | undefined>();
