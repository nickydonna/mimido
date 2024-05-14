/// <reference types="vite-plugin-pwa/client" />
/// <reference types="vite-plugin-pwa/svelte" />
// See https://kit.svelte.dev/docs/types#app
import 'vite-plugin-pwa/pwa-assets';
import type { CalendarBackend } from '$lib/server/calendar';
import { Prisma, User, Calendar } from '@prisma/client'
import { LRUCache } from 'lru-cache';

interface CalendarAccess {
	type: 'oauth' | 'extend';
	provider: 'parent' | 'google';
}

/**
 * Used to reuse the same credentials but query another calendar
 */
interface ExtendCalendarAccess extends CalendarAccess {
	type: 'extend';
	provider: 'parent';
	name: string;
	ctag?: string;
	syncToken?: string;
	url?: string;
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


// 3: This type will include a user and all their posts
type UserWithCalendars = Prisma.UserGetPayload<typeof userWithPosts>

// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			loggedIn: boolean;
			user: User & { calendars: Calendar[] };
			backend: CalendarBackend;
			loginCache: LRUCache<number, User>;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

