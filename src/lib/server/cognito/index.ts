import { COGNITO_CLIENT_ID, COGNITO_CLIENT_SECRET, DOMAIN } from '$env/static/private';
import { CognitoJwtVerifier } from 'aws-jwt-verify';
import { JwtExpiredError } from 'aws-jwt-verify/error';
import type { CognitoToken } from '../../../app';
import type { CognitoAccessTokenPayload } from 'aws-jwt-verify/jwt-model';

const COGNITO_POOL_ID = 'us-east-1_p74VfoeuG';
const COGNITO_UI_ID = 'mimido';

export function getCognitoUIUrl() {
	const params = new URLSearchParams();
	params.set('client_id', COGNITO_CLIENT_ID);
	params.set('response_type', 'code');
	params.set('scope', 'email openid phone');
	// Replace with actual URL
	params.set('redirect_uri', `http://${DOMAIN}/cognito`);
	return (
		'https://' +
		COGNITO_UI_ID +
		'.auth.us-east-1.amazoncognito.com/oauth2/authorize?' +
		params.toString()
	);
}

export async function getTokenFromCode(code: string): Promise<CognitoToken> {
	const params = new URLSearchParams();
	params.set('grant_type', 'authorization_code');
	params.set('redirect_uri', `http://${DOMAIN}/cognito`);
	params.set('code', code);
	const res = await fetch(
		`https://${COGNITO_UI_ID}.auth.us-east-1.amazoncognito.com/oauth2/token`,
		{
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded',
				Authorization: `Basic ${btoa(`${COGNITO_CLIENT_ID}:${COGNITO_CLIENT_SECRET}`)}`
			},
			body: params
		}
	);
	return res.json();
}

/**
 * @param {string} refreshToken
 * @returns {Promise<Omit<import('../../../app').CognitoToken, 'refresh_token'>>}
 */
export async function refreshToken(
	refreshToken: string
): Promise<Omit<CognitoToken, 'refresh_token'>> {
	const params = new URLSearchParams();
	params.set('grant_type', 'refresh_token');
	params.set('refresh_token', refreshToken);
	params.set('client_id', COGNITO_CLIENT_ID);
	params.set('redirect_uri', `http://${DOMAIN}/cognito`);
	const res = await fetch(
		`https://${COGNITO_UI_ID}.auth.us-east-1.amazoncognito.com/oauth2/token`,
		{
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded',
				Authorization: `Basic ${btoa(`${COGNITO_CLIENT_ID}:${COGNITO_CLIENT_SECRET}`)}`
			},
			body: params
		}
	);
	return res.json();
}

const verifier = CognitoJwtVerifier.create({
	userPoolId: COGNITO_POOL_ID,
	tokenUse: 'access',
	clientId: COGNITO_CLIENT_ID
});

export async function verifyToken(
	token: string,
	refresh?: string
): Promise<{ newToken?: string; payload: CognitoAccessTokenPayload }> {
	try {
		const payload = await verifier.verify(token);
		return { payload };
	} catch (e) {
		if (e instanceof JwtExpiredError && refresh) {
			console.log('Token expired, refreshing');
			const newToken = await refreshToken(refresh);
			const { payload } = await verifyToken(newToken.access_token);
			return { payload, newToken: newToken.access_token };
		}
		throw new Error('Token is invalid', { cause: e });
	}
}
