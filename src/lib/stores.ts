import { derived, writable } from 'svelte/store';
import type { TAllTypesWithId } from '$lib/server/calendar';

export const selectedEvent = writable<TAllTypesWithId | undefined>();

const loadingStore = writable<number>(0);

export const loading = {
	subscribe: loadingStore.subscribe,
	increase: () => loadingStore.update((n) => n + 1),
	// lower cap to 0
	decrease: () => loadingStore.update((n) => Math.max(n - 1, 0))
};

export const isLoading = derived(loadingStore, ($loading) => $loading > 0);
