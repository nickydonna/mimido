/// <reference types="vite-plugin-pwa/client" />
/// <reference types="vite-plugin-pwa/svelte" />
// See https://kit.svelte.dev/docs/types#app
import 'vite-plugin-pwa/pwa-assets';
import type { CalendarBackend } from '$lib/server/calendar';
import type { Session } from 'svelte-kit-cookie-session';
import type { StringMappingType } from 'typescript';
import type { User } from '$lib/server/db';

interface CalendarAccess {
	type: 'oauth' | 'extend';
	provider: 'parent' | 'google';
}

interface GoogleCalendarAccess extends CalendarAccess {
	type: 'oauth';
	provider: 'google';
	accessToken: string;
	refreshToken: string;
}

/**
 * Used to reuse the same credentials but query another calendar
 */
interface ExtendCalendarAccess extends CalendarAccess {
	type: 'extend';
	provider: 'parent';
	name: string;
}

interface UserCalendar {
	email: string;
	password: string;
	calendar: string;
	server: string;
	syncToken?: string;
	url?: string;
	ctag?: string;
}

interface CognitoToken {
	access_token: string;
	id_token: string;
	refresh_token: string;
	token_type: 'Bearer';
	expires_in: number;
}

// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			loggedIn: boolean;
			user: User;
			backend: CalendarBackend;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export { UserCalendar, GoogleCalendarAccess, CognitoToken, ExtendCalendarAccess };
