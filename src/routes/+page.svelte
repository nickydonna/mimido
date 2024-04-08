<script>
	import Register from 'flowbite-svelte-blocks/Register.svelte';
	import Section from 'flowbite-svelte-blocks/Section.svelte';
	import { Tabs, TabItem, Spinner } from 'flowbite-svelte';
	import { Button, Label, Input } from 'flowbite-svelte';
	import { enhance } from '$app/forms';


	let signingIn = false;
	let newPasswordRequired = false;
	/** @type {string | undefined }*/
	let sessionId = undefined;
	let email = '';
</script>

<Section name="login">
	<Register href="/">
		<svelte:fragment slot="top">MimiDo</svelte:fragment>
		<Tabs>
			<TabItem open title="Login">
				<div class="space-y-4 p-6 sm:p-8 md:space-y-6">
					{#if newPasswordRequired}
					<form
						class="flex flex-col space-y-6"
						method="POST"
						action="?/newPassword"
						use:enhance={() => {
							signingIn = true;
							return async ({ result }) => {
								signingIn = false;
							};
						}}
					>
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Login</h3>
						<Label class="space-y-2">
							<span>Your email</span>
							<Input
								bind:value={email}
								type="email"
								name="username"
								disable={true}
								placeholder="name@company.com"
								required
							/>
						</Label>
						<Label class="space-y-2">
							<span>Your password</span>
							<Input
								type="password"
								disable={signingIn}
								name="password"
								placeholder="•••••"
								required
							/>
						</Label>
						<input type="text" class="hidden" value={sessionId} name={sessionId}>
						<Button disable={signingIn} type="submit" class="w-full1">
							{#if signingIn}
								<Spinner class="me-3" size="4" />
							{/if}
							Set new password
						</Button>
					</form>
					{:else}
					<form
						class="flex flex-col space-y-6"
						method="POST"
						action="?/cognito"
						use:enhance={() => {
							signingIn = true;
							return async ({ result }) => {
								signingIn = false;
								if (
									result.type === 'failure' &&
									result.data?.exception === 'NEW_PASSWORD_REQUIRED'
								) {
									sessionId = /** @type {string} */ (result.data.sessionId);
									email = /** @type {string} */ (result.data.username);
									newPasswordRequired = true;
								}
							};
						}}
					>
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Login</h3>
						<Label class="space-y-2">
							<span>Your email</span>
							<Input
								bind:value={email}
								type="email"
								name="email"
								disable={signingIn}
								placeholder="name@company.com"
								required
							/>
						</Label>
						<Label class="space-y-2">
							<span>Your password</span>
							<Input
								type="password"
								disable={signingIn}
								name="password"
								placeholder="•••••"
								required
							/>
						</Label>
						<Button disable={signingIn} type="submit" class="w-full1">
							{#if signingIn}
								<Spinner class="me-3" size="4" />
							{/if}
							Login
						</Button>
					</form>
					{/if}
					<!-- <form class="flex flex-col space-y-6" method="POST" action="?/login">
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Login</h3>
						<Label class="space-y-2">
							<span>CalDav Server</span>
							<Input type="url" name="server" placeholder="https://caldav.fastmail.com/" required />
						</Label>
						<Label class="space-y-2">
							<span>Your email</span>
							<Input type="email" name="email" placeholder="name@company.com" required />
						</Label>
						<Label class="space-y-2">
							<span>Your password</span>
							<Input type="password" name="password" placeholder="•••••" required />
						</Label>
						<Label class="space-y-2">
							<span>Calendar</span>
							<Input type="text" name="calendar" placeholder="Calendar to use as store" required />
						</Label>
						<Button type="submit" class="w-full1">Connect</Button>
					</form> -->
				</div>
			</TabItem>
			<TabItem title="Import Token">
				<div class="space-y-4 p-6 sm:p-8 md:space-y-6">
					<form class="flex flex-col space-y-6" method="POST" action="?/import">
						<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">From Token</h3>
						<Label class="space-y-2">
							<span>Token</span>
							<Input type="password" name="token" placeholder="...." required />
						</Label>
						<Button type="submit" class="w-full1">Connect</Button>
					</form>
				</div>
			</TabItem>
		</Tabs>
	</Register>
</Section>
