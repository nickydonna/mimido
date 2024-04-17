<script lang="ts">
	import { Section, Register } from 'flowbite-svelte-blocks';
	import { Button, Label, Input, Skeleton } from 'flowbite-svelte';
	import { signIn, getCurrentUser, fetchAuthSession } from 'aws-amplify/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { cognitoLogin, tryGetToken } from '$lib/utils/cognitoClient';

	let username: string;
	let password: string;
	let promise: ReturnType<typeof tryGetToken> = new Promise(() => {
	});

	onMount(async () => {
		promise = tryGetToken();

		const user = await promise;

		if (user) {
			goto('/day')
		}

	});

	async function login() {
		await cognitoLogin(username, password);
		await goto('/day');
	}
</script>

<Section name="login">
	{#await promise}
		<Skeleton size="sm" class="my-8" />
		<Skeleton size="md" class="my-8" />
		<Skeleton size="lg" class="my-8" />
		<Skeleton size="xl" class="my-8" />
		<Skeleton size="xxl" class="mt-8 mb-2.5" />
	{:then user}
		{#if user}
			Redirecting...
		{:else}
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
		{/if}
	{/await}
</Section>
