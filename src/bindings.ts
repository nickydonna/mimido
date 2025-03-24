
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async createServer(serverUrl: string, user: string, password: string) : Promise<Server> {
    return await TAURI_INVOKE("create_server", { serverUrl, user, password });
},
async listServers() : Promise<Server[]> {
    return await TAURI_INVOKE("list_servers");
},
async listCalendars() : Promise<Result<Calendar[], string>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("list_calendars") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async fetchCalendars(serverId: number) : Promise<Result<Calendar[], string>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("fetch_calendars", { serverId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async syncCalendar(calendarId: number) : Promise<Result<null, string>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("sync_calendar", { calendarId }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async syncAllCalendars() : Promise<Result<null, string>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("sync_all_calendars") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async listEventsForDay(datetime: string) : Promise<Result<Event[], string>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("list_events_for_day", { datetime }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
}
}

/** user-defined events **/



/** user-defined constants **/



/** user-defined types **/

export type Calendar = { id: number; name: string; url: string; etag: string | null; server_id: number }
export type Event = { id: number; calendar_id: number; uid: string; href: string; ical_data: string; summary: string; description: string | null; starts_at: string; ends_at: string; has_rrule: boolean; tag: string | null; status: EventStatus; event_type: EventType; original_text: string | null; load: number; urgency: number; importance: number; postponed: number; last_modified: string }
export type EventStatus = "Backlog" | "Todo" | "Doing" | "Done"
export type EventType = "Event" | "Block" | "Reminder" | "Task"
export type Server = { id: number; server_url: string; user: string; password: string; last_sync: string | null }

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
