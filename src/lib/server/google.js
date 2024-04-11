import { google } from 'googleapis';
import { GOOGLE_CLIENT_SECRET, GOOGLE_CLIENT_ID, DOMAIN } from '$env/static/private';
import { dev } from '$app/environment';

function getClient() {
	const oauth2Client = new google.auth.OAuth2(
		GOOGLE_CLIENT_ID,
		GOOGLE_CLIENT_SECRET,
		dev ? 'https://redirectmeto.com/http://localhost:5173/oauth' : `${DOMAIN}/oauth`
	);
	return oauth2Client;
}

// generate a url that asks permissions for Blogger and Google Calendar scopes
const scopes = [
	'https://www.googleapis.com/auth/calendar.events',
	'https://www.googleapis.com/auth/calendar.readonly'
];

export function getAuthUrl() {
	const url = getClient().generateAuthUrl({
		// 'online' (default) or 'offline' (gets refresh_token)
		access_type: 'offline',
		// If you only need one scope you can pass it as a string
		scope: scopes
	});
	return url;
}

/**
 *
 * @param {string} code
 */
export async function handleCode(code) {
	const oauth2Client = getClient();
	const { tokens } = await oauth2Client.getToken(code);
	oauth2Client.setCredentials(tokens);
	return { tokens, client: oauth2Client };
}

/**
 *
 * @param {import('../../app').GoogleCalendarAccess} access
 * @returns
 */
export async function getAuthedClient(access) {
	const client = getClient();
	client.setCredentials({ access_token: access.accessToken, refresh_token: access.refreshToken });
	return client;
}
