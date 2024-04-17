import { Amplify } from 'aws-amplify';
import { cognitoUserPoolsTokenProvider } from 'aws-amplify/auth/cognito';
import { CookieStorage } from 'aws-amplify/utils';
import { fetchAuthSession, getCurrentUser, signIn } from 'aws-amplify/auth';
import { PUBLIC_COGNITO_CLIENT_ID } from '$env/static/public';
import { timeStore } from '$lib/util';
import { getMinutes } from 'date-fns';

Amplify.configure({
	Auth: {
		Cognito: {
			userPoolClientId: PUBLIC_COGNITO_CLIENT_ID,
			userPoolId: 'us-east-1_p74VfoeuG',
			loginWith: {
				email: true
			}
		}
	}
});

cognitoUserPoolsTokenProvider.setKeyValueStorage(new CookieStorage());

let syncing = false;

function startBackgroundSync() {
	if (syncing) return;
	console.log('Started Token syncing');
	syncing = true;
	timeStore.subscribe((t) => {
		const minutes = getMinutes(t);
		if (minutes % 5 === 0) {
			fetchAuthSession().then(() => console.log('FetchAuthSession'));
		}
	});
}

export async function tryGetToken() {
	try {
		const user = await getCurrentUser();
		await fetchAuthSession();
		startBackgroundSync();
		return user;
	} catch (e) {
		console.log('GetToken failed with: ', e);
		return undefined;
	}
}

export async function cognitoLogin(username: string, password: string) {
	return await signIn({
		username,
		password
	});
}
