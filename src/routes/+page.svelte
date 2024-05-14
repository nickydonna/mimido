<script lang="ts">
	import { Section, Register } from 'flowbite-svelte-blocks';
	import { Button, Label, Input, Skeleton } from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { loading } from '$lib/stores';
	import type { ActionData, PageData } from './$types';

	export let data: PageData;

	export let form: ActionData;
	console.log(form);
</script>

<Section name="login">
	{#if data.loggedIn}
		Redirecting...
	{:else}
		<Register href="/">
			<svelte:fragment slot="top">
				<img class="w-8 h-8 mr-2" src="/frog.jpg" alt="logo" />
				MimiDo
			</svelte:fragment>
			<div class="p-6 space-y-4 md:space-y-6 sm:p-8">
				<form class="flex flex-col space-y-6" method="POST" action="?/login">
					<h3 class="text-xl font-medium text-gray-900 dark:text-white p-0">Login</h3>
					{#if form?.error}
						{form.error}
					{/if}
					<Label class="space-y-2">
						<span>Your email</span>
						<Input
							value={form?.email ?? ''}
							type="email"
							name="email"
							placeholder="name@company.com"
							required
						/>
					</Label>
					<Label class="space-y-2">
						<span>Your password</span>
						<Input type="password" name="password" placeholder="•••••" required />
					</Label>
					<Button type="submit" class="w-full1">Sign in</Button>
				</form>
			</div>
		</Register>
	{/if}
</Section>
