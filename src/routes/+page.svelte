<script lang="ts">
	import { Section, Register } from 'flowbite-svelte-blocks';
	import { Button, Label, Input } from 'flowbite-svelte';
	import { signIn, getCurrentUser, fetchAuthSession } from 'aws-amplify/auth';
	import { PUBLIC_COGNITO_CLIENT_ID } from '$env/static/public';
	import { CookieStorage } from 'aws-amplify/utils';
	import { cognitoUserPoolsTokenProvider } from 'aws-amplify/auth/cognito';
	import { Amplify } from 'aws-amplify';
	import { goto } from '$app/navigation';

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

	let username: string;
	let password: string;

	async function login() {
		try {
			await getCurrentUser();
		} catch (e) {
			await signIn({
				username,
				password
			});
		}
		goto('/day')
	}
</script>

<Section name="login">
	<Register href="/">
		<svelte:fragment slot="top">
			<img class="w-8 h-8 mr-2" src="/frog.jpg" alt="logo" />
			MimiDo
		</svelte:fragment>
		<div class="p-6 space-y-4 md:space-y-6 sm:p-8">
			<form class="flex flex-col space-y-6" action="/?login" on:submit|preventDefault={login}>
				<h3 class="text-xl font-medium text-gray-900 dark:text-white p-0">Login</h3>
				<Label class="space-y-2">
					<span>Your email</span>
					<Input bind:value={username} type="email" name="email" placeholder="name@company.com" required />
				</Label>
				<Label class="space-y-2">
					<span>Your password</span>
					<Input bind:value={password} type="password" name="password" placeholder="•••••" required />
				</Label>
				<Button type="submit" class="w-full1">Sign in</Button>
			</form>
		</div>
	</Register>
</Section>
