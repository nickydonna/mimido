import { derived, writable } from 'svelte/store';
import type { TAllTypesWithId } from '$lib/server/calendar';

const loadingStore = writable<number>(0);

export const loading = {
	subscribe: loadingStore.subscribe,
	increase: () => loadingStore.update((n) => n + 1),
	// lower cap to 0
	decrease: () => loadingStore.update((n) => Math.max(n - 1, 0))
};
export const hideCreateDrawer = writable<boolean>(true);

const _upsert = writable({
	editing: undefined as TAllTypesWithId | undefined,
	creating: false,
	date: undefined as Date | undefined
});

export const upsert = {
	subscribe: _upsert.subscribe,
	reset() {
		_upsert.set({ editing: undefined, creating: false, date: undefined });
	},
	create(date?: Date) {
		_upsert.set({ date, creating: true, editing: undefined });
	},
	update(event: TAllTypesWithId) {
		_upsert.set({ editing: event, creating: false, date: undefined });
	}
};
export const isUpserting = derived(_upsert, ($upsert) => $upsert.creating || !!$upsert.editing);
export const isLoading = derived(loadingStore, ($loading) => $loading > 0);
