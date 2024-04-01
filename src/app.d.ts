/// <reference types="vite-plugin-pwa/client" />
/// <reference types="vite-plugin-pwa/svelte" />
// See https://kit.svelte.dev/docs/types#app
import 'vite-plugin-pwa/pwa-assets';
import type { CalendarBackend } from "$lib/server/calendar";
import type { Session } from 'svelte-kit-cookie-session';
import type { StringMappingType } from 'typescript';

interface GoogleCalendarAccess {
	type: 'google',
	accessToken: string,
	refreshToken: string,
}

interface User {
	email: string,
	password: string,
	calendar: string,
	server: string,
}

interface SessionData {
	user: User,
	calendars: Array<GoogleCalendarAccess>
};


// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			session: Session<SessionData>;
			user: { email: string, password: string, calendar: string, server: string}
			backend: CalendarBackend,
		}
		interface PageData {
			session: SessionData;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {
	User,
	GoogleCalendarAccess,
};
