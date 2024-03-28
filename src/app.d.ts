// See https://kit.svelte.dev/docs/types#app
import 'vite-plugin-pwa/pwa-assets';
import type { Backend } from "$lib/server/calendar";

// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			user: { email: string, password: string, calendar: string, server: string}
			backend: Backend,
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
